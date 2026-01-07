use mhub_storage::*;
use tempfile::TempDir;

#[tokio::test]
async fn test_path_traversal_blocked() {
    let temp = TempDir::new().unwrap();

    let storage = Storage::builder().root(temp.path()).connect().await.unwrap();

    assert!(storage.resolve("../etc/passwd").is_err());
    assert!(storage.resolve("foo/../../bar").is_err());
}

#[tokio::test]
async fn test_write_read_roundtrip_uncompressed() {
    let temp = TempDir::new().unwrap();
    let storage = Storage::builder().root(temp.path()).connect().await.unwrap();

    let payload = b"hello world";
    storage.write("foo/bar.bin", payload).await.unwrap();
    assert!(storage.exists("foo/bar.bin").unwrap());

    let data = storage.read("foo/bar.bin").await.unwrap();
    assert_eq!(data, payload);

    let meta = storage.metadata("foo/bar.bin").await.unwrap();
    assert!(meta.len() > 0);
}

#[tokio::test]
async fn test_write_read_roundtrip_compressed() {
    let temp = TempDir::new().unwrap();
    let storage =
        Storage::builder().root(temp.path()).compression(Compression::Lz4).connect().await.unwrap();

    let payload = vec![1u8; 4096];
    storage.write("bin/data.dat", &payload).await.unwrap();

    let data = storage.read("bin/data.dat").await.unwrap();
    assert_eq!(data, payload);
}

#[tokio::test]
async fn test_namespace_isolation_and_sharding() {
    let temp = TempDir::new().unwrap();
    let storage = Storage::builder().root(temp.path()).connect().await.unwrap();

    let ns_a = storage.namespace("user_a").unwrap();
    let ns_b = storage.namespace("user_b").unwrap();

    ns_a.write("photo.png", b"a").await.unwrap();
    ns_b.write("photo.png", b"b").await.unwrap();

    let a_path = ns_a.resolve("photo.png").unwrap();
    let b_path = ns_b.resolve("photo.png").unwrap();
    assert_ne!(a_path, b_path, "sharded paths must differ across namespaces");

    assert_eq!(ns_a.read("photo.png").await.unwrap(), b"a");
    assert_eq!(ns_b.read("photo.png").await.unwrap(), b"b");
}

#[tokio::test]
async fn test_delete_and_exists() {
    let temp = TempDir::new().unwrap();
    let storage = Storage::builder().root(temp.path()).connect().await.unwrap();

    storage.write("tmp/file.txt", b"x").await.unwrap();
    assert!(storage.exists("tmp/file.txt").unwrap());

    storage.delete("tmp/file.txt").await.unwrap();
    assert!(!storage.exists("tmp/file.txt").unwrap());
}

#[tokio::test]
async fn test_subdir_preserved_in_sharding() {
    let temp = TempDir::new().unwrap();
    let storage = Storage::builder().root(temp.path()).connect().await.unwrap();

    let resolved = storage.resolve("foo/bar.bin").unwrap();
    let path_str = resolved.to_string_lossy();
    assert!(path_str.contains("foo"), "expected subdirectory to be preserved");
}

#[tokio::test]
async fn test_read_missing_returns_file_not_found() {
    let temp = TempDir::new().unwrap();
    let storage = Storage::builder().root(temp.path()).connect().await.unwrap();

    let err = storage.read("missing.bin").await.expect_err("expected error");
    match err {
        StorageError::FileNotFound { .. } => {},
        other => panic!("unexpected error: {other:?}"),
    }
}
