use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::Arc;

/// Top-level API configuration shared across services.
#[derive(Default, Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ApiConfigInner {
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
}

/// Thin Arc-wrapped config for inexpensive cloning into subsystems.
#[derive(Default, Debug, Clone, Deserialize)]
pub struct ApiConfig {
    #[serde(flatten, default)]
    inner: Arc<ApiConfigInner>,
}

impl Deref for ApiConfig {
    type Target = ApiConfigInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ApiConfig {
    fn deref_mut(&mut self) -> &mut ApiConfigInner {
        Arc::make_mut(&mut self.inner)
    }
}

/// HTTP server configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub address: IpAddr,
    pub port: u16,
    pub ssl: Option<SslConfig>,
}

/// TLS certificate/key paths.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SslConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
}

/// `SurrealDB` connection configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    pub url: String,
    pub namespace: String,
    pub database: String,
    pub credentials: Option<DatabaseCredentials>,
}

/// `SurrealDB` root credentials (optional when using unauthenticated engines like mem://).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DatabaseCredentials {
    pub username: String,
    pub password: String,
}

/// Storage roots (data and static assets).
#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub static_dir: PathBuf,
}

/// Optional API security knobs.
#[derive(Default, Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SecurityConfig {
    pub identity: IdentityConfig,
}

/// Identity/JWT/DPoP security configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct IdentityConfig {
    pub session_cache_capacity: u64,
    pub jwt: JwtConfig,
    pub dpop: DpopConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: String,
    pub audience: Option<String>,
    pub ttl_seconds: u64,
    pub clock_skew_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DpopConfig {
    pub required: bool,
    pub nonce_required: bool,
    pub nonce_ttl_seconds: u64,
    pub nonce_max_retries: u8,
    pub nonce_cache_capacity: u64,
    pub replay_ttl_seconds: u64,
    pub replay_cache_capacity: u64,
    pub clock_skew_seconds: u64,
}

// --- Default ---

impl Default for ServerConfig {
    fn default() -> Self {
        Self { address: IpAddr::V4(Ipv4Addr::UNSPECIFIED), port: 4583, ssl: None }
    }
}

impl Default for SslConfig {
    fn default() -> Self {
        Self { cert: PathBuf::from("cert.pem"), key: PathBuf::from("key.pem") }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "mem://".to_owned(),
            namespace: "mhub".to_owned(),
            database: "core".to_owned(),
            credentials: Some(DatabaseCredentials::default()),
        }
    }
}

impl Default for DatabaseCredentials {
    fn default() -> Self {
        Self { username: "root".to_owned(), password: "root".to_owned() }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self { data_dir: PathBuf::from("."), static_dir: PathBuf::from("public") }
    }
}

impl Default for IdentityConfig {
    fn default() -> Self {
        Self {
            session_cache_capacity: 10_000,
            jwt: JwtConfig::default(),
            dpop: DpopConfig::default(),
        }
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "dev-only-change-me".to_owned(),
            issuer: "mhub".to_owned(),
            audience: None,
            ttl_seconds: 3600,
            clock_skew_seconds: 60,
        }
    }
}

impl Default for DpopConfig {
    fn default() -> Self {
        Self {
            required: true,
            nonce_required: true,
            nonce_ttl_seconds: 300,
            nonce_max_retries: 3,
            nonce_cache_capacity: 10_000,
            replay_ttl_seconds: 300,
            replay_cache_capacity: 50_000,
            clock_skew_seconds: 60,
        }
    }
}
