#[mhub_error::error]
pub enum {{ shortname | pascal_case }}Error {
    /// An unexpected failure.
    #[error(message = "Internal failure", code = 500, kind = "{{ shortname | shouty_snake_case }}_INTERNAL_ERROR")]
    Internal,
}
