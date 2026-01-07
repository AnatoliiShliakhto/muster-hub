use crate::engine::{Compression, Storage, StorageInner};
use crate::error::{StorageError, StorageErrorExt};
use private::Sealed;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use tokio::fs;
use tracing::info;

#[derive(Debug, Clone)]
struct StorageConfig {
    compression: Compression,
    create: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self { compression: Compression::None, create: true }
    }
}

#[derive(Debug, Default)]
pub struct NoRoot;
#[derive(Debug)]
pub struct WithRoot(PathBuf);

mod private {
    pub(super) trait Sealed {}
}
impl Sealed for NoRoot {}
impl Sealed for WithRoot {}

#[allow(private_bounds)]
#[derive(Debug, Default)]
pub struct StorageBuilder<S: Sealed = NoRoot> {
    state: S,
    config: StorageConfig,
}

#[allow(private_bounds)]
impl<S: Sealed> StorageBuilder<S> {
    #[must_use = "Sets compression for the storage engine"]
    pub const fn compression(mut self, compression: Compression) -> Self {
        self.config.compression = compression;
        self
    }

    #[must_use = "Sets whether the storage engine should be created if it does not exist"]
    pub const fn create(mut self, enable: bool) -> Self {
        self.config.create = enable;
        self
    }

    fn transition<N: Sealed>(self, state: N) -> StorageBuilder<N> {
        StorageBuilder { state, config: self.config }
    }
}

impl StorageBuilder<NoRoot> {
    #[must_use = "Creates a new storage builder with default configuration"]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use = "Sets the root directory path for the storage engine"]
    pub fn root(self, path: impl Into<PathBuf>) -> StorageBuilder<WithRoot> {
        self.transition(WithRoot(path.into()))
    }
}

impl StorageBuilder<WithRoot> {
    /// Consumes the configuration and initializes the storage engine.
    ///
    /// This method performs the following boot sequence:
    /// 1. **Bootstrapping**: Creates the root directory if `create(true)` was set.
    /// 2. **Canonicalization**: Resolves the root path to an absolute, physical path
    ///    on disk to prevent symlink-based escape attacks.
    /// 3. **Self-Healing**: Scans the root for orphaned `.tmp` files left behind by
    ///    previous system crashes and removes them to reclaim space.
    /// 4. **Registration**: Returns a thread-safe [`Storage`] handle.
    ///
    /// # Reliability
    ///
    /// The self-healing routine is non-critical; if cleanup fails (e.g., due to
    /// transient file locks), the initialization will still proceed, but a
    /// warning will be logged.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError::Io`] if:
    /// - The root directory does not exist and `create` is false.
    /// - The process lacks permissions to create or resolve the root directory.
    /// - The path contains invalid UTF-8 characters on some platforms.
    pub async fn connect(self) -> Result<Storage, StorageError> {
        let root = &self.state.0;

        if self.config.create {
            fs::create_dir_all(root)
                .await
                .context(format!("Failed to bootstrap storage root: {}", root.display()))?;
            info!(path = %root.display(), "Bootstrapped storage root directory");
        }

        let canonical = fs::canonicalize(root)
            .await
            .context(format!("Failed to resolve storage root: {}", root.display()))?;

        let storage = Storage {
            inner: Arc::new(StorageInner {
                root: canonical,
                compression: self.config.compression,
                tmp_counter: AtomicU64::new(1),
            }),
        };

        storage.purge_tmp().await;

        Ok(storage)
    }
}
