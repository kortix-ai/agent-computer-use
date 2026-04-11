use thiserror::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("element not found: {message}")]
    ElementNotFound { message: String },

    #[error("ambiguous selector: found {count} elements matching the query")]
    AmbiguousSelector { count: usize },

    #[error("action not supported: {message}")]
    ActionNotSupported { message: String },

    #[error("accessibility permission denied: {message}")]
    PermissionDenied { message: String },

    #[error("application not found: {name}")]
    ApplicationNotFound { name: String },

    #[error("platform error: {message}")]
    PlatformError { message: String },

    #[error("unsupported platform: {platform}")]
    UnsupportedPlatform { platform: String },

    #[error("timeout after {seconds}s: {message}")]
    Timeout { seconds: f64, message: String },

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
