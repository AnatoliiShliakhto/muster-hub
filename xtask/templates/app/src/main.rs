use {{ name | snake_case }}::error::{{ shortname | pascal_case }}Error;

#[mhub_runtime::main(default)]
async fn main() -> Result<(), {{ shortname | pascal_case }}Error> {
    let _log = mhub_logger::Logger::builder()
        .name(env!("CARGO_PKG_NAME"))
        .init()
        .map_err({{ shortname | pascal_case }}Error::from)?;

    {{ name | snake_case }}::run().await
}