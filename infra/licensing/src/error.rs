/// Error types specific to the licensing feature.
#[mhub_error::error]
pub enum LicenseError {
    /// The license is valid, but the allotted usage period has lapsed.
    #[error(message = "License has expired", code = 403, kind = "LICENSE_EXPIRED")]
    Expired,

    /// Cryptographic verification failed: the signature is invalid or tampered with.
    #[error(
        message = "Invalid license signature",
        source = ed25519_dalek::SignatureError,
        code = 401,
        kind = "LICENSE_INVALID_SIGNATURE"
    )]
    InvalidSignature,

    /// The license is valid but is registered to a different hardware fingerprint.
    #[error(
        message = "Hardware machine ID mismatch",
        code = 403,
        kind = "LICENSE_HARDWARE_MISMATCH"
    )]
    HardwareMismatch,

    /// Failed to retrieve or generate the local hardware identification string.
    #[error(message = "Hardware identity generation failed", kind = "LICENSE_ID_GEN_ERROR")]
    MachineIDGeneration,

    /// An error occurred during JSON metadata processing or serialization.
    #[error(
        message = "License metadata processing failure",
        source = serde_json::Error,
        kind = "LICENSE_JSON_ERROR"
    )]
    SerdeSerialize,

    /// Failure during compact binary format (Postcard) handling.
    #[error(
        message = "Binary license format failure",
        source = postcard::Error,
        kind = "LICENSE_POSTCARD_ERROR"
    )]
    PostcardSerialize,

    /// Failure during the creation or signing process of a new license.
    #[error(message = "License generation failure", kind = "LICENSE_GEN_ERROR")]
    Generation,

    /// Unexpected internal logic error within the licensing subsystem.
    #[error(message = "Internal licensing subsystem failure", kind = "LICENSE_INTERNAL_ERROR")]
    Internal,
}
