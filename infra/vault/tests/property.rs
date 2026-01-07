use mhub_vault::prelude::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_arbitrary_bytes_across_domains(data in proptest::collection::vec(any::<u8>(), 0..2048)) {
        let vault = Vault::<ChaCha>::builder()
            .derived_keys("ikm", "salt", "machine-id")
            .unwrap()
            .build()
            .unwrap();

        let sealed_local = vault.seal_bytes::<Local>(&data, b"ctx").unwrap();
        let unsealed_local = vault.unseal_bytes::<Local>(&sealed_local, b"ctx").unwrap();
        prop_assert_eq!(&data, &unsealed_local);

        let sealed_fleet = vault.seal_bytes::<Fleet>(&data, b"ctx").unwrap();
        let unsealed_fleet = vault.unseal_bytes::<Fleet>(&sealed_fleet, b"ctx").unwrap();
        prop_assert_eq!(data, unsealed_fleet);
    }
}
