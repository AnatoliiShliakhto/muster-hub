#![windows_subsystem = "windows"]

use mhub_server::error::ServerError;

#[mhub_runtime::main(high_performance)]
async fn main() -> Result<(), ServerError> {
    let _logger =
        mhub_logger::Logger::builder().name(env!("CARGO_PKG_NAME")).console(true).init()?;

    run_server();

    Ok(())
}

#[allow(dead_code)]
fn run_server() {
    std::thread::spawn(move || {});
}
