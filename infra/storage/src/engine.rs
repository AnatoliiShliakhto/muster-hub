//! Core storage engine implementation providing sandboxed, atomic, and compressed file I/O.
//!
//! This module contains the primary [`Storage`] handle, which serves as the entry point
//! for all storage operations. It manages the physical filesystem root, handles security
//! enforcement via path resolution, and provides a unified interface for both direct
//! and namespaced access.

use crate::builder::StorageBuilder;
use crate::error::{StorageError, StorageErrorExt};
use crate::maintenance;
use crate::namespace::{NamespaceName, NamespacedStorage};
use crate::security;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::debug;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum Compression {
    #[default]
    None,
    Lz4,
}

impl Compression {
    #[must_use]
    fn compress(self, data: &[u8]) -> Vec<u8> {
        match self {
            Self::None => data.to_vec(),
            Self::Lz4 => lz4_flex::compress_prepend_size(data),
        }
    }

    fn decompress(self, data: &[u8]) -> Result<Vec<u8>, StorageError> {
        match self {
            Self::None => Ok(data.to_vec()),
            Self::Lz4 => {
                lz4_flex::decompress_size_prepended(data).context("Lz4 decompression failed")
            },
        }
    }
}

/// The internal shared state of a [`Storage`] instance.
#[derive(Debug)]
pub struct StorageInner {
    /// The canonicalized physical path on the disk where all data is stored.
    pub(crate) root: PathBuf,
    /// Whether transparent LZ4 compression is globally enabled for this instance.
    pub(crate) compression: Compression,
    /// A unique counter used to generate temporary file names.
    pub(crate) tmp_counter: AtomicU64,
}

/// A thread-safe handle to the storage engine.
///
/// `Storage` provides a sandboxed filesystem environment where all paths are validated
/// to prevent traversal attacks. It supports:
/// - **Atomic Writes**: Prevents file corruption using temporary files and renames.
/// - **Namespacing**: Logical separation of data (e.g., `users/`, `logs/`).
/// - **Transparent Compression**: Optional LZ4 block compression.
/// - **Self-Healing**: Automatic cleanup of stale temporary files on initialization.
///
/// This handle is internally reference-counted (`Arc`) and can be cheaply cloned
/// across threads or tasks.
///
/// # Example
///
/// ```rust
/// use mhub_storage::{Storage, Compression, StorageError};
///
/// #[tokio::main]
/// async fn main() -> Result<(), StorageError> {
///     # let tmp = tempfile::tempdir().unwrap();
///     # let root = tmp.path().join("data");
///     // 1. Build and Initialize
///     let storage = Storage::builder()
///         .root(&root)
///         .create(true)
///         .compression(Compression::Lz4)
///         .connect()
///         .await?;
///
///     // 2. Write and Read from Root
///     storage.write("global.meta", b"root_data").await?;
///     let data = storage.read("global.meta").await?;
///
///     // 3. Create a Namespace
///     let user_storage = storage.namespace("user_001")?;
///     user_storage.write("profile.json", b"{\"name\": \"Alice\"}").await?;
///
///     // 4. Verify path resolution (sharding in action)
///     let path = user_storage.resolve("profile.json").unwrap();
///     println!("Physical path: {}", path.display());
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Storage {
    pub(crate) inner: Arc<StorageInner>,
}

