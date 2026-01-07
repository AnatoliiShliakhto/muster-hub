#[test]
fn mhub_error_ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/mhub_error_pass.rs");
    t.compile_fail("tests/ui/mhub_error_no_context.rs");
    t.compile_fail("tests/ui/mhub_error_bad_context_type.rs");
    t.compile_fail("tests/ui/mhub_error_tuple_variant.rs");
}
