//! # Muster Hub Server
//!
//! A production-ready web server built on `Axum`, `SurrealDB`, and a type-safe event bus.
//!
//! ## Example
//! ```no_run
//! use server::{Server, SslConfig};
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     Server::builder()
//!         .with_host(([0, 0, 0, 0], 4583))
//!         .with_db_url("mem://")
//!         .with_work_dir(".")
//!         .build()
//!         .await?
//!         .run()
//!         .await
//! }
//! ```

mod features;
mod internal;

use anyhow::{Context, Result};
use axum_server::Handle;
use mhub::kernel::api::ApiState;
use mhub_db::DatabaseBuilder;
use mhub_event_bus::EventBus;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use tokio::signal;
use tracing::{error, info};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

/// Default bind address for the server (0.0.0.0:4583)
const DEFAULT_HOST: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 4583);

/// Default database connection string (in-memory)
const DEFAULT_DB_URL: &str = "mem://";

/// Default database namespace
const DEFAULT_DB_NAMESPACE: &str = "mhub";

/// Default database name
const DEFAULT_DB_NAME: &str = "core";

/// Default working directory
const DEFAULT_WORK_DIR: &str = ".";

/// Default public assets directory
const DEFAULT_PUB_DIR: &str = "public";

/// SSL/TLS configuration for HTTPS support.
#[derive(Debug, Clone)]
pub struct SslConfig {
    /// Path to the TLS certificate file (PEM format)
    pub cert: PathBuf,
    /// Path to the private key file (PEM format)
    pub key: PathBuf,
}

/// `SurrealDB` connection configuration.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database connection URL
    pub url: String,
    /// namespace
    pub namespace: String,
    /// database name
    pub database: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: DEFAULT_DB_URL.to_owned(),
            namespace: DEFAULT_DB_NAMESPACE.to_owned(),
            database: DEFAULT_DB_NAME.to_owned(),
        }
    }
}

/// A fluent builder for configuring and initializing the [`Server`].
///
/// This builder follows the same pattern as other workspace crates
/// (e.g., `DatabaseBuilder`, `LoggerBuilder`) for consistency.
#[must_use = "builders do nothing unless you call .build()"]
#[derive(Debug, Default)]
pub struct ServerBuilder {
    host: Option<SocketAddr>,
    db: Option<DatabaseConfig>,
    work_dir: Option<PathBuf>,
    pub_dir: Option<PathBuf>,
    ssl: Option<SslConfig>,
}

impl ServerBuilder {
    /// Configures the server's bind address.
    ///
    /// # Default
    /// `0.0.0.0:4583` if not specified.
    pub fn with_host(mut self, host: impl Into<SocketAddr>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Configures the database connection URL.
    ///
    /// # Arguments
    /// * `url` - Connection string (e.g., `"mem://"`, `"ws://localhost:8000"`)
    /// * `namespace` - Namespace for the database
    /// * `database` - Database name
    pub fn with_db(
        mut self,
        url: impl Into<String>,
        namespace: impl Into<String>,
        database: impl Into<String>,
    ) -> Self {
        self.db = Some(DatabaseConfig {
            url: url.into(),
            namespace: namespace.into(),
            database: database.into(),
        });
        self
    }

    /// Configures the server's working directory.
    ///
    /// This directory is used for database, temporary files, caches, and other runtime data.
    pub fn with_work_dir(mut self, work_dir: impl Into<PathBuf>) -> Self {
        self.work_dir = Some(work_dir.into());
        self
    }

    /// Configures the directory for serving static public assets.
    pub fn with_pub_dir(mut self, pub_dir: impl Into<PathBuf>) -> Self {
        self.pub_dir = Some(pub_dir.into());
        self
    }

    /// Configures SSL/TLS for HTTPS support.
    ///
    /// If not provided, the server will run in HTTP mode.
    pub fn with_ssl(mut self, ssl: SslConfig) -> Self {
        self.ssl = Some(ssl);
        self
    }

    /// Consumes the builder and initializes the server.
    ///
    /// # Process
    /// 1. Applies default values for unspecified configuration
    /// 2. Establishes database connection via [`DatabaseBuilder`]
    /// 3. Initializes event bus for inter-slice communication
    /// 4. Constructs application state
    /// 5. Builds Axum router with all feature slices
    ///
    /// # Errors
    /// Returns an error if:
    /// - Database connection fails (unreachable host, invalid credentials)
    /// - Working directory or public directory is invalid
    /// - SSL certificate/key files cannot be read
    ///
    /// # Examples
    /// ```no_run
    /// # use server::Server;
    /// # async fn example() -> anyhow::Result<()> {
    /// let server = Server::builder()
    ///     .with_host(([127, 0, 0, 1], 8080))
    ///     .with_db_url("ws://localhost:8000")
    ///     .with_db_config("production", "main")
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn build(self) -> Result<Server> {
        // 1. Apply Defaults
        let host = self.host.unwrap_or(DEFAULT_HOST);
        let db_cfg = self.db.unwrap_or_default();
        let work_dir =
            self.work_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_WORK_DIR));
        let pub_dir =
            self.pub_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_PUB_DIR));

        info!(
            host = %host,
            "Initializing server"
        );

        // 2. Initialize Database
        let db = DatabaseBuilder::new(
            &db_cfg.url,
            &db_cfg.namespace,
            &db_cfg.database,
        )
        .connect()
        .await
        .context("Failed to establish database connection")?;

        // 3. Initialize Event Bus
        let event_bus = EventBus::new();

        // 4. Build API State
        let state = ApiState::new(db, event_bus);

        Ok(Server { host, state, work_dir, pub_dir, ssl: self.ssl })
    }
}

