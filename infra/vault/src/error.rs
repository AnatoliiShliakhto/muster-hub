/// A specialized error enum for cryptographic, serialization, and configuration failures
/// within the Vault infrastructure.
#[mhub_error::error]
pub enum VaultError {
    /// Failure during the encryption process, typically involving AEAD primitives.
    #[error(message = "Cryptographic encryption failure", kind = "VAULT_ENCRYPTION_ERROR")]
    Encryption,

    /// Failure during decryption, often indicating an incorrect key,
    /// mismatched AAD, or tampered ciphertext.
    #[error(
        message = "Cryptographic decryption failure",
        code = 401,
        kind = "VAULT_DECRYPTION_ERROR"
    )]
    Decryption,

    /// Failure during binary serialization or deserialization (Postcard).
    #[error(
        message = "Binary serialization failure", 
        source = postcard::Error,
        kind = "VAULT_SERIALIZATION_ERROR"
    )]
    Serialization,

    /// Failure during data decompression, indicating corrupted or malformed blocks.
    #[error(message = "Data decompression failure", kind = "VAULT_DECOMPRESSION_ERROR")]
    Decompression,

    /// The vault or builder was initialized with invalid or missing parameters.
    #[error(
        message = "Invalid vault configuration",
        code = 400,
        kind = "VAULT_CONFIGURATION_ERROR"
    )]
    InvalidConfiguration,

    /// The provided payload is malformed, truncated, or lacks required headers.
    #[error(message = "Invalid or malformed payload", code = 400, kind = "VAULT_PAYLOAD_ERROR")]
    InvalidPayload,

    /// An unexpected logic error occurred within the vault subsystem.
    #[error(message = "Internal vault subsystem failure", kind = "VAULT_INTERNAL_ERROR")]
    Internal,
}
