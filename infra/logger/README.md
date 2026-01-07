# mhub-logger 📝

Structured logging built on `tracing`/`tracing-subscriber` with console and rolling file support,
optional JSON output, tokio-console integration, and optional OpenTelemetry tracing.

## Features

- Console logging (ANSI) and rolling file appender with a non-blocking writer.
- JSON or compact text formats.
- Configurable rotation, max files, and log level.
- Env filters via `RUST_LOG` and programmatic `env_filter`.
- Optional `profiling` feature for tokio-console (requires `RUSTFLAGS="--cfg tokio_unstable"`).
- Optional `opentelemetry` feature for OpenTelemetry tracing.
- Optional `opentelemetry-otlp` helper for configuring an OTLP tracer provider.

## Quick start

```rust
use mhub_logger::{LevelFilter, Logger, Rotation};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _logger = Logger::builder()
        .name("my-app")
        .console(true)
        .path("./logs")
        .rotation(Rotation::DAILY)
        .max_files(7)
        .level(LevelFilter::INFO)
        .env_filter("myapp=debug,hyper=info")
        .json()
        .init()?;

    Ok(())
}
```

## tokio-console

- Enable crate feature `profiling`.
- Compile with `RUSTFLAGS="--cfg tokio_unstable"` (compile-time enforced).

## OpenTelemetry

- Enable crate feature `opentelemetry`.
- Install a tracer provider (for example, via `opentelemetry-otlp`) before calling `init()`.
- The logger attaches a `tracing-opentelemetry` layer that uses the global tracer.

```rust
use mhub_logger::{LevelFilter, Logger};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _logger = Logger::builder()
        .name("my-app")
        .level(LevelFilter::INFO)
        .opentelemetry(true)
        .init()?;

    Ok(())
}
```

## OpenTelemetry OTLP helper

```rust
use mhub_logger::{init_otlp_tracer, LevelFilter, Logger};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _otel = init_otlp_tracer("my-app")?;

    let _logger = Logger::builder()
        .name("my-app")
        .level(LevelFilter::INFO)
        .opentelemetry(true)
        .init()?;

    Ok(())
}
```

## Tips

- Keep the returned `Logger` alive to flush non-blocking writers; call `flush()` before shutdown.

## Testing

- Logger tests cover builder config, file creation, env filters, and init-if-absent behavior.