/// A fully initialized server instance ready to run.
///
/// This struct is returned by [`ServerBuilder::build`] and contains
/// all necessary runtime state.
#[must_use = "call .run().await to start the server"]
pub struct Server {
    host: SocketAddr,
    state: ApiState,
    work_dir: PathBuf,
    pub_dir: PathBuf,
    ssl: Option<SslConfig>,
}

impl Server {
    /// Returns a new [`ServerBuilder`] to configure the server.
    ///
    /// This is the recommended way to initialize the server.
    ///
    /// # Examples
    /// ```no_run
    /// # use server::Server;
    /// # async fn example() -> anyhow::Result<()> {
    /// let server = Server::builder()
    ///     .with_host(([0, 0, 0, 0], 4583))
    ///     .build()
    ///     .await?;
    ///
    /// server.run().await
    /// # }
    /// ```
    pub fn builder() -> ServerBuilder {
        ServerBuilder::default()
    }

    /// Starts the server and runs until the shutdown signal is received.
    ///
    /// # Errors
    /// Returns an error if the server fails to bind to the configured host
    /// or if SSL/TLS setup fails.
    ///
    /// # Examples
    /// ```no_run
    /// # use server::Server;
    /// # async fn example() -> anyhow::Result<()> {
    /// Server::builder()
    ///     .with_host(([0, 0, 0, 0], 4583))
    ///     .build()
    ///     .await?
    ///     .run()
    ///     .await
    /// # }
    /// ```
    pub async fn run(self) -> Result<()> {
        #[derive(OpenApi)]
        struct ApiDoc;

        info!(
            host = %self.host,
            ssl = self.ssl.is_some(),
            "Starting server"
        );

        let api = ApiDoc::openapi();

        // Separate the OpenAPI routes and the API documentation object
        let (openapi_routes, api_doc) = OpenApiRouter::with_openapi(api)
            .merge(mhub::api::router::system_router())
            .with_state(self.state)
            .split_for_parts();

        // Create the Scalar UI routes
        let scalar_routes = Scalar::with_url("/api", api_doc);

        // Merge all routes and then apply the state to the final router
        let router =
            axum::Router::new().merge(openapi_routes).merge(scalar_routes);

        // 2. Set up Graceful Shutdown
        let handle = Handle::<SocketAddr>::new();
        let shutdown_handle = handle.clone();

        // Spawn shutdown signal listener
        tokio::spawn(async move {
            if let Err(e) = shutdown_signal().await {
                error!("Error while waiting for shutdown signal: {e}");
                return;
            }
            info!("Shutdown signal received, starting graceful shutdown...");
            shutdown_handle
                .graceful_shutdown(Some(std::time::Duration::from_secs(30)));
        });

        // 3. Start Server (HTTP or HTTPS)
        if let Some(ssl_config) = self.ssl {
            // HTTPS mode
            info!("Starting HTTPS server on https://{}", self.host);

            let tls_config =
                axum_server::tls_rustls::RustlsConfig::from_pem_file(
                    &ssl_config.cert,
                    &ssl_config.key,
                )
                .await
                .context("Failed to load SSL/TLS certificates")?;

            axum_server::bind_rustls(self.host, tls_config)
                .handle(handle)
                .serve(router.into_make_service())
                .await
                .context("HTTPS server failed")?;
        } else {
            // HTTP mode
            info!("Starting HTTP server on http://{}", self.host);

            axum_server::bind(self.host)
                .handle(handle)
                .serve(router.into_make_service())
                .await
                .context("HTTP server failed")?;
        }

        info!("Server shutdown complete");
        Ok(())
    }

    /// Returns a reference to the application state.
    #[must_use]
    pub const fn state(&self) -> &ApiState {
        &self.state
    }
}

/// Listens for shutdown signals (Ctrl+C, SIGTERM).
///
/// This function waits for either:
/// - SIGINT (Ctrl+C)
/// - SIGTERM (sent by process managers like systemd)
async fn shutdown_signal() -> Result<()> {
    let ctrl_c = async {
        signal::ctrl_c().await.context("Failed to install Ctrl+C handler")
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .context("Failed to install SIGTERM handler")?
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        () = terminate => {},
    }

    Ok(())
}
