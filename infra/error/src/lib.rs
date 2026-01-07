//! # mhub-error
//!
//! Centralized error handling system for the infrastructure.
//!
//! This crate provides a unified error model that combines:
//! - **ErrorCode-based Typing**: Seamless compatibility with HTTP status codes.
//! - **Contextual Wrapping**: The ability to attach high-level descriptions as errors bubbles up.
//! - **Cause Chains**: Tools for debugging and iterating through nested error sources.
//! - **Procedural Macros**: Automatic boilerplate generation via the `#[error]` attribute.
//!
//! ## Core Components
//!
//! 1. **`#[error]`**: An attribute macro used to define error enums (re-exported from `mhub_derive`).
//! 2. **`ErrorMetadata`**: The primary trait providing access to codes, slugs, and backtraces.
//! 3. **`ErrorCode`**: An enum defining error categories based on HTTP RFC standards.
//! 4. **`DetailedReport`**: A formatter for rendering hierarchical error cause trees.
//!
//! ## Example Usage
//!
//! ```rust
//! use mhub_error::error;
//!
//! #[error]
//! pub enum MyError {
//!     #[error(message = "Access denied", code = 403)]
//!     Forbidden,
//!
//!     #[error(message = "Database failure", source = std::io::Error)]
//!     Database,
//!
//!     Internal,
//! }
//!
//! fn run() -> Result<(), MyError> {
//!     // Using the .context() extension provided by the generated MyErrorExt trait
//!     std::fs::read_to_string("config.db")
//!         .with_context("Failed to read system configuration")?;
//!     Ok(())
//! }
//!
//! fn main() {
//!     if let Err(e) = run() {
//!         // Print the high-level error message
//!         eprintln!("{e}");
//!
//!         // Print the raw error message with backtrace and error source
//!         eprintln!("{e:?}");
//!
//!         // Print a pretty hierarchical report
//!         eprintln!("{}", e.detailed());
//!     }
//! }
//! ```

mod code;
pub mod interop;

#[cfg(feature = "json")]
use serde::Serialize;
use std::backtrace::Backtrace;
use std::borrow::Cow;
use std::error::Error;
use std::fmt;

pub use code::ErrorCode;
pub use mhub_derive::mhub_error as error;

// --- Error metadata ---

/// A trait providing extended metadata for an error.
///
/// This trait is automatically implemented by the `#[error]` macro. It allows
/// retrieval of metadata such as HTTP-compatible codes, machine-readable slugs (kind),
/// and additional context.
pub trait ErrorMetadata: Error {
    /// Returns the human-readable error message.
    fn message(&self) -> Cow<'static, str> {
        self.to_string().into()
    }

    /// Returns the additional context added via `.context()`, if any.
    fn context(&self) -> Option<&Cow<'static, str>> {
        None
    }

    /// Returns the associated [`ErrorCode`] for this error.
    fn code(&self) -> ErrorCode {
        ErrorCode::InternalServerError
    }

    /// Returns a machine-readable identifier (slug) for the error.
    /// Default: `SHOUTY_SNAKE_CASE` of the variant name.
    fn kind(&self) -> Cow<'static, str> {
        "UNKNOWN_ERROR".into()
    }

    /// Returns the name of the component or package where the error originated.
    fn target(&self) -> &'static str {
        "UNKNOWN_TARGET"
    }

    /// Returns the captured backtrace, if available.
    fn backtrace(&self) -> Option<&Backtrace> {
        None
    }

    /// Creates an iterator over the entire error cause chain (from current to root cause).
    fn chain(&self) -> ErrorChain<'_>
    where
        Self: Sized + 'static,
    {
        ErrorChain { current: Some(self) }
    }

    /// Wraps the error in a structure for rendering a detailed hierarchical report.
    fn detailed(&self) -> DetailedReport<'_, Self>
    where
        Self: Sized,
    {
        DetailedReport(self)
    }

    /// Returns the help added via `.help()`, if any.
    fn help(&self) -> Option<&Cow<'static, str>> {
        None
    }

    #[cfg(feature = "json")]
    fn to_serializable(&self) -> SerializableReport
    where
        Self: Sized,
    {
        SerializableReport::from_metadata(self)
    }

    /// Serializes the error and its full context into a JSON-formatted string.
    /// *Note:* This function is only available when the json feature is enabled.
    ///
    /// # Returns
    /// `Ok(String)`: A JSON string representing the serialized error metadata.
    ///
    /// # Errors
    /// This function can fail in the following scenarios:
    /// - Serialization Failure: If custom data attached to the error (via the `#[error]` macro or context) contains types that `serde_json` cannot process (e.g., certain non-standard types or recursive structures).
    /// - Resource Constraints: Extremely rare, but can occur if the system runs out of memory while allocating the final string.
    #[cfg(feature = "json")]
    fn to_json_string(&self) -> Result<String, serde_json::Error>
    where
        Self: Sized,
    {
        serde_json::to_string(&self.to_serializable())
    }
}

