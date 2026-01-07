use mhub_database::*;

#[tokio::test]
async fn connect_in_memory_and_health_check() {
    let db = Database::builder()
        .url("mem://")
        .session("test_ns", "test_db")
        .init()
        .await
        .expect("connect to mem://");

    // Health should be OK for mem://
    db.health().await.expect("health check");
    db.use_ns("test_ns").use_db("test_db").await.expect("session switch");
}

#[tokio::test]
async fn missing_parameters_fail_validation() {
    let err = Database::builder().init().await.unwrap_err();
    assert!(matches!(err, DatabaseError::Validation { .. }));
}
