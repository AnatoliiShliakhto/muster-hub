use thiserror::Error;

#[derive(Debug, Error)]
pub enum LicenseError {
    #[error("License has expired")]
    Expired,
    
    #[error("License signature is invalid or tampered: {0}")]
    InvalidSignature(#[from] ed25519_dalek::SignatureError),
    
    #[error("License is not valid for this hardware")]
    HardwareMismatch,

    #[error("Machine ID generation failed: {0}")]
    MachineIDGenerationFailed(String),

    #[error("Serialization error: {0}")]
    Serialize(#[from] serde_json::Error),
    
    #[error("Internal error during validation: {0}")]
    Internal(String),
}