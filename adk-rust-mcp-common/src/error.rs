//! Error types for the common library.
//!
//! This module provides a unified error hierarchy using `thiserror` for consistent
//! error handling across all MCP GenMedia servers.
//!
//! # Error Categories
//!
//! - `ConfigError`: Missing or invalid configuration
//! - `GcsError`: Google Cloud Storage operations
//! - `AuthError`: Authentication failures
//! - `Error::Api`: Google Cloud API errors (includes endpoint and status)
//! - `Error::Validation`: Input validation failures
//! - `Error::Io`: File system operations
//! - `Error::Ffmpeg`: FFmpeg/FFprobe execution errors
//! - `Error::Timeout`: Long-running operation timeouts

use thiserror::Error;

/// Unified error type for the common library.
///
/// This enum provides a single error type that can represent all error conditions
/// across the MCP GenMedia servers, enabling consistent error handling and reporting.
#[derive(Debug, Error)]
pub enum Error {
    /// Configuration errors (missing env vars, invalid values)
    #[error(transparent)]
    Config(#[from] ConfigError),

    /// GCS operation errors (upload, download, invalid URIs)
    #[error(transparent)]
    Gcs(#[from] GcsError),

    /// Authentication errors (ADC not configured, token refresh failures)
    #[error(transparent)]
    Auth(#[from] AuthError),

    /// API errors with endpoint and HTTP status context
    ///
    /// Includes the API endpoint that failed, HTTP status code, and error message
    /// for debugging and user feedback.
    #[error("API error for {endpoint} (HTTP {status_code}): {message}")]
    Api {
        /// The API endpoint that was called
        endpoint: String,
        /// HTTP status code returned by the API
        status_code: u16,
        /// Error message from the API or describing the failure
        message: String,
    },

    /// Input validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// File system I/O errors
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// FFmpeg/FFprobe execution errors
    #[error("FFmpeg error: {0}")]
    Ffmpeg(String),

    /// Operation timeout errors
    #[error("Operation timed out after {0} seconds")]
    Timeout(u64),
}

impl Error {
    /// Create a new API error with endpoint, status code, and message.
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The API endpoint that was called
    /// * `status_code` - The HTTP status code returned
    /// * `message` - A description of the error
    ///
    /// # Example
    ///
    /// ```
    /// use adk_rust_mcp_common::error::Error;
    ///
    /// let err = Error::api(
    ///     "https://api.example.com/v1/generate",
    ///     500,
    ///     "Internal server error"
    /// );
    /// assert!(err.to_string().contains("api.example.com"));
    /// assert!(err.to_string().contains("500"));
    /// ```
    pub fn api(endpoint: impl Into<String>, status_code: u16, message: impl Into<String>) -> Self {
        Error::Api {
            endpoint: endpoint.into(),
            status_code,
            message: message.into(),
        }
    }

    /// Create a new validation error.
    ///
    /// # Example
    ///
    /// ```
    /// use adk_rust_mcp_common::error::Error;
    ///
    /// let err = Error::validation("prompt cannot be empty");
    /// assert!(err.to_string().contains("prompt cannot be empty"));
    /// ```
    pub fn validation(message: impl Into<String>) -> Self {
        Error::Validation(message.into())
    }

    /// Create a new FFmpeg error.
    ///
    /// # Example
    ///
    /// ```
    /// use adk_rust_mcp_common::error::Error;
    ///
    /// let err = Error::ffmpeg("Invalid input format");
    /// assert!(err.to_string().contains("Invalid input format"));
    /// ```
    pub fn ffmpeg(message: impl Into<String>) -> Self {
        Error::Ffmpeg(message.into())
    }

    /// Create a new timeout error.
    ///
    /// # Example
    ///
    /// ```
    /// use adk_rust_mcp_common::error::Error;
    ///
    /// let err = Error::timeout(300);
    /// assert!(err.to_string().contains("300 seconds"));
    /// ```
    pub fn timeout(seconds: u64) -> Self {
        Error::Timeout(seconds)
    }
}

/// Configuration errors.
///
/// These errors occur when loading or validating configuration from
/// environment variables or configuration files.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// A required environment variable is not set
    #[error("Required environment variable {0} is not set")]
    MissingEnvVar(String),

    /// An environment variable has an invalid value
    #[error("Invalid value for {0}: {1}")]
    InvalidValue(String, String),
}

impl ConfigError {
    /// Create a new missing environment variable error.
    pub fn missing_env_var(name: impl Into<String>) -> Self {
        ConfigError::MissingEnvVar(name.into())
    }

    /// Create a new invalid value error.
    pub fn invalid_value(name: impl Into<String>, reason: impl Into<String>) -> Self {
        ConfigError::InvalidValue(name.into(), reason.into())
    }
}

/// GCS operation type for error context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GcsOperation {
    /// Upload operation
    Upload,
    /// Download operation
    Download,
    /// Check existence operation
    Exists,
    /// Delete operation
    Delete,
}

impl std::fmt::Display for GcsOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GcsOperation::Upload => write!(f, "upload"),
            GcsOperation::Download => write!(f, "download"),
            GcsOperation::Exists => write!(f, "exists"),
            GcsOperation::Delete => write!(f, "delete"),
        }
    }
}

/// GCS operation errors.
///
/// These errors occur during Google Cloud Storage operations such as
/// uploading, downloading, or checking object existence.
#[derive(Debug, Error)]
pub enum GcsError {
    /// The GCS URI format is invalid
    #[error("Invalid GCS URI: {0}")]
    InvalidUri(String),

