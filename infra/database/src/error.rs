/// A specialized error enum for database operations, supporting `SurrealDB`
/// and standard repository lifecycle failures.
#[mhub_error::error]
pub enum DatabaseError {
    /// Data validation failed at the database or repository level.
    #[error(message = "Data validation failed", code = 400, kind = "DB_VALIDATION_ERROR")]
    Validation,

    /// Connectivity issues, connection pool exhaustion, or health check failures.
    #[error(
        message = "Database connectivity issue",
        code = 502,
        kind = "DB_CONNECTION_ERROR",
        help = "Check your database connection settings."
    )]
    Connection,

    /// Credentials rejected or insufficient permissions for the operation.
    #[error(
        message = "Database access denied",
        code = 401,
        kind = "DB_AUTH_ERROR",
        help = "Check your credentials."
    )]
    Auth,

    /// Migration failures, schema version mismatches, or invariant violations.
    #[error(message = "Schema migration failure", code = 500, kind = "DB_MIGRATION_ERROR")]
    Migration,

    /// A wrapper for underlying `SurrealDB` engine errors.
    #[cfg(not(target_arch = "wasm32"))]
    #[error(
        message = "Database engine failure",
        source = surrealdb::Error,
        kind = "DB_SURREAL_ENGINE_ERROR"
    )]
    Surreal,

    /// A wrapper for underlying jsonwebtoken errors.
    #[error(
        message = "Security token verification failed",
        source = jsonwebtoken::errors::Error,
        kind = "DB_AUTH_TOKEN_ERROR"
    )]
    Token,

    /// Database internal error, likely due to unexpected conditions or bugs.
    #[error(message = "Internal database wrapper failure", kind = "DB_INTERNAL_ERROR")]
    Internal,
}

use std::sync::Arc;
impl From<Arc<Self>> for DatabaseError {
    fn from(error: Arc<Self>) -> Self {
        Arc::try_unwrap(error).unwrap_or_else(|e| {
            Self::internal()
                .with_message("Couldn't unwrap shared error")
                .with_context_fn(|| format!("original error: {}", e.detailed()))
        })
    }
}
