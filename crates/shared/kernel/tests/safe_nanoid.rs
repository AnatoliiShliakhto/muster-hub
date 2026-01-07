use mhub_kernel::SAFE_ALPHABET;
use mhub_kernel::safe_nanoid;

#[test]
fn generates_expected_length_and_charset() {
    let id = safe_nanoid!();
    assert_eq!(id.len(), 12);

    for ch in id.chars() {
        assert!(SAFE_ALPHABET.contains(&ch), "unexpected character in nanoid: {ch}");
    }
}

#[test]
fn custom_length() {
    let id = safe_nanoid!(20);
    assert_eq!(id.len(), 20);
}
