#![allow(dead_code)]

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
//! use mhub_database::Database;
//! use mhub_database::error::DatabaseError;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), DatabaseError> {
//!     let db = Database::builder()
//!         .url("mem://")
//!         .session("mhub", "core")
//!         .init()
//!         .await?;
//!
//!     let scoped_db_session = db.authenticate("user123").await?;
//!     let _version = scoped_db_session.version().await?;
//!
//!     Ok(())
//! }
//! ```

mod auth;
pub mod error;
mod generated;
mod migrations;

use crate::auth::AuthProvider;
use crate::error::{DatabaseError, DatabaseErrorExt};
use crate::migrations::MigrationRunner;
use moka::future::Cache;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;
use surrealdb::Surreal;
use surrealdb::engine::any::{Any, connect};
use surrealdb::opt::auth::Root;
use tracing::{info, instrument, trace, warn};

/// TTL in seconds for external JWTs issued for the database.
static JWT_TTL_SECONDS: i64 = 3600;
/// Max bound cache sessions tokens to prevent 'slow lori'
static MAX_CACHE_CAPACITY: u64 = 100_000;

// #[derive(Debug)]
// pub(crate) struct Embedded;
// #[derive(Debug)]
// pub(crate) struct External;
//
// mod private {
//     pub(super) trait Sealed {}
// }
//
// impl private::Sealed for Embedded {}
// impl private::Sealed for External {}

/// Inner state of the [`Database`] wrapper.
#[derive(Debug)]
pub(crate) struct DatabaseInner {
    instance: Surreal<Any>,
    auth: AuthProvider,
    cache: Cache<String, Surreal<Any>>,
    ns: String,
    db: String,
    // kind: PhantomData<K>,
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
#[must_use = "builders do nothing unless you call .init()"]
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
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Sets the namespace and database name.
    pub fn session(mut self, namespace: impl Into<String>, database: impl Into<String>) -> Self {
        self.ns = Some(namespace.into());
        self.db = Some(database.into());
        self
    }

