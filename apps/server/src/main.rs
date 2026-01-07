use anyhow::Context;
use mhub::kernel::config::load_config;
use mhub_logger::Logger;
use mhub_server::Server;

#[cfg(feature = "profiling")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[mhub_runtime::main(high_performance)]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "profiling")]
    let _profiler = dhat::Profiler::new_heap();

    let _log = Logger::builder().name(env!("CARGO_PKG_NAME")).init()?;

    let cfg = load_config(Some("server")).context("Critical: Configuration is malformed")?;

    Server::builder().config(cfg).build().await?.run().await
}
