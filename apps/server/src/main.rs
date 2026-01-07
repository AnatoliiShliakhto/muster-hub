use mhub::kernel::config::load_config;
use mhub_logger::Logger;
use mhub_server::Server;
use mhub_server::error::ServerError;

#[mhub_runtime::main(high_performance)]
async fn main() -> Result<(), ServerError> {
    let _log = Logger::builder().name(env!("CARGO_PKG_NAME")).init().map_err(ServerError::from)?;

    let cfg = load_config(Some("server")).map_err(ServerError::from)?;

    Server::builder().config(cfg).build().await?.run().await?;

    Ok(())
}
