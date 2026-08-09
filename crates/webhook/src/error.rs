/// Everything that can go wrong building or validating a custom payload template.
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    /// A `Custom` webhook had no `payload_template`.
    #[error("custom webhook requires a payload template")]
    Missing,
    /// The template string was not valid JSON.
    #[error("template is not valid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    /// A `${...}` placeholder referenced a variable that doesn't exist.
    #[error("unknown template variable `${{{0}}}`")]
    UnknownVar(String),
    /// An array/object variable was interpolated into the middle of a string.
    #[error(
        "cannot embed array/object variable `${{{0}}}` inside a string; \
         use it as the whole value instead"
    )]
    NonScalarInString(String),
    /// A `${` was opened but never closed with `}`.
    #[error("unterminated `${{` in template string `{0}`")]
    UnterminatedToken(String),
}