    /// Add root credentials to the connection.
    pub fn auth(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
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
    /// 4. **Authentication**: If credentials were provided via [`auth`], signs in as a Root user.
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
    #[instrument(skip(self))]
    pub async fn init(self) -> Result<Database, DatabaseError> {
        let url =
            self.url.ok_or_else(|| DatabaseError::validation().with_context("URL is required"))?;
        let ns = self
            .ns
            .ok_or_else(|| DatabaseError::validation().with_context("namespace is required"))?;
        let db = self
            .db
            .ok_or_else(|| DatabaseError::validation().with_context("database is required"))?;

        let is_embedded = is_embedded(&url)?;
        let instance =
            connect(&url).with_capacity(1024).await.with_context("initializing engine")?;

        // 1. Connectivity & Health Check with Retries
        let mut delay = Duration::from_millis(500);
        for attempt in 1..=3 {
            if instance.health().await.is_ok() {
                break;
            }
            if attempt == 3 {
                Err(DatabaseError::connection().with_context("unhealthy after retries"))?;
            }
            warn!(attempt, ?delay, "Database not ready, retrying...");
            tokio::time::sleep(delay).await;
            delay *= 2;
        }

        // 2. Authentication
        if !is_embedded && let Some((u, p)) = self.auth {
            instance
                .signin(Root { username: u, password: p })
                .await
                .with_context("authenticating as root")?;
        }

        // 3. Session Initialization
        instance.use_ns(&ns).use_db(&db).await.with_context("activating session")?;

        let version =
            instance.version().await.map_or_else(|_e| "unknown".to_owned(), |v| v.to_string());
        info!(namespace = %ns, database = %db, %version, "SurrealDB connection established");

        info!("Applying database migrations...");
        let migration_report = MigrationRunner::new(instance.clone()).run().await?;
        for skipped in migration_report.skipped {
            trace!(slice = skipped.slice_key, version = skipped.version, "Skipping migration");
        }
        for applied in migration_report.applied {
            info!(slice = applied.slice_key, version = applied.version, "Applied migration");
        }
        info!("Database migrations applied successfully");

        let auth = AuthProvider::init()?;
        auth.setup_database(&instance).await?;

        let cache = Cache::builder()
            .max_capacity(MAX_CACHE_CAPACITY)
            .time_to_live(Duration::from_secs(JWT_TTL_SECONDS.cast_unsigned() - 60)) // (-1 minute of JWT)
            .build();

        Ok(Database {
            inner: Arc::new(DatabaseInner {
                instance,
                auth,
                cache,
                ns,
                db,
                // kind: PhantomData::<Embedded>,
            }),
        })
    }
}

impl Database {
    /// Authenticates as a specific user and returns a scoped `SurrealDB` client session.
    ///
    /// This method creates (or reuses) an authenticated session for the given `user_id`.
    /// Internally, it generates a short-lived JWT for the user scope and calls SurrealDB’s
    /// `authenticate(...)`. Successful sessions may be cached, so repeated calls for the same
    /// `user_id` avoid it re-authenticating until the cache entry expires.
    ///
    /// # Parameters
    /// - `user_id`: The user identifier used to build the scope subject (e.g. `user:{user_id}`).
    ///
    /// # Returns
    /// - `Ok(Surreal<Any>)`: A cloned `SurrealDB` client handle authenticated for the
    ///   requested user scope (ready to run queries as that user).
    /// - `Err(DatabaseError)`: If token creation, authentication, or internal caching fails.
    ///
    /// # Errors
    /// This function can return:
    /// - [`DatabaseError::Auth`]:
    ///   - If JWT encoding/signing fails;
    ///   - If `SurrealDB` rejects the token during `authenticate(...)`.
    /// - [`DatabaseError::Internal`]:
    ///   - If an internal caching/loading invariant is violated (e.g., an error is returned
    ///     from the cache layer in an unexpected shared form).
    ///
    /// Notes:
    /// - This method does **not** validate that the user exists; it authenticates using the
    ///   provided `user_id` as the token subject.
    /// - Connection/engine failures may surface indirectly via authentication errors depending
    ///   on the underlying client behavior.
    #[instrument(skip(self), fields(user_id = %user_id.as_ref()))]
    pub async fn authenticate(
        &self,
        user_id: impl AsRef<str>,
    ) -> Result<Surreal<Any>, DatabaseError> {
        let user_id = if user_id.as_ref().starts_with("user:") {
            user_id.as_ref()
        } else {
            &format!("user:{}", user_id.as_ref())
        };

        self.inner
            .cache
            .try_get_with(user_id.to_owned(), async {
                // let claims = Claims {
                //     ns: &self.inner.ns,
                //     db: &self.inner.db,
                //     ac: "user",
                //     id: user_id.to_owned(),
                //     exp: (chrono::Utc::now() + chrono::Duration::seconds(JWT_TTL_SECONDS))
                //         .timestamp(),
                // };

                // let token = encode(
                //     &Header::new(jsonwebtoken::Algorithm::EdDSA),
                //     &claims,
                //     &self.inner.auth.encoding_key,
                // )
                // .with_context("failed to encode token")?;

                let scoped_instance = self.inner.instance.clone();
                scoped_instance
                    .set("user", user_id.to_owned())
                    .await
                    .with_context("setting user scope")?;
                // scoped_instance.authenticate(token).await.map_err(|e| DatabaseError::Auth {
                //     message: e.to_string().into(),
                //     context: Some("User auth session".into()),
                // })?;

                Ok(scoped_instance)
            })
            .await
            .map_err(|e: Arc<DatabaseError>| DatabaseError::from(e))
    }
}

// --- Helper Functions ---
fn is_embedded(url: &str) -> Result<bool, DatabaseError> {
    let (scheme, _) = url.split_once("://").ok_or_else(|| {
        DatabaseError::validation()
            .with_message("invalid database URL")
            .with_context("missing protocol scheme")
    })?;

    match scheme {
        "mem" | "rocksdb" | "file" => Ok(true),
        "ws" | "wss" | "http" | "https" | "tikv" => Ok(false),

        _ => Err(DatabaseError::validation()
            .with_message("unsupported protocol")
            .with_context_fn(|| format!("unsupported protocol: {scheme}"))),
    }
}
