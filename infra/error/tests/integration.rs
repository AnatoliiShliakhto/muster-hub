use mhub_error::error;
use std::io;

#[error]
pub enum TestError {
    #[error(message = "Unauthorized access", code = 401, kind = "AUTH_FAIL")]
    Auth,
    #[error(message = "IO failure", source = io::Error, code = 500)]
    Io,
    #[error(code = 404)]
    NotFound,
}

#[test]
fn test_fluent_api_chaining() {
    let err = TestError::auth()
        .with_context("Session: abc-123")
        .with_help("Try logging in again")
        .with_code(ErrorCode::Forbidden);

    assert_eq!(err.code(), ErrorCode::Forbidden);
    assert_eq!(err.context().unwrap(), "Session: abc-123");
    assert_eq!(err.help().unwrap(), "Try logging in again");
}

#[test]
fn test_result_extension_laziness() {
    let mut counter = 0;
    let res: Result<(), TestError> = Err(TestError::not_found());

    let _ = res.with_context_fn(|| {
        counter += 1;
        "Lazy context".to_owned()
    });

    assert_eq!(counter, 1, "Closure should be called on Err");

    let ok_res: Result<(), TestError> = Ok(());
    let _ = ok_res.with_context_fn(|| {
        counter += 1;
        "Should not be called".to_owned()
    });

    assert_eq!(counter, 1, "Closure should NOT be called on Ok");
}

#[test]
fn test_system_io_parsing_in_detailed_report() {
    let internal_io = io::Error::new(io::ErrorKind::ConnectionRefused, "os error 10061");
    let wrapper_io = io::Error::other(format!("WebSocket: IO error: {internal_io}"));

    let err = TestError::from(wrapper_io).with_context("Initializing database");

    let report = format!("{}", err.detailed());

    assert!(report.contains("1: WebSocket: IO error: os error 10061"));
    assert!(report.contains("Context:"));
    assert!(report.contains("Initializing database"));
}

#[test]
fn test_source_chain_iterator() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
    let err = TestError::from(io_err);

    let chain: Vec<_> = err.chain().collect();
    assert_eq!(chain.len(), 2);
    assert!(chain[1].to_string().contains("file missing"));
}

#[cfg(feature = "json")]
#[test]
fn test_json_serialization() {
    let err = TestError::auth().with_context("User: admin").with_help("Check LDAP");

    let json = err.to_json_string().unwrap();

    assert!(json.contains("\"code\":401"));
    assert!(json.contains("\"kind\":\"AUTH_FAIL\""));
    assert!(json.contains("\"context\":\"User: admin\""));
}
