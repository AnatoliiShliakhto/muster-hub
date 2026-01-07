use mhub_kernel::security::resource::ResourceGuard;

#[test]
fn resource_guard_validates_and_prefixes() {
    assert_eq!(ResourceGuard::verify("user:123", "user").unwrap(), "user:123");

    assert_eq!(ResourceGuard::verify("123", "user").unwrap(), "user:123");

    assert!(ResourceGuard::verify("system:123", "user").is_err());
}