//impl ErrorMetadata for Box<dyn Error + Send + Sync + 'static> {}

// --- Error chain ---

/// An iterator over the chain of error sources (`source`).
///
/// Allows for programmatic traversal of all nested error levels.
#[derive(Debug)]
pub struct ErrorChain<'a> {
    current: Option<&'a (dyn Error + 'static)>,
}

impl<'a> Iterator for ErrorChain<'a> {
    type Item = &'a (dyn Error + 'static);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        self.current = current.source();
        Some(current)
    }
}

// --- Error reporting ---

/// A helper for pretty-printing nested error causes.
///
/// Implements `Display` to build a tree structure:
/// ```text
/// Top-level Error
///   └─> Intermediate Cause
///     └─> Root Cause (e.g., io error)
/// ```
#[derive(Debug)]
pub struct DetailedReport<'a, E: ?Sized>(pub &'a E);

impl<E> fmt::Display for DetailedReport<'_, E>
where
    E: ErrorMetadata + ?Sized,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err = &self.0;

        writeln!(f, "  × {}", err.message())?;

        let mut all_causes = Vec::new();
        let mut current_source = err.source();

        while let Some(cause) = current_source {
            let msg = cause.to_string();
            if !all_causes.contains(&msg) {
                all_causes.push(msg);
            }
            current_source = cause.source();
        }

        let has_chain = !all_causes.is_empty();
        let has_context = err.context().is_some();
        let has_help = err.help().is_some();

        if has_chain || has_context || has_help {
            writeln!(f, "  │")?;
        }

        if has_chain {
            let is_last_section = !has_context && !has_help;
            let section_prefix = if is_last_section { "  ╰─" } else { "  ├─" };
            writeln!(f, "{section_prefix} Caused by:")?;

            let pipe = if is_last_section { "    " } else { "  │ " };
            for (i, cause) in all_causes.iter().enumerate() {
                let mut lines = cause.lines();
                if let Some(first_line) = lines.next() {
                    writeln!(f, "{pipe} {}: {first_line}", i + 1)?;
                }
                for line in lines {
                    writeln!(f, "{pipe}    {line}")?;
                }
            }

            if !is_last_section {
                writeln!(f, "  │")?;
            }
        }

        if let Some(ctx) = err.context() {
            let is_last_section = !has_help;
            let section_prefix = if is_last_section { "  ╰─" } else { "  ├─" };
            writeln!(f, "{section_prefix} Context:")?;
            let pipe = if is_last_section { "    " } else { "  │ " };
            for line in ctx.lines() {
                writeln!(f, "{pipe} {line}")?;
            }
            if !is_last_section {
                writeln!(f, "  │")?;
            }
        }

        if let Some(help) = err.help() {
            writeln!(f, "  ╰─ Help: {help}")?;
        }

        Ok(())
    }
}

// --- Serialization ---

#[cfg(feature = "json")]
#[derive(Debug, Serialize)]
pub struct SerializableReport {
    message: String,
    code: u16,
    kind: Cow<'static, str>,
    target: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    chain: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    backtrace: Option<String>,
}

#[cfg(feature = "json")]
impl SerializableReport {
    pub fn from_metadata<E: ErrorMetadata + ?Sized>(err: &E) -> Self {
        let chain = err
            .source()
            .map(|s| {
                let mut v = Vec::new();
                let mut curr = Some(s);
                while let Some(e) = curr {
                    v.push(e.to_string());
                    curr = e.source();
                }
                v
            })
            .unwrap_or_default();

        Self {
            message: err.to_string(),
            code: err.code().as_u16(),
            kind: err.kind(),
            target: err.target(),
            context: err.context().map(ToString::to_string),
            chain,
            backtrace: err.backtrace().map(|b| format!("{b:?}")),
        }
    }
}
