use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub db: DatabaseConfig,
    pub paths: PathsConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub namespace: String,
    pub database: String,
}

#[derive(Debug, Clone)]
pub struct PathsConfig {
    pub work: PathBuf,
    pub public: PathBuf,
}