    /// A GCS operation failed with context about the URI and operation type
    #[error("GCS {operation} failed for {uri}: {message}")]
    OperationFailed {
        /// The GCS URI that was being accessed
        uri: String,
        /// The type of operation that failed
        operation: GcsOperation,
        /// Error message describing the failure
        message: String,
    },

    /// Authentication error during GCS operation
    #[error("GCS authentication error: {0}")]
    AuthError(String),
}

impl GcsError {
    /// Create a new invalid URI error.
    pub fn invalid_uri(uri: impl Into<String>) -> Self {
        GcsError::InvalidUri(uri.into())
    }

    /// Create a new operation failed error with full context.
    ///
    /// # Arguments
    ///
    /// * `uri` - The GCS URI that was being accessed
    /// * `operation` - The type of operation that failed
    /// * `message` - A description of the failure
    ///
    /// # Example
    ///
    /// ```
    /// use adk_rust_mcp_common::error::{GcsError, GcsOperation};
    ///
    /// let err = GcsError::operation_failed(
    ///     "gs://my-bucket/path/to/file.txt",
    ///     GcsOperation::Upload,
    ///     "Permission denied"
    /// );
    /// assert!(err.to_string().contains("gs://my-bucket"));
    /// assert!(err.to_string().contains("upload"));
    /// ```
    pub fn operation_failed(
        uri: impl Into<String>,
        operation: GcsOperation,
        message: impl Into<String>,
    ) -> Self {
        GcsError::OperationFailed {
            uri: uri.into(),
            operation,
            message: message.into(),
        }
    }

    /// Create a new authentication error.
    pub fn auth_error(message: impl Into<String>) -> Self {
        GcsError::AuthError(message.into())
    }
}

/// Authentication errors.
///
/// These errors occur during authentication with Google Cloud services
/// using Application Default Credentials (ADC).
#[derive(Debug, Error)]
pub enum AuthError {
    /// ADC is not configured
    #[error("ADC not configured. Run 'gcloud auth application-default login' or set GOOGLE_APPLICATION_CREDENTIALS")]
    NotConfigured,

    /// Token refresh failed
    #[error("Token refresh failed: {0}")]
    RefreshFailed(String),
}

impl AuthError {
    /// Create a new token refresh failed error.
    pub fn refresh_failed(message: impl Into<String>) -> Self {
        AuthError::RefreshFailed(message.into())
    }
}

/// Result type alias using the unified Error type.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_includes_endpoint_and_status() {
        let err = Error::api("https://vertex.googleapis.com/v1/generate", 500, "Internal error");
        let msg = err.to_string();
        assert!(msg.contains("vertex.googleapis.com"), "Should contain endpoint");
        assert!(msg.contains("500"), "Should contain status code");
        assert!(msg.contains("Internal error"), "Should contain message");
    }

    #[test]
    fn test_gcs_error_includes_uri_and_operation() {
        let err = GcsError::operation_failed(
            "gs://my-bucket/path/file.txt",
            GcsOperation::Upload,
            "Access denied",
        );
        let msg = err.to_string();
        assert!(msg.contains("gs://my-bucket"), "Should contain URI");
        assert!(msg.contains("upload"), "Should contain operation type");
        assert!(msg.contains("Access denied"), "Should contain message");
    }

    #[test]
    fn test_config_error_includes_var_name() {
        let err = ConfigError::missing_env_var("PROJECT_ID");
        let msg = err.to_string();
        assert!(msg.contains("PROJECT_ID"), "Should contain variable name");
    }

    #[test]
    fn test_error_from_config_error() {
        let config_err = ConfigError::missing_env_var("TEST_VAR");
        let err: Error = config_err.into();
        assert!(matches!(err, Error::Config(_)));
    }

    #[test]
    fn test_error_from_gcs_error() {
        let gcs_err = GcsError::invalid_uri("invalid://uri");
        let err: Error = gcs_err.into();
        assert!(matches!(err, Error::Gcs(_)));
    }

    #[test]
    fn test_error_from_auth_error() {
        let auth_err = AuthError::NotConfigured;
        let err: Error = auth_err.into();
        assert!(matches!(err, Error::Auth(_)));
    }

    #[test]
    fn test_error_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_timeout_error() {
        let err = Error::timeout(300);
        let msg = err.to_string();
        assert!(msg.contains("300"), "Should contain timeout duration");
        assert!(msg.contains("seconds"), "Should mention seconds");
    }

    #[test]
    fn test_ffmpeg_error() {
        let err = Error::ffmpeg("Invalid codec");
        let msg = err.to_string();
        assert!(msg.contains("FFmpeg"), "Should mention FFmpeg");
        assert!(msg.contains("Invalid codec"), "Should contain message");
    }

    #[test]
    fn test_validation_error() {
        let err = Error::validation("prompt too long");
        let msg = err.to_string();
        assert!(msg.contains("Validation"), "Should mention validation");
        assert!(msg.contains("prompt too long"), "Should contain message");
    }

    #[test]
    fn test_gcs_operation_display() {
        assert_eq!(GcsOperation::Upload.to_string(), "upload");
        assert_eq!(GcsOperation::Download.to_string(), "download");
        assert_eq!(GcsOperation::Exists.to_string(), "exists");
        assert_eq!(GcsOperation::Delete.to_string(), "delete");
    }
}
