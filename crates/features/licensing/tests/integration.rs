use ed25519_dalek::{Signer, SigningKey};
use mhub_licensing::validator::validate_license;
use mhub_licensing::*;

fn keypair() -> (SigningKey, [u8; 32]) {
    let seed = [7u8; 32];
    let signing = SigningKey::from_bytes(&seed);
    let public: [u8; 32] = signing.verifying_key().to_bytes();
    (signing, public)
}

fn sample_license() -> LicenseData {
    LicenseData {
        customer: "test".into(),
        alias: "test-ns".into(),
        constraint: MachineConstraint::Any,
        features: mhub_domain::features::FeatureSet::all(),
        salt: vec![1, 2, 3],
        issued: 0,
        expires: i64::MAX,
    }
}

#[test]
fn signed_license_roundtrip_json_and_bin() {
    let (signing, public) = keypair();
    let data = sample_license();
    let signature = signing.sign(&postcard::to_stdvec(&data).unwrap()).to_bytes().to_vec();
    let signed = SignedLicense { data, signature };

    let json = signed.to_json().unwrap();
    let from_json = SignedLicense::from_json(&json).unwrap();
    assert_eq!(from_json.data.customer, "test");
    assert_eq!(from_json.data.alias, "test-ns");

    let bin = signed.encode_bin().unwrap();
    let from_bin = SignedLicense::decode_bin(&bin).unwrap();
    assert_eq!(from_bin.data.customer, "test");
    assert_eq!(from_bin.data.alias, "test-ns");

    validate_license(&from_bin, &public).unwrap();
}

#[test]
fn expired_license_is_rejected() {
    let (signing, public) = keypair();
    let mut data = sample_license();
    data.issued = 0;
    data.expires = 1;
    let signature = signing.sign(&postcard::to_stdvec(&data).unwrap()).to_bytes().to_vec();
    let signed = SignedLicense { data, signature };

    let err = validate_license(&signed, &public).unwrap_err();
    assert!(matches!(err, LicenseError::Expired { .. }));
}
