use thiserror::Error;

#[derive(Debug, Clone)]
pub struct ApiBaseConfig {
    pub server_url: String,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone)]
pub enum AuthConfig {
    BasicAuth { username: String, password: String },
    ApiKey { api_key: String },
}

#[derive(Error, Debug)]
pub enum ApiSdkError {
    #[error("HTTP client error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// A non-2xx response whose body was successfully read. Carries the
    /// server's actual error message (e.g. a `bad_request(...)` reason)
    /// rather than the bare status line `reqwest::Error` would otherwise
    /// produce once `error_for_status()` has already dropped the body.
    #[error("API error ({status}): {message}")]
    ApiError { status: u16, message: String },

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Rate limit exceeded")]
    RateLimitError,
}

pub type ApiResult<T> = Result<T, ApiSdkError>;
