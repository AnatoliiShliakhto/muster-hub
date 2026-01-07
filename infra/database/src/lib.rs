//! # Database Infrastructure
//!
//! This crate provides a unified interface for initializing and managing [SurrealDB](https://surrealdb.com)
//! connections across the workspace.
//!
//! ## Key Features
//! - **Engine Agnostic**: Supports `mem://`, `rocksdb://`, `ws://`, and `http://` via the `any` engine.
//! - **Resilient Connectivity**: Built-in retry logic for health checks during engine startup.
//! - **Builder Pattern**: Fluent API for configuring connections and authentication.
//!
//! ## Example
//!
//! ```rust
//! use mhub_db::Database;
//!
//! #[tokio::main]
//! async fn main() -> mhub_db::Result<()> {
//!     use mhub_database::Database;
//! let db = Database::builder()
//!         .with_url("mem://")
//!         .with_session("mhub", "core")
//!         .connect()
//!         .await?;
//!     
//!     Ok(())
//! }
//! ```

mod error;

pub use error::{DatabaseError, DatabaseErrorExt, Result};
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use surrealdb::Surreal;
use surrealdb::engine::any::{Any, connect};
use surrealdb::opt::auth::Root;
use tracing::{info, instrument, warn};

/// Inner state of the [`Database`] wrapper.
#[derive(Debug)]
pub struct DatabaseInner {
    instance: Surreal<Any>,
    ns: String,
    db: String,
}

impl Drop for DatabaseInner {
    fn drop(&mut self) {
        info!(ns = %self.ns, db = %self.db, "SurrealDB session handle dropped");
    }
}

/// `SurrealDB` client wrapper that provides thread-safety and contextual error handling.
#[derive(Debug, Clone)]
pub struct Database {
    inner: Arc<DatabaseInner>,
}

impl Database {
    /// Creates a new [`DatabaseBuilder`].
    pub fn builder() -> DatabaseBuilder {
        DatabaseBuilder::new()
    }
}

impl Deref for Database {
    type Target = Surreal<Any>;

    fn deref(&self) -> &Self::Target {
        &self.inner.instance
    }
}

/// A fluent builder for configuring and establishing a `SurrealDB` connection.
///
/// This builder ensures that fundamental parameters like the connection URL,
/// namespace, and database name are provided upfront.
#[must_use = "builders do nothing unless you call .connect()"]
#[derive(Debug, Default)]
pub struct DatabaseBuilder {
    url: Option<String>,
    ns: Option<String>,
    db: Option<String>,
    auth: Option<(String, String)>,
}

impl DatabaseBuilder {
    /// Creates a new [`DatabaseBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the connection URL.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Sets the namespace and database name.
    pub fn with_session(
        mut self,
        namespace: impl Into<String>,
        database: impl Into<String>,
    ) -> Self {
        self.ns = Some(namespace.into());
        self.db = Some(database.into());
        self
    }

    /// Add root credentials to the connection.
    pub fn with_auth(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.auth = Some((username.into(), password.into()));
        self
    }

    /// Consumes the builder and attempts to establish a connection to the database.
    ///
    /// This method executes the full connection lifecycle, including engine initialization,
    /// health check retries with exponential backoff, authentication, and session activation.
    ///
    /// # Process
    /// 1. **Validation**: Ensures URL, Namespace, and Database name are provided.
    /// 2. **Engine Initialization**: Connects to the underlying `SurrealDB` engine (Any).
    /// 3. **Resilience**: Performs up to 3 health checks using `INFO FOR DB`. If the first check fails,
    ///    it retries with exponential backoff (starting at 500ms).
    /// 4. **Authentication**: If credentials were provided via [`with_auth`], signs in as a Root user.
    /// 5. **Session Activation**: Sets the global namespace and database for the connection.
    ///
    /// # Returns
    /// * `Ok(Database)` - A thread-safe, cloned handle to the established session.
    /// * `Err(DatabaseError)` - Detailed error information if any step of the process fails.
    ///
    /// # Errors
    /// * [`DatabaseError::Validation`] if required parameters are missing.
    /// * [`DatabaseError::Connection`] if the engine fails to start or remains unhealthy.
    /// * [`DatabaseError::Auth`] if the provided credentials are rejected.
    /// * [`DatabaseError::Surreal`] if the session activation (`use_ns`/`use_db`) fails.
    #[instrument(skip(self), fields(url = self.url))]
    pub async fn connect(self) -> Result<Database> {
        let url = self.url.ok_or(DatabaseError::Validation {
            message: "URL is required".into(),
            context: None,
        })?;
        let ns = self.ns.ok_or(DatabaseError::Validation {
            message: "Namespace is required".into(),
            context: None,
        })?;
        let db = self.db.ok_or(DatabaseError::Validation {
            message: "Database is required".into(),
            context: None,
        })?;

        let instance = connect(&url).await.map_err(|e| DatabaseError::Connection {
            message: e.to_string().into(),
            context: Some("Initializing engine".into()),
        })?;

        // 1. Connectivity & Health Check with Retries
        let mut delay = Duration::from_millis(500);
        for attempt in 1..=3 {
            if instance.health().await.is_ok() {
                break;
            }
            if attempt == 3 {
                return Err(DatabaseError::Connection {
                    message: "Unhealthy after retries".into(),
                    context: Some(url.into()),
                });
            }
            warn!(attempt, ?delay, "Database not ready, retrying...");
            tokio::time::sleep(delay).await;
            delay *= 2;
        }

        // 2. Authentication
        if let Some((u, p)) = self.auth {
            instance.signin(Root { username: u, password: p }).await.map_err(|e| {
                DatabaseError::Auth { message: e.to_string().into(), context: Some(url.into()) }
            })?;
        }

        // 3. Session Initialization
        instance.use_ns(&ns).use_db(&db).await.with_context("Activating session")?;

        let version =
            instance.version().await.map_or_else(|_| "unknown".to_owned(), |v| v.to_string());

        info!(namespace = %ns, database = %db, %version, "SurrealDB connection established");

        Ok(Database { inner: Arc::new(DatabaseInner { instance, ns, db }) })
    }
}
