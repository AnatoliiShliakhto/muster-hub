mod error;

pub use crate::error::{Result, StorageError, StorageErrorExt};
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tracing::info;

#[derive(Debug)]
pub struct StorageInner {
    root_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Storage {
    inner: Arc<StorageInner>,
}

impl Storage {
    #[must_use]
    pub fn builder() -> StorageBuilder {
        StorageBuilder::default()
    }

    /// Safely joins a path to the root and ensures it doesn't escape the sandbox.
    pub fn resolve(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
        let joined = self.inner.root_dir.join(path);
        if joined.starts_with(&self.inner.root_dir) {
            Ok(joined)
        } else {
            Err(StorageError::PathTraversalAttempt {
                message: format!(
                    "Attempted to access a path outside the sandbox {path}",
                    path = joined.display()
                )
                .into(),
                context: None,
            })
        }
    }
}

/// A fluent builder for the [`Storage`] engine.
#[derive(Debug, Default)]
pub struct StorageBuilder {
    root_dir: Option<PathBuf>,
    create_if_missing: bool,
}

impl StorageBuilder {
    /// Sets the base directory for all storage operations.
    #[must_use]
    pub fn with_root(mut self, path: impl Into<PathBuf>) -> Self {
        self.root_dir = Some(path.into());
        self
    }

    /// Whether to automatically create the root directory if it doesn't exist.
    #[must_use]
    pub const fn create_dirs(mut self, enable: bool) -> Self {
        self.create_if_missing = enable;
        self
    }

    /// Consumes the builder and initializes the storage engine.
    ///
    /// This is async because it performs I/O checks and directory creation.
    pub async fn build(self) -> Result<Storage> {
        let root = self.root_dir.ok_or_else(|| StorageError::MissingRootDirectory {
            message: "No root directory provided".into(),
            context: None,
        })?;

        if self.create_if_missing {
            fs::create_dir_all(&root).await.with_context(format!(
                "Failed to create root storage directory {}",
                root.display()
            ))?;

            info!(
                root = %root.display(),
                message = "Root storage directory created",
            );
        } else if !root.exists() {
            return Err(StorageError::DirectoryNotFound {
                message: Cow::Owned(root.to_string_lossy().into_owned()),
                context: None,
            });
        }

        // Convert to absolute path to prevent traversal issues
        let root = fs::canonicalize(root)
            .await
            .with_context("Failed to canonicalize root storage directory")?;

        Ok(Storage { inner: Arc::new(StorageInner { root_dir: root }) })
    }
}
