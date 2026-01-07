# mhub-database 🗄️

SurrealDB infrastructure wrapper with a fluent builder, health checks, optional auth, and
Axum-friendly extraction via `FromRef` (through kernel state).

## Highlights

- Supports `mem://`, `rocksdb://`, `ws://`, `http://` via `any` engine.
- Validates URL/namespace/database up front; retries health check with backoff.
- Optional root auth via username/password.
- Deref to `Surreal<Any>` for direct query API use.

## Quick start

```rust
use mhub_database::{Database, DatabaseError};

#[tokio::main]
async fn main() -> Result<(), DatabaseError> {
    let db = Database::builder()
        .url("mem://")
        .session("mhub", "core")
        .connect()
        .await?;

    db.health().await?;
    Ok(())
}
```

## Features

- `rocksdb`: enable RocksDB kv backend for SurrealDB.

## Behavior

- Health check: up to three attempts with exponential backoff starting at 500 ms.
- Auth: call `.auth(user, pass)` to sign in as root before setting namespace/db.

## Testing

- Integration tests cover `mem://` connect/health/session and validation errors.

