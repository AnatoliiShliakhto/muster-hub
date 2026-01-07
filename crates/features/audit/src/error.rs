/// Error types specific to the audit logging and compliance tracking slice.
#[mhub_error::error]
pub enum AuditError {
    /// An unexpected failure occurred within the audit subsystem,
    /// potentially resulting in a loss of telemetry or activity logs.
    #[error(
        message = "Internal audit subsystem failure",
        code = 500,
        kind = "AUDIT_INTERNAL_ERROR"
    )]
    Internal,
}