impl Deref for Storage {
    type Target = StorageInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Storage {
    #[must_use = "The storage engine is not initialized until you call .connect()"]
    pub fn builder() -> StorageBuilder {
        StorageBuilder::new()
    }

    /// Returns a namespaced view of the storage engine.
    ///
    /// Namespacing allows you to logically partition your storage (e.g., by user ID,
    /// feature name, or environment) while sharing the same underlying configuration
    /// and security sandbox.
    ///
    /// # Constraints
    /// - Names must be **alphanumeric** (a-z, 0-9) or use **underscores** (`_`).
    /// - Names are automatically converted to **lowercase**.
    /// - Empty names are prohibited.
    ///
    /// # Errors
    /// Returns [`StorageError::PathTraversalAttempt`] if the name is empty or
    /// contains illegal characters.
    pub fn namespace<N>(&self, name: N) -> Result<NamespacedStorage, StorageError>
    where
        N: TryInto<NamespaceName, Error = StorageError>,
    {
        let ns = name.try_into()?;
        Ok(NamespacedStorage::new(self.clone(), ns.0))
    }

    /// Resolves a relative path to a physical path on the disk within the storage root.
    ///
    /// This method performs strict security validation to prevent path traversal attacks:
    /// 1. It ensures the provided path is relative (absolute paths are rejected).
    /// 2. It canonicalizes the path.
    /// 3. It verifies that the resulting physical path is still within the configured `root_dir`.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::PathTraversalAttempt`] if the path tries to escape the sandbox.
    /// Returns [`StorageError::Io`] if the path or its parent cannot be verified on the filesystem.
    pub fn resolve(&self, path: impl AsRef<Path>) -> Result<PathBuf, StorageError> {
        security::resolve_path(&self.root, path)
    }

    /// Internal resolve that adds the namespace and sharding.
    pub(crate) fn resolve_internal(
        &self,
        namespace: Option<&str>,
        path: impl AsRef<Path>,
    ) -> Result<PathBuf, StorageError> {
        security::resolve_sharding(&self.root, namespace, path)
    }

    /// Reads the entire contents of a file from storage into a byte vector.
    ///
    /// If transparent compression is enabled for this storage instance, the data
    /// will be automatically decompressed (LZ4) before being returned.
    ///
    /// # Security
    ///
    /// The path is validated against the sandbox root. Attempting to read outside
    /// the root will result in a security error.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::FileNotFound`] if the path does not exist.
    /// Returns [`StorageError::Decompress`] if the data is corrupted or compression is misconfigured.
    pub async fn read(&self, path: impl AsRef<Path>) -> Result<Vec<u8>, StorageError> {
        self.read_internal(None, path).await
    }

    pub(crate) async fn read_internal(
        &self,
        namespace: Option<&str>,
        path: impl AsRef<Path>,
    ) -> Result<Vec<u8>, StorageError> {
        let resolved = self.resolve_internal(namespace, path)?;

        let data = match fs::read(&resolved).await {
            Ok(data) => data,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Err(StorageError::FileNotFound {
                    message: resolved.display().to_string().into(),
                    context: None,
                });
            },
            Err(err) => {
                return Err(StorageError::Io {
                    source: err,
                    context: Some(format!("Read failed: {}", resolved.display()).into()),
                });
            },
        };

        self.inner.compression.decompress(&data)
    }

    /// Writes data to a file in storage atomically.
    ///
    /// This method ensures data integrity by using an "atomic swap" pattern:
    /// 1. Data is written to a unique temporary file (`.mhubtmp.<id>`).
    /// 2. The file is synced to hardware (`fsync`) to ensure it's physically on disk.
    /// 3. The temporary file is renamed to the final destination.
    /// 4. Parent directories and shard directories are created automatically.
    ///
    /// On platforms that do not support atomic replace for existing targets, the
    /// implementation falls back to remove-then-rename.
    ///
    /// If transparent compression is enabled, the data is compressed using LZ4
    /// before being written to disk.
    ///
    /// # Reliability
    ///
    /// Because of the atomic rename, the target file will never be left in a
    /// partially written or corrupted state, even if the system crashes during the writing.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::PathTraversalAttempt`] if the path escapes the sandbox.
    /// Returns [`StorageError::Io`] if disk space is full or hardware failure occurs.
    pub async fn write(&self, path: impl AsRef<Path>, data: &[u8]) -> Result<(), StorageError> {
        self.write_internal(None, path, data).await
    }

    pub(crate) async fn write_internal(
        &self,
        namespace: Option<&str>,
        path: impl AsRef<Path>,
        data: &[u8],
    ) -> Result<(), StorageError> {
        let resolved = self.resolve_internal(namespace, path)?;

        if let Some(parent) = resolved.parent() {
            fs::create_dir_all(parent)
                .await
                .context(format!("Failed to create shards for {}", resolved.display()))?;
        }

        let temp = unique_tmp_path(&resolved, &self.tmp_counter);

        let final_data = self.inner.compression.compress(data);

        {
            let mut file = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temp)
                .await
                .context(format!("Temp creation failed: {}", temp.display()))?;
            file.write_all(&final_data).await.context("Write failed")?;
            file.sync_all().await.context("Hardware sync failed")?;
        }

