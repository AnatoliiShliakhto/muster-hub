use crate::error::Result;

/// Utilities for safe resource handling and ID validation.
pub struct ResourceGuard;

impl ResourceGuard {
    /// Validates a `SurrealDB` ID string against a specific table.
    ///
    /// Prevents "ID Spoofing" where a user provides an ID from a different table
    /// (e.g., providing a 'system:config' ID to a 'user' endpoint).
    ///
    /// # Arguments
    /// * `id` - The ID to verify (e.g., "user:123" or just "123")
    /// * `expected_table` - The table the ID must belong to (e.g., "user")
    pub fn verify<I, T>(id: I, expected_table: T) -> Result<String>
    where
        I: AsRef<str>,
        T: AsRef<str>,
    {
        let id_ref = id.as_ref();
        let table_ref = expected_table.as_ref();

        if let Some((table, _)) = id_ref.split_once(':') {
            if table != table_ref {
                Err(format!(
                    "Security violation: ID table mismatch. Expected '{table_ref}', got '{table}'"
                ))?;
            }
            // Return the full validated ID
            Ok(id_ref.to_owned())
        } else {
            // Automatically prefix if only the random part was provided
            Ok(format!("{table_ref}:{id_ref}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_verification() {
        // Correct table
        assert_eq!(
            ResourceGuard::verify("user:123", "user").unwrap(),
            "user:123"
        );

        // Auto-prefix
        assert_eq!(ResourceGuard::verify("123", "user").unwrap(), "user:123");

        // Malicious mismatch
        let err = ResourceGuard::verify("system:config", "user");
        assert!(err.is_err());
    }
}
