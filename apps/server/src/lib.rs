pub mod error;
mod router;

use axum_server::Handle;
use mhub::domain::config::ApiConfig;
use mhub::kernel::server::ApiState;
use mhub_database::Database;
use mhub_event_bus::EventBus;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::signal;
use tracing::{error, info};

use crate::error::{ServerError, ServerErrorExt};

#[derive(Debug, Default)]
pub struct ServerBuilder {
    cfg: ApiConfig,
}

impl ServerBuilder {
    #[must_use]
    pub fn config(mut self, cfg: ApiConfig) -> Self {
        self.cfg = cfg;
        self
    }

    #[must_use]
    pub fn port(mut self, port: u16) -> Self {
        self.cfg.server.port = port;
        self
    }

    async fn init_database(&self) -> Result<Database, ServerError> {
        let db_cfg = &self.cfg.database;
        let mut builder =
            Database::builder().url(&db_cfg.url).session(&db_cfg.namespace, &db_cfg.database);

        if let Some(creds) = &db_cfg.credentials {
            builder = builder.auth(&creds.username, &creds.password);
        }

        builder.init().await.map_err(ServerError::from)
    }

    fn validate_ssl_config(&self) -> Result<(), ServerError> {
        if let Some(ssl) = &self.cfg.server.ssl {
            if !ssl.cert.exists() {
                Err(ServerError::ssl_config().with_context_fn(|| {
                    format!("certificate missing at `{}`", ssl.cert.display())
                }))?;
            }
            if !ssl.key.exists() {
                Err(ServerError::ssl_config().with_context_fn(|| {
                    format!("private key missing at `{}`", ssl.key.display())
                }))?;
            }
        }
        Ok(())
    }

    /// Validates and initializes the server configuration, including SSL settings.
    ///
    /// # Result
    /// Successful initialization returns a `Server` handle that can be used to start the server.
    ///
    /// # Errors
    /// Returns a `ServerError` if SSL configuration is invalid or missing required files.
    pub async fn build(self) -> Result<Server, ServerError> {
        self.validate_ssl_config()?;
        let address = SocketAddr::new(self.cfg.server.address, self.cfg.server.port);

        info!(address = %address, "Initializing server");

        let db = self
            .init_database()
            .await
            .with_context_fn(|| format!("Url: {}\nNamespace: {}\nDatabase: {}", self.cfg.database.url, self.cfg.database.namespace, self.cfg.database.database))
            .with_help("Verify that configuration of SurrealDB is valid and accessible from the current environment.")?;
        let events = EventBus::new();

        // Wrap platform bootstrap in Bootstrap error
        let slices = mhub::init(&self.cfg, &db, &events)
            .map_err(|e| ServerError::bootstrap().with_context_fn(|| e.to_string()))?;

        let state = slices
            .into_iter()
            .fold(ApiState::builder().config(self.cfg).db(db).events(events), |builder, slice| {
                builder.register_slice(slice)
            })
            .build()
            .map_err(|_| {
                ServerError::bootstrap().with_context("failed to finalize API state registry")
            })?;

        Ok(Server { state })
    }
}

#[derive(Debug)]
pub struct Server {
    state: ApiState,
}

impl Server {
    #[must_use]
    pub fn builder() -> ServerBuilder {
        ServerBuilder::default()
    }

    /// Runs the server with the configured settings.
    ///
    /// # Errors
    /// Returns a [`ServerError`] if the server fails to start or if there's an issue with the shutdown signal.
    pub async fn run(self) -> Result<(), ServerError> {
        let cfg = self.state.config.clone();
        let address = SocketAddr::new(cfg.server.address, cfg.server.port);
        let app = router::init(self.state);
        let handle = Handle::<SocketAddr>::new();
        let shutdown_handle = handle.clone();

        tokio::spawn(async move {
            if let Err(e) = shutdown_signal().await {
                error!("Shutdown signal listener failed: {e:?}");
                return;
            }
            info!("Shutdown signal received, starting graceful shutdown...");
            shutdown_handle.graceful_shutdown(Some(Duration::from_secs(30)));
        });

        if let Some(ssl_config) = &cfg.server.ssl {
            let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(
                &ssl_config.cert,
                &ssl_config.key,
            )
            .await
            .map_err(|e| ServerError::ssl_config().with_context_fn(|| e.to_string()))?;

            axum_server::bind_rustls(address, tls_config)
                .handle(handle)
                .serve(app.into_make_service())
                .await
                .map_err(|e| ServerError::binding().with_context_fn(|| e.to_string()))?;
        } else {
            axum_server::bind(address)
                .handle(handle)
                .serve(app.into_make_service())
                .await
                .map_err(|e| ServerError::binding().with_context_fn(|| e.to_string()))?;
        }

        info!("Server shutdown complete");
        Ok(())
    }
}

async fn shutdown_signal() -> Result<(), ServerError> {
    let ctrl_c = signal::ctrl_c();

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .map_err(|e| {
                ServerError::shutdown()
                    .with_context_fn(|| format!("failed to initialize terminate signal: {e}"))
            })?
            .recv()
            .await;
        Ok(())
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<Result<(), ServerError>>();

    tokio::select! {
        _ = ctrl_c => info!("Ctrl+C received"),
        res = terminate => return res,
    }
    Ok(())
}
