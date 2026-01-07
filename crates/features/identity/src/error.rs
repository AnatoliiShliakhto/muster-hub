/// Error types specific to user identity management and account operations.
#[mhub_error::error]
pub enum IdentityError {
    /// An unexpected failure occurred within the user identity subsystem.
    #[error(
        message = "Internal identity management failure",
        code = 500,
        kind = "IDENTITY_INTERNAL_ERROR"
    )]
    Internal,
}
