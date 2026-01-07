use config::{Config, Environment, File};
use serde::de::DeserializeOwned;
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use tracing::info;

/// Custom error type for config loading.
#[mhub_derive::mhub_error]
pub enum ConfigError {
    #[error("Config error{}: {source}", format_context(.context))]
    Config { source: config::ConfigError, context: Option<Cow<'static, str>> },
}

/// A reusable configuration loader that combines file-based settings with environment overrides.
///
/// This function implements a layered configuration strategy:
/// 1. **Base File**: Loads settings from a file (e.g., `server.toml`). If no path is provided, it defaults to `"server"`.
/// 2. **Environment Overrides**: Overlays values from environment variables prefixed with `MHUB__`.
///    Nested structures are accessed using double underscores (e.g., `MHUB__DATABASE__URL` maps to `database.url`).
///
/// # Type Parameters
/// * `T`: The target configuration structure. Must implement [`serde::Deserialize`].
///
/// # Arguments
/// * `path`: An optional file path to the configuration source. Defaults to the `server` file in the current working directory.
///
/// # Returns
/// * `Ok(T)`: The successfully populated configuration object.
/// * `Err(ConfigError)`: If the file is missing, the environment variables are malformed, or deserialization fails.
///
/// # Errors
/// This function will return an error if:
/// * The specified (or default) configuration file cannot be found.
/// * The content of the file does not match the structure of type `T`.
///
/// # Example
/// ```rust
/// use mhub_kernel::config::load_config;
///
/// #[derive(Default, serde::Deserialize)]
/// struct AppConfig {
///     port: u16,
/// }
///
/// let cfg: AppConfig = load_config(Some("config/local")).unwrap_or_default();
/// ```
pub fn load_config<T>(path: Option<impl AsRef<Path>>) -> Result<T, ConfigError>
where
    T: DeserializeOwned,
{
    let effective_path = path.map_or_else(|| PathBuf::from("server"), |p| p.as_ref().to_path_buf());

    let builder = Config::builder()
        .add_source(File::from(effective_path.as_path()).required(true))  // Now required since we checked existence
        .add_source(
            Environment::with_prefix("MHUB")
                .separator("__")
                .convert_case(config::Case::Snake),  // Env var overrides (e.g., MHUB__API__KEY)
        );

    info!("Loading config from {}", effective_path.display());

    let config = builder
        .build()
        .context("Failed to build config")?
        .try_deserialize::<T>()
        .context("Failed to deserialize config")?;

    Ok(config)
}
