use mhub_identity::init;

#[cfg(feature = "server")]
#[test]
fn init_creates_slice() {
    let slice = init().expect("init should succeed");
    assert_eq!(slice.id, std::any::TypeId::of::<mhub_identity::Identity>());
}
