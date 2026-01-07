use anyhow::Context;
use mhub::kernel::config::load_config;
use mhub_logger::Logger;
use mhub_server::Server;

#[mhub_runtime::main(high_performance)]
async fn main() -> Result<()> {
    let _log = Logger::builder().with_app_name(env!("CARGO_PKG_NAME")).with_stdout(true).init()?;

    let cfg = load_config(Some("server")).context("Critical: Configuration is malformed")?;

    Server::builder().with_config(cfg).build().await?.run().await
}
