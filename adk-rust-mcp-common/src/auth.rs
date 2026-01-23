//! Authentication module using Application Default Credentials.
//!
//! This module provides authentication for Google Cloud APIs using ADC (Application Default Credentials).
//! It supports:
//! - Service account credentials via `GOOGLE_APPLICATION_CREDENTIALS` environment variable
//! - User credentials from `gcloud auth application-default login`
//! - GCE metadata server for workloads running on Google Cloud
//! - gcloud CLI fallback

use std::sync::Arc;

use gcp_auth::TokenProvider;
use tracing::{debug, instrument};

use crate::error::AuthError;

/// Internal token source abstraction for production and testing.
enum TokenSource {
    /// Production token provider from gcp_auth
    Provider(Arc<dyn TokenProvider>),
    /// Mock token for testing
    #[cfg(test)]
    Mock(String),
}

/// Authentication provider using Application Default Credentials.
///
/// Wraps the `gcp_auth` crate to provide automatic credential discovery and token refresh.
/// Tokens are cached internally and refreshed automatically when they expire.
pub struct AuthProvider {
    /// The underlying token source
    source: TokenSource,
}

impl AuthProvider {
    /// Create a new auth provider using Application Default Credentials.
    ///
    /// This will attempt to find credentials in the following order:
    /// 1. Service account JSON file specified by `GOOGLE_APPLICATION_CREDENTIALS`
    /// 2. User credentials from `~/.config/gcloud/application_default_credentials.json`
    /// 3. GCE metadata server (when running on Google Cloud)
    /// 4. gcloud CLI (if available on PATH)
    ///
    /// # Errors
    ///
    /// Returns `AuthError::NotConfigured` if no valid credentials can be found.
    #[instrument(level = "debug", name = "auth_provider_new")]
    pub async fn new() -> Result<Self, AuthError> {
        debug!("Initializing AuthProvider with ADC");

        let provider = gcp_auth::provider().await.map_err(|e| {
            debug!("Failed to initialize ADC: {}", e);
            AuthError::NotConfigured
        })?;

        debug!("AuthProvider initialized successfully");
        Ok(Self {
            source: TokenSource::Provider(provider),
        })
    }

    /// Create a mock auth provider for testing.
    ///
    /// This method is only available in test builds and returns a provider
    /// that always returns the specified token without making any network calls.
    #[cfg(test)]
    pub fn mock(token: &str) -> Self {
        Self {
            source: TokenSource::Mock(token.to_string()),
        }
    }

    /// Get a valid access token for the specified scopes.
    ///
    /// Tokens are cached internally and will be refreshed automatically when they expire.
    /// The caller should not cache tokens themselves.
    ///
    /// # Arguments
    ///
    /// * `scopes` - OAuth2 scopes to request. Common scopes include:
    ///   - `https://www.googleapis.com/auth/cloud-platform` - Full access to Google Cloud
    ///   - `https://www.googleapis.com/auth/devstorage.read_write` - GCS read/write
    ///
    /// # Errors
    ///
    /// Returns `AuthError::RefreshFailed` if the token cannot be obtained or refreshed.
    #[instrument(level = "debug", name = "get_token", skip(self))]
    pub async fn get_token(&self, scopes: &[&str]) -> Result<String, AuthError> {
        debug!(?scopes, "Requesting token");

        match &self.source {
            TokenSource::Provider(provider) => {
                let token = provider.token(scopes).await.map_err(|e| {
                    debug!("Token refresh failed: {}", e);
                    AuthError::RefreshFailed(e.to_string())
                })?;

                debug!("Token obtained successfully");
                Ok(token.as_str().to_string())
            }
            #[cfg(test)]
            TokenSource::Mock(token) => {
                debug!("Returning mock token");
                Ok(token.clone())
            }
        }
    }
}

/// Common OAuth2 scopes for Google Cloud APIs.
pub mod scopes {
    /// Full access to Google Cloud Platform APIs.
    pub const CLOUD_PLATFORM: &str = "https://www.googleapis.com/auth/cloud-platform";

    /// Read/write access to Google Cloud Storage.
    pub const DEVSTORAGE_READ_WRITE: &str = "https://www.googleapis.com/auth/devstorage.read_write";

    /// Read-only access to Google Cloud Storage.
    pub const DEVSTORAGE_READ_ONLY: &str = "https://www.googleapis.com/auth/devstorage.read_only";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_auth_provider() {
        let mock = AuthProvider::mock("test-token-123");
        let token = mock.get_token(&["scope"]).await.unwrap();
        assert_eq!(token, "test-token-123");
    }

    #[tokio::test]
    async fn test_mock_auth_provider_ignores_scopes() {
        let mock = AuthProvider::mock("my-token");
        
        // Different scopes should return the same mock token
        let token1 = mock.get_token(&["scope1"]).await.unwrap();
        let token2 = mock.get_token(&["scope1", "scope2"]).await.unwrap();
        let token3 = mock.get_token(&[]).await.unwrap();
        
        assert_eq!(token1, "my-token");
        assert_eq!(token2, "my-token");
        assert_eq!(token3, "my-token");
    }

    #[test]
    fn test_scopes_constants() {
        assert!(scopes::CLOUD_PLATFORM.contains("cloud-platform"));
        assert!(scopes::DEVSTORAGE_READ_WRITE.contains("devstorage"));
        assert!(scopes::DEVSTORAGE_READ_ONLY.contains("devstorage"));
    }
}
