#![windows_subsystem = "windows"]

use mhub_logger::Logger;

#[mhub_runtime::main(high_performance)]
async fn main() -> anyhow::Result<()> {
    let _logger = Logger::builder().name(env!("CARGO_PKG_NAME")).console(true).init()?;

    run_server();

    Ok(())
}

fn run_server() {
    std::thread::spawn(move || {});
}
