# mhub-storage 📦

Sandboxed storage engine with path traversal protection, atomic writes, optional LZ4 compression,
and namespace + sharding support.

## Highlights

- **Sandboxed paths:** Resolves/canonicalizes to prevent escape via `..` or symlinks.
- **Atomic writes:** Write to a unique temp file + `fsync` + `rename` to avoid corruption.
- **Namespaces + sharding:** Deterministic sharding for hot directories; per-namespace views, while
  preserving subdirectories in paths.
- **Transparent compression:** Optional LZ4 block compression.
- **Self-healing:** Cleans stale `.tmp` files on startup.

## Quick start

```rust
use mhub_storage::{Storage, Compression, StorageError};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), StorageError> {
    let root = PathBuf::from("./data");
    let storage = Storage::builder()
        .root(&root)
        .create(true)
        .compression(Compression::Lz4)
        .connect()
        .await?;

    storage.write("config.bin", b"important data").await?;
    let data = storage.read("config.bin").await?;

    assert_eq!(data, b"important data");
    Ok(())
}
```

## Namespaces

```rust
use mhub_storage::{Storage, StorageError};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), StorageError> {
    let root = PathBuf::from("data");
    let storage = Storage::builder().root(&root).connect().await?;
    let user = storage.namespace("user_123")?;

    user.write("avatars/avatar.png", b"bytes").await?;

    assert!(user.exists("avatar.png")?);

    let meta = user.metadata("avatar.png").await?;
    println!("size on disk: {}", meta.len());

    Ok(())
}
```

## Testing & benches

- Integration tests cover traversal blocking, round-trips (compressed/uncompressed), namespace
  isolation, delete/exists.
- Benchmarks (`cargo bench -p mhub-storage`) measure path resolution, compression, file I/O,
  namespaces, and atomic writes.

## Safety notes

- Always provide relative paths; absolute paths are rejected.
- When compression is on, metadata size reflects compressed bytes.
- Temp files use a `.mhubtmp.<id>` suffix and are pruned on startup.
- Use per-environment roots; examples/tests use temp dirs to avoid touching real FS.
