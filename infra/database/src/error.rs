use std::borrow::Cow;

/// A specialized [`DatabaseError`] enum of this crate.
#[mhub_derive::mhub_error]
pub enum DatabaseError {
    /// Validation errors.
    #[error("Validation error{}: {message}", format_context(.context))]
    Validation { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    /// Occurs when connectivity or health checks fail.
    #[error("Database connection failed{}: {message}", format_context(.context))]
    Connection { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    /// Occurs when authentication fails.
    #[error("Authentication failed{}: {message}", format_context(.context))]
    Auth { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    /// A wrapper for underlying `SurrealDB` engine errors.
    #[error("SurrealDB error{}: {source}", format_context(.context))]
    Surreal {
        #[source]
        source: surrealdb::Error,
        context: Option<Cow<'static, str>>,
    },

    /// Migration failures or invariant violations.
    #[error("Migration error{}: {message}", format_context(.context))]
    Migration { message: Cow<'static, str>, context: Option<Cow<'static, str>> },

    /// Internal fallback for unexpected issues or logic errors.
    #[error("Internal database error{}: {message}", format_context(.context))]
    Internal { message: Cow<'static, str>, context: Option<Cow<'static, str>> },
}
