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
//! use mhub_db::DatabaseBuilder;
//!
//! #[tokio::main]
//! async fn main() -> mhub_db::Result<()> {
//!     let db = DatabaseBuilder::new("mem://", "muster", "core")
//!         .connect()
//!         .await?;
//!     
//!     Ok(())
//! }
//! ```

mod error;

pub use error::{Error, Result};
use std::time::Duration;
use surrealdb::Surreal;
use surrealdb::engine::any::{Any, connect};
use surrealdb::opt::auth::Root;
use tracing::{error, info, warn};

/// A type alias for a thread-safe `SurrealDB` client using the multi-engine backend.
pub type Database = Surreal<Any>;

/// The maximum number of connection attempts before failing.
const MAX_RETRIES: u32 = 3;

/// The delay between connection attempts.
const RETRY_DELAY: Duration = Duration::from_millis(1000);

/// A fluent builder for configuring and establishing a `SurrealDB` connection.
///
/// This builder ensures that fundamental parameters like the connection URL,
/// namespace, and database name are provided upfront.
#[must_use = "builders do nothing unless you call .connect()"]
pub struct DatabaseBuilder {
    url: String,
    ns: String,
    db: String,
    username: Option<String>,
    password: Option<String>,
}

impl DatabaseBuilder {
    /// Creates a new [`DatabaseBuilder`] with the required connection parameters.
    ///
    /// # Arguments
    /// * `url` - The connection string (e.g., `"rocksdb://path/to/db"` or `"ws://localhost:8000"`)
    /// * `ns` - The namespace to use for the session.
    /// * `db` - The database name to use for the session.
    pub fn new(
        url: impl Into<String>,
        ns: impl Into<String>,
        db: impl Into<String>,
    ) -> Self {
        Self {
            url: url.into(),
            ns: ns.into(),
            db: db.into(),
            username: None,
            password: None,
        }
    }

    /// Configures root-level authentication for the database connection.
    ///
    /// This is typically required when connecting to remote instances (WebSocket/HTTP).
    /// Embedded engines like `mem://` or `rocksdb://` often do not require authentication
    /// unless explicitly configured.
    pub fn with_auth(
        mut self,
        user: impl Into<String>,
        pass: impl Into<String>,
    ) -> Self {
        self.username = Some(user.into());
        self.password = Some(pass.into());
        self
    }

    /// Consumes the builder and attempts to establish a connection to the database.
    ///
    /// # Process
    /// 1. Validates that the namespace and database names are not empty.
    /// 2. Initializes the engine connection.
    /// 3. Performs up to [`MAX_RETRIES`] health checks.
    /// 4. Signs in if credentials were provided via [`with_auth`](Self::with_auth).
    /// 5. Sets the namespace and database for the session.
    ///
    /// # Errors
    /// Returns [`Error::ConnectionFailed`] if the host is unreachable or health checks fail.
    /// Returns [`Error::AuthFailed`] if the provided credentials are rejected.
    pub async fn connect(self) -> Result<Database> {
        if self.ns.is_empty() || self.db.is_empty() {
            Err("Namespace and Database names cannot be empty")?;
        }

        let db = connect(&self.url)
            .await
            .map_err(|e| Error::ConnectionFailed(e.to_string()))?;

        // 1. Connectivity & Health Check with Retries
        let mut retries = 0;
        while let Err(e) = db.health().await {
            if retries >= MAX_RETRIES {
                error!(url = %self.url, "Database health check failed after {MAX_RETRIES} attempts");
                return Err(Error::ConnectionFailed(format!(
                    "Health check failed: {e}"
                )));
            }
            retries += 1;
            warn!(
                url = %self.url,
                "Database not ready (attempt {retries}/{MAX_RETRIES}). Retrying in {RETRY_DELAY:?}..."
            );
            tokio::time::sleep(RETRY_DELAY).await;
        }

        // 2. Authentication
        if let (Some(username), Some(password)) = (self.username, self.password)
        {
            db.signin(Root { username, password })
                .await
                .map_err(|e| Error::AuthFailed(e.to_string()))?;
            info!("Authenticated to SurrealDB as root user");
        }

        // 3. Session Initialization
        db.use_ns(&self.ns).use_db(&self.db).await?;

        info!(
            url = %self.url,
            ns = %self.ns,
            db = %self.db,
            "SurrealDB session established successfully"
        );

        Ok(db)
    }
}
