use mhub_domain::config::{ApiConfig, DatabaseConfig, ServerConfig, StorageConfig};
use serde_json::json;

#[test]
fn config_defaults_are_sane() {
    let server = ServerConfig::default();
    assert_eq!(server.port, 4583);
    assert!(server.ssl.is_none());

    let db = DatabaseConfig::default();
    assert_eq!(db.url, "mem://");
    assert_eq!(db.namespace, "mhub");
    assert_eq!(db.database, "core");
    assert!(db.credentials.is_some());

    let storage = StorageConfig::default();
    assert_eq!(storage.static_dir, std::path::PathBuf::from("public"));
}

#[test]
fn api_config_deserializes() {
    let raw = json!({
        "server": { "address": "::", "port": 8080, "security": {} },
        "database": { "url": "mem://", "namespace": "n", "database": "d", "credentials": null },
        "storage": { "data_dir": "/tmp/data", "static_dir": "/tmp/static" }
    });

    let cfg: ApiConfig = serde_json::from_value(raw).expect("config deserialize");
    assert_eq!(cfg.server.port, 8080);
    assert_eq!(cfg.database.namespace, "n");
    assert_eq!(cfg.storage.static_dir, std::path::PathBuf::from("/tmp/static"));
}
