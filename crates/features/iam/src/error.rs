/// Error types specific to the Identity and Access Management (IAM) subsystem.
#[mhub_error::error]
pub enum IamError {
    /// An unexpected failure occurred during identity processing, authentication, or authorization logic.
    #[error(message = "Internal identity service failure", code = 500, kind = "IAM_INTERNAL_ERROR")]
    Internal,
}
