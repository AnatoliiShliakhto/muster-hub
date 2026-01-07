use crate::engine::Storage;
use crate::error::{StorageError, StorageErrorExt};
use std::borrow::Cow;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamespaceName(pub String);

impl TryFrom<String> for NamespaceName {
    type Error = StorageError;

    fn try_from(value: String) -> Result<Self, StorageError> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&str> for NamespaceName {
    type Error = StorageError;

    fn try_from(value: &str) -> Result<Self, StorageError> {
        let name = value.to_lowercase();

        if name.is_empty() {
            return Err(StorageError::PathTraversalAttempt {
                message: "EMPTY".into(),
                context: Some("Namespace cannot be empty".into()),
            });
        }

        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(StorageError::PathTraversalAttempt {
                message: name.into(),
                context: Some("Namespace contains illegal characters".into()),
            });
        }

        Ok(Self(name))
    }
}

impl AsRef<str> for NamespaceName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for NamespaceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A lightweight, namespaced view of the storage engine.
///
/// `NamespacedStorage` provides a scoped interface where all paths are automatically
/// prefixed with the namespace name and subjected to sharding. This is the
/// recommended way to handle multi-tenant data or grouped assets.
///
/// # Characteristics
/// - **Automatic Sharding**: Files are sharded within the namespace directory to
///   maintain filesystem performance, while preserving any subdirectories you provide.
/// - **Inherited Config**: Inherits compression and security settings from the
///   parent [`Storage`] instance.
/// - **Zero Copy**: Cloning a `NamespacedStorage` is inexpensive as it only holds a
///   reference-counted handle to the core engine.
#[derive(Debug, Clone)]
pub struct NamespacedStorage {
    storage: Storage,
    namespace: Arc<Cow<'static, str>>,
}

impl NamespacedStorage {
    pub(crate) fn new(storage: Storage, namespace: impl Into<Cow<'static, str>>) -> Self {
        Self { storage, namespace: Arc::new(namespace.into()) }
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
        self.storage.resolve_internal(Some(&self.namespace), path)
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
        self.storage.read_internal(Some(&self.namespace), path).await
    }

    /// Writes data to a file in storage atomically.
    ///
    /// This method ensures data integrity by using an "atomic swap" pattern:
    /// 1. Data is written to a temporary file (`.tmp`).
    /// 2. The file is synced to hardware (`fsync`) to ensure it's physically on disk.
    /// 3. The temporary file is renamed to the final destination.
    /// 4. If sharding is enabled, parent directories are created automatically.
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
        self.storage.write_internal(Some(&self.namespace), path, data).await
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
        self.storage.delete_internal(Some(&self.namespace), path).await
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
        let resolved = self.storage.resolve_internal(Some(&self.namespace), path)?;
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
        let resolved = self.storage.resolve_internal(Some(&self.namespace), path)?;
        fs::metadata(&resolved)
            .await
            .context(format!("Failed to get metadata: {}", resolved.display()))
    }
}
