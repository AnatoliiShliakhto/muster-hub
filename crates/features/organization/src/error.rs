/// Error types related to organization management, multi-tenancy, and group structures.
#[mhub_error::error]
pub enum OrganizationError {
    /// An unexpected failure occurred within the organization management subsystem.
    #[error(
        message = "Internal organization service failure",
        code = 500,
        kind = "ORG_INTERNAL_ERROR"
    )]
    Internal,
}