        if let Err(err) = fs::rename(&temp, &resolved).await {
            if err.kind() == std::io::ErrorKind::AlreadyExists {
                fs::remove_file(&resolved)
                    .await
                    .context(format!("Failed to replace existing file: {}", resolved.display()))?;
                fs::rename(&temp, &resolved).await.context(format!(
                    "Atomic swap failed: {} -> {}",
                    temp.display(),
                    resolved.display()
                ))?;
            } else {
                return Err(StorageError::Io {
                    source: err,
                    context: Some(
                        format!("Atomic swap failed: {} -> {}", temp.display(), resolved.display())
                            .into(),
                    ),
                });
            }
        }

        if let Some(parent) = resolved.parent() {
            Self::sync_dir(parent).await;
        }

        debug!(path = %resolved.display(), "File saved atomically");
        Ok(())
    }

    /// Deletes a file from the storage sandbox.
    ///
    /// This method resolves the path (including sharding if applicable) and removes
    /// the physical file from the disk.
    ///
    /// # Security
    ///
    /// The path is strictly validated against the sandbox root. Deletion of files
    /// outside the sandbox via traversal is impossible.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Io`] if the file does not exist or if there are
    /// not enough permissions to perform the deletion.
    pub async fn delete(&self, path: impl AsRef<Path>) -> Result<(), StorageError> {
        self.delete_internal(None, path).await
    }

    pub(crate) async fn delete_internal(
        &self,
        namespace: Option<&str>,
        path: impl AsRef<Path>,
    ) -> Result<(), StorageError> {
        let resolved = self.resolve_internal(namespace, path)?;
        match fs::remove_file(&resolved).await {
            Ok(()) => {},
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Err(StorageError::FileNotFound {
                    message: resolved.display().to_string().into(),
                    context: None,
                });
            },
            Err(err) => {
                return Err(StorageError::Io {
                    source: err,
                    context: Some(format!("Failed to delete: {}", resolved.display()).into()),
                });
            },
        }
        debug!(path = %resolved.display(), "File deleted");
        Ok(())
    }

    /// Checks if a file exists within the storage sandbox.
    ///
    /// This performs a metadata check on the resolved physical path.
    ///
    /// # Security
    ///
    /// This method is subject to the same path traversal protections as [`read`] and [`write`].
    ///
    /// # Errors
    ///
    /// Returns `Ok(false)` if the file is not found. Returns an `Err` only if
    /// path resolution fails (e.g., due to a security violation) or if a
    /// critical I/O error occurs.
    pub fn exists(&self, path: impl AsRef<Path>) -> Result<bool, StorageError> {
        let resolved = self.resolve_internal(None, path)?;
        Ok(resolved.exists())
    }

    /// Retrieves filesystem metadata for a file within the sandbox.
    ///
    /// This provides information such as file size, creation/modification times,
    /// and read-only status.
    ///
    /// # Important: Compression Awareness
    ///
    /// If transparent compression is enabled, the `len()` returned by the metadata
    /// represents the **compressed size** on the disk, not the original data size.
    ///
    /// # Security
    ///
    /// The path is resolved and validated against the sandbox root. Accessing
    /// metadata for files outside the sandbox is prohibited.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::FileNotFound`] if the target does not exist.
    /// Returns [`StorageError::Io`] if a hardware or permission error occurs.
    pub async fn metadata(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<std::fs::Metadata, StorageError> {
        let resolved = self.resolve_internal(None, path)?;
        match fs::metadata(&resolved).await {
            Ok(meta) => Ok(meta),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                Err(StorageError::FileNotFound {
                    message: resolved.display().to_string().into(),
                    context: None,
                })
            },
            Err(err) => Err(StorageError::Io {
                source: err,
                context: Some(format!("Failed to get metadata: {}", resolved.display()).into()),
            }),
        }
    }

    pub async fn purge_tmp(&self) {
        maintenance::purge_tmp(&self.root).await;
    }

    async fn sync_dir(path: &Path) {
        match fs::File::open(path).await {
            Ok(dir) => {
                if let Err(err) = dir.sync_all().await {
                    tracing::warn!(path = %path.display(), error = %err, "Directory sync failed");
                }
            },
            Err(err) => {
                tracing::warn!(path = %path.display(), error = %err, "Directory open failed");
            },
        }
    }
}

fn unique_tmp_path(target: &Path, counter: &AtomicU64) -> PathBuf {
    let counter = counter.fetch_add(1, Ordering::Relaxed);
    let file_name = target.file_name().and_then(|s| s.to_str()).unwrap_or("storage");
    let tmp_name = format!("{file_name}.mhubtmp.{counter}");
    target.with_file_name(tmp_name)
}
