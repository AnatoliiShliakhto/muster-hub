use mhub_server::{Server, SslConfig};

#[mhub_runtime::main(high_performance)]
async fn main() -> Result<()> {
    let _logger = mhub_logger::Logger::builder(env!("CARGO_PKG_NAME"))
        .with_stdout(true)
        .init()?;

    let mut server = Server::builder();
    
    server = server.with_db("mem://", "mhub", "core");
    
    server.build().await?.run().await
}
