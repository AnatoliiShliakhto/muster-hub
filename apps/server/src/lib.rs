//! # Muster Hub Server
//!
//! A production-ready web server built on `Axum`, `SurrealDB`, and a type-safe event bus.
//!
//! ## Example
//! ```no_run
//! use mhub_server::Server;
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     Server::builder()
//!         .port(4583)
//!         .build()
//!         .await?
//!         .run()
//!         .await
//! }
//! ```

mod router;

use anyhow::{Context, Result, anyhow};
use axum_server::Handle;
use mhub::domain::config::ApiConfig;
use mhub::kernel::server::ApiState;
use mhub_database::Database;
use mhub_event_bus::EventBus;
use std::net::SocketAddr;
use tokio::signal;
use tracing::{error, info};

/// A fluent builder for configuring and initializing the [`Server`].
#[must_use = "builders do nothing unless you call .build()"]
#[derive(Debug, Default)]
pub struct ServerBuilder {
    cfg: ApiConfig,
}

impl ServerBuilder {
    /// Set up the server's configuration.
    pub fn config(mut self, cfg: ApiConfig) -> Self {
        self.cfg = cfg;
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.cfg.server.port = port;
        self
    }

    async fn init_database(&self) -> Result<Database> {
        let db_cfg = &self.cfg.database;
        let mut builder =
            Database::builder().url(&db_cfg.url).session(&db_cfg.namespace, &db_cfg.database);

        if let Some(creds) = &db_cfg.credentials {
            builder = builder.auth(&creds.username, &creds.password);
        }

        builder.init().await.context("Failed to establish database connection")
    }

    fn validate_ssl_config(&self) -> Result<()> {
        if let Some(ssl) = &self.cfg.server.ssl {
            if !ssl.cert.exists() {
                anyhow::bail!("SSL certificate not found at: {}", ssl.cert.display());
            }
            if !ssl.key.exists() {
                anyhow::bail!("SSL key not found at: {}", ssl.key.display());
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let metadata = ssl.key.metadata()?;
                if metadata.permissions().mode() & 0o077 != 0 {
                    tracing::warn!(
                        "SECURITY: SSL Private Key {} has insecure permissions (should be 600)",
                        ssl.key.display()
                    );
                }
            }
        }
        Ok(())
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
    /// * Database connection fails (unreachable host, invalid credentials)
    /// * Working directory or public directory is invalid
    /// * SSL certificate/key files cannot be read
    ///
    /// # Examples
    /// ```no_run
    /// # use mhub_server::Server;
    /// # async fn example() -> anyhow::Result<()> {
    /// let server = Server::builder()
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn build(self) -> Result<Server> {
        // 1. Validate SSL Configuration
        self.validate_ssl_config()?;

        let address = SocketAddr::new(self.cfg.server.address, self.cfg.server.port);

        info!(
            address = %address,
            "Initializing server"
        );

        // 2. Initialize Database
        let db = self.init_database().await?;

        // 3. Orchestrate Feature Slices
        let events = EventBus::new();
        let slices = mhub::init(&self.cfg, &db, &events)
            .map_err(|e| anyhow!("Platform bootstrap failed: {e}"))?;

        // 4. Construct State using Functional Folding
        let state = slices
            .into_iter()
            .fold(ApiState::builder().config(self.cfg).db(db).events(events), |builder, slice| {
                builder.register_slice(slice)
            })
            .build()
            .context("Failed to finalize API state registry")?;
        Ok(Server { state })
    }
}

/// A fully initialized server instance ready to run.
///
/// This struct is returned by [`ServerBuilder::build`] and contains
/// all necessary runtime state.
#[must_use = "call .run().await to start the server"]
#[derive(Debug)]
pub struct Server {
    state: ApiState,
}

impl Server {
    /// Returns a new [`ServerBuilder`] to configure the server.
    ///
    /// This is the recommended way to initialize the server.
    ///
    /// # Examples
    /// ```no_run
    /// # use mhub_server::Server;
    /// # async fn example() -> anyhow::Result<()> {
    /// let server = Server::builder()
    ///     .port(4583)
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
    /// Returns an error if the server fails to bind to the configured address
    /// or if SSL/TLS setup fails.
    ///
    /// # Examples
    /// ```no_run
    /// # use mhub_server::Server;
    /// # async fn example() -> anyhow::Result<()> {
    /// Server::builder()
    ///     .build()
    ///     .await?
    ///     .run()
    ///     .await
    /// # }
    /// ```
    pub async fn run(self) -> Result<()> {
        let cfg = self.state.config.clone();
        let address = SocketAddr::new(cfg.server.address, cfg.server.port);

        info!(
            address = %address,
            ssl = cfg.server.ssl.is_some(),
            "Starting server"
        );

        let app = router::init(self.state);

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
            shutdown_handle.graceful_shutdown(Some(std::time::Duration::from_secs(30)));
        });

        // 3. Start Server (HTTP or HTTPS)
        if let Some(ssl_config) = &cfg.server.ssl {
            // HTTPS mode
            info!("Starting HTTPS server on https://{address}");

            let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(
                &ssl_config.cert,
                &ssl_config.key,
            )
            .await
            .context("Failed to load SSL/TLS certificates")?;

            axum_server::bind_rustls(address, tls_config)
                .handle(handle)
                .serve(app.into_make_service())
                .await
                .context("HTTPS server failed")?;
        } else {
            // HTTP mode
            info!("Starting HTTP server on http://{address}");

            axum_server::bind(address)
                .handle(handle)
                .serve(app.into_make_service())
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
/// * SIGINT (Ctrl+C)
/// * SIGTERM (sent by process managers like systemd)
async fn shutdown_signal() -> Result<()> {
    let ctrl_c = async { signal::ctrl_c().await.context("Failed to install Ctrl+C handler") };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .context("Failed to install SIGTERM handler")?
            .recv()
            .await;
        Ok::<_, anyhow::Error>(())
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<Result<()>>();

    tokio::select! {
        res = ctrl_c => {
            res.context("Ctrl+C signal received")?;
        },
        res = terminate => {
            res.context("SIGTERM signal received")?;
        },
    }

    Ok(())
}
