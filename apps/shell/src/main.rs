#![windows_subsystem = "windows"]

use anyhow::Result;

#[mhub_runtime::main(high_performance)]
async fn main() -> Result<()> {
    let _logger = mhub_logger::Logger::builder(env!("CARGO_PKG_NAME"))
        .with_stdout(true)
        .init()?;

    run_server()?;

    Ok(())
}

fn run_server() -> Result<()> {
    std::thread::spawn(move || todo!("run server"));

    Ok(())
}
