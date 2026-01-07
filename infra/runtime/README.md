# mhub-runtime ⚡

Specialized orchestration layer for the Tokio async runtime with opinionated profiles and a
proc-macro for `async fn main`.

## Features

- **Profiles:** High-performance (server) and memory-efficient (client) presets.
- **Customization:** Configure worker threads, stack size, thread names, and keep-alive.
- **Proc-macro:** `#[mhub_runtime::main(<profile>)]` wraps `async fn main` with the chosen runtime.
- **Global runtime:** Lazily initialized singleton for components that need a runtime outside async
  contexts.

## Quick start

```rust
#[mhub_runtime::main(high_performance)]
async fn main() -> anyhow::Result<()> {
    println!("Running on a high-performance runtime");
    Ok(())
}
```

The macro requires `async fn main() -> Result<...>`. Supported profiles are:
`high_performance`, `memory_efficient`, and `default`.

## Building runtimes manually

```rust
use mhub_runtime::{build_runtime_with_config, RuntimeConfig};

fn main() -> anyhow::Result<()> {
    let config = RuntimeConfig::default()
        .with_worker_threads(8)
        .with_stack_size(4 * 1024 * 1024)
        .with_thread_name("custom-worker")
        .with_thread_keep_alive(std::time::Duration::from_secs(120));

    let rt = build_runtime_with_config(&config)?;
    rt.block_on(async {
        Ok::<_, anyhow::Error>(())
    })?;

    Ok(())
}
```

## Global runtime

```rust
fn main() -> anyhow::Result<()> {
    let rt = mhub_runtime::get_global_runtime();
    rt.block_on(async {
        // run async tasks without creating a new runtime
    });

    Ok(())
}
```

## Environment knobs

- `TOKIO_WORKER_THREADS`: override detected worker threads (1..1024).

## Safety notes

- Stack size is clamped to 1–16 MiB to avoid OS issues.
- Global runtime panics on initialization failure (considered fatal).
