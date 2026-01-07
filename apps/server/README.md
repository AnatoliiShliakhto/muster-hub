# mhub-server 🌐

MusterHub HTTP server built on Axum, SurrealDB, and the shared kernel/event bus. Provides the runtime entrypoint and wiring for feature slices.

## Quick start
```sh
cargo run -p mhub-server
```

## Configuration
- Loads config via `mhub_kernel::config::load_config(Some("server"))`.
- Supports env overrides: `MHUB__SERVER__PORT`, `MHUB__DATABASE__URL`, etc.
- SSL: set `server.ssl.cert` and `server.ssl.key` paths; warns on permissive key perms (Unix).

## Features
- `profiling`: enables `dhat` allocator profiling and `mhub-logger/tokio-console`.

## Entry point
```rust
#[mhub_runtime::main(high_performance)]
async fn main() -> anyhow::Result<()> {
    let _log = Logger::builder().name("mhub-server").init()?;
    let cfg = load_config(Some("server"))?;
    Server::builder().config(cfg).build().await?.run().await
}
```

## Server builder
- Validates SSL paths.
- Connects SurrealDB via `mhub-database`.
- Initializes feature slices via `mhub::init` with a shared EventBus.
- Builds `ApiState` with slices and starts Axum router.

## Shutdown
- Graceful shutdown on Ctrl+C / SIGTERM; 30s timeout.

## Testing
- Add end-to-end integration tests for router/state as features mature.
