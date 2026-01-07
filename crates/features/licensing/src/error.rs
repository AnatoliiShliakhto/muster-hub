use std::borrow::Cow;

/// Error types specific to the licensing feature.
#[mhub_derive::mhub_error]
pub enum LicenseError {
    #[error("License has expired{}: {message}", format_context(.context))]
    Expired { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    /// Postcard serialization error with optional context.
    #[error("License signature is invalid or tampered{}: {source}", format_context(.context))]
    InvalidSignature { source: ed25519_dalek::SignatureError, context: Option<Cow<'static, str>> },

    #[error("MachineID mismatch{}: {message}", format_context(.context))]
    HardwareMismatch { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    /// Machine ID generation failed with optional context.
    #[error("Machine ID generation failed{}: {message}", format_context(.context))]
    MachineIDGeneration { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    /// Serde serialization error with optional context.
    #[error("Serde serialization error{}: {source}", format_context(.context))]
    SerdeSerialize { source: serde_json::Error, context: Option<Cow<'static, str>> },

    /// Postcard serialization error with optional context.
    #[error("Postcard serialization error{}: {source}", format_context(.context))]
    PostcardSerialize { source: postcard::Error, context: Option<Cow<'static, str>> },

    /// Internal fallback for unexpected issues or logic errors.
    #[error("Internal licensing error{}: {message}", format_context(.context))]
    Internal { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
}
