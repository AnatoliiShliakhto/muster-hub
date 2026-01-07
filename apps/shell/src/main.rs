#![windows_subsystem = "windows"]

use mhub_logger::Logger;

#[mhub_runtime::main(high_performance)]
async fn main() -> Result<()> {
    let _logger = Logger::builder()
        .with_app_name(env!("CARGO_PKG_NAME"))
        .with_stdout(true)
        .init()?;

    run_server();

    Ok(())
}

fn run_server() {
    std::thread::spawn(move || todo!("run server"));
}
