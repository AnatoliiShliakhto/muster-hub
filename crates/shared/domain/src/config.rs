use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Default, Debug, Clone, Deserialize)]
pub struct ApiConfigInner {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
}

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

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub address: IpAddr,
    pub port: u16,
    pub ssl: Option<SslConfig>,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SslConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub namespace: String,
    pub database: String,
    pub credentials: Option<DatabaseCredentials>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub static_dir: PathBuf,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    pub api_key: Option<String>,
}

// --- Default ---

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port: 4583,
            ssl: None,
            security: SecurityConfig::default(),
        }
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
