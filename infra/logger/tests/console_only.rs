use mhub_logger::{LevelFilter, Logger};

#[test]
fn init_console_only_has_no_guard() {
    let logger = Logger::builder()
        .name("integration-console-only")
        .console(true)
        .level(LevelFilter::INFO)
        .init()
        .expect("logger should initialize");

    assert!(logger.guard().is_none(), "console-only logger should not create a file guard");
}
