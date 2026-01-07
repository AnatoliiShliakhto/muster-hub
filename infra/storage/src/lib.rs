//! A high-performance, sandboxed storage engine.
//! It provides a secure abstraction over the filesystem with built-in protections against common
//! I/O pitfalls and security vulnerabilities. All examples use temporary directories to avoid
//! writing to the real filesystem.
//!
//! # Core Features
//!
//! - **Sandbox Security**: Strict path traversal protection using physical path canonicalization.
//! - **Atomic Writes**: Uses an "atomic swap" pattern (unique temp write + `fsync` + `rename`) to prevent data corruption during crashes.
//! - **Transparent Compression**: Integrated LZ4 block compression that is invisible to the consumer.
//! - **Namespacing & Sharding**: Logical data partitioning with automatic directory sharding to maintain filesystem performance.
//! - **Self-Healing**: Automatically identifies and cleans up orphaned temporary files during initialization.
//!
//! # Architectural Overview
//!
//! The crate follows a layered approach:
//! 1.  **[`Storage`]**: The primary thread-safe handle and entry point.
//! 2.  **[`NamespacedStorage`]**: A scoped view for multi-tenant or grouped data.
//! 3.  **[`StorageBuilder`]**: A type-safe fluent builder for configuration.
//!
//! # Examples
//!
//! ```rust
//! use mhub_storage::{Storage, Compression, StorageError};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), StorageError> {
//!     // Use a temp directory for examples/tests
//!     # let tmp = tempfile::tempdir().unwrap();
//!     # let root = tmp.path().join("data");
//!     let storage = Storage::builder()
//!         .root(&root)
//!         .create(true)
//!         .compression(Compression::Lz4)
//!         .connect()
//!         .await?;
//!
//!     // Write data atomically
//!     storage.write("config.bin", b"important data").await?;
//!
//!     // Read data (automatically decompressed)
//!     let data = storage.read("config.bin").await?;
//!     assert_eq!(data, b"important data");
//!
//!     Ok(())
//! }
//! ```
//!
//! ```rust
//! # use mhub_storage::{Storage, StorageError};
//! # async fn run(storage: Storage) -> Result<(), StorageError> {
//! # let tmp = tempfile::tempdir().unwrap();
//! # let root = tmp.path().join("data");
//! # let storage = Storage::builder().root(&root).connect().await?;
//! let user_id = "user_12345";
//! let user_storage = storage.namespace(user_id)?;
//!
//! // Files are stored in a sharded path: <root>/user_12345/avatars/av/at/avatar.png
//! user_storage.write("avatars/avatar.png", b"content").await?;
//!
//! if user_storage.exists("avatar.png")? {
//!     let meta = user_storage.metadata("avatar.png").await?;
//!     println!("Size on disk: {} bytes", meta.len());
//! }
//! # Ok(())
//! # }
//! ```

mod builder;
mod engine;
mod error;
mod maintenance;
mod namespace;
mod security;

pub use builder::StorageBuilder;
pub use engine::{Compression, Storage};
pub use error::{StorageError, StorageErrorExt};
pub use namespace::NamespacedStorage;
