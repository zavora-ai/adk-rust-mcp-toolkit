//! Google Cloud Storage utilities.

use crate::auth::AuthProvider;
use crate::error::{GcsError, GcsOperation};

/// Parsed GCS URI components.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GcsUri {
    /// Bucket name
    pub bucket: String,
    /// Object path within the bucket
    pub object: String,
}

impl GcsUri {
    /// Parse a `gs://bucket/path` URI into components.
    ///
    /// # Errors
    /// Returns `GcsError::InvalidUri` if the URI format is invalid.
    pub fn parse(uri: &str) -> Result<Self, GcsError> {
        let uri = uri
            .strip_prefix("gs://")
            .ok_or_else(|| GcsError::InvalidUri(format!("URI must start with 'gs://': {}", uri)))?;

        let (bucket, object) = uri
            .split_once('/')
            .ok_or_else(|| GcsError::InvalidUri(format!("URI must contain bucket and path: {}", uri)))?;

        if bucket.is_empty() {
            return Err(GcsError::InvalidUri("Bucket name cannot be empty".to_string()));
        }

        Ok(Self {
            bucket: bucket.to_string(),
            object: object.to_string(),
        })
    }
}

impl std::fmt::Display for GcsUri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "gs://{}/{}", self.bucket, self.object)
    }
}

/// GCS operations client.
pub struct GcsClient {
    client: reqwest::Client,
    auth: AuthProvider,
    /// Base URL for GCS API (configurable for testing)
    base_url: String,
}

impl GcsClient {
    /// Create a new GCS client.
    ///
    /// # Errors
    /// Returns `GcsError::AuthError` if authentication setup fails.
    pub async fn new() -> Result<Self, GcsError> {
        let auth = AuthProvider::new()
            .await
            .map_err(|e| GcsError::AuthError(e.to_string()))?;

        Ok(Self {
            client: reqwest::Client::new(),
            auth,
            base_url: "https://storage.googleapis.com".to_string(),
        })
    }

    /// Create a new GCS client with a provided auth provider.
    pub fn with_auth(auth: AuthProvider) -> Self {
        Self {
            client: reqwest::Client::new(),
            auth,
            base_url: "https://storage.googleapis.com".to_string(),
        }
    }

    /// Create a new GCS client with custom base URL (for testing).
    #[cfg(test)]
    pub fn with_base_url(auth: AuthProvider, base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            auth,
            base_url,
        }
    }

    /// Upload bytes to GCS.
    ///
    /// # Arguments
    /// * `uri` - The GCS URI to upload to
    /// * `data` - The bytes to upload
    /// * `content_type` - The MIME type of the content
    ///
    /// # Errors
    /// Returns `GcsError::OperationFailed` if the upload fails.
    pub async fn upload(
        &self,
        uri: &GcsUri,
        data: &[u8],
        content_type: &str,
    ) -> Result<(), GcsError> {
        let token = self
            .auth
            .get_token(&["https://www.googleapis.com/auth/devstorage.read_write"])
            .await
            .map_err(|e| GcsError::AuthError(e.to_string()))?;

        let url = format!(
            "{}/upload/storage/v1/b/{}/o?uploadType=media&name={}",
            self.base_url,
            uri.bucket,
            urlencoding::encode(&uri.object)
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", content_type)
            .body(data.to_vec())
            .send()
            .await
            .map_err(|e| GcsError::OperationFailed {
                uri: uri.to_string(),
                operation: GcsOperation::Upload,
                message: format!("Upload request failed: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(GcsError::OperationFailed {
                uri: uri.to_string(),
                operation: GcsOperation::Upload,
                message: format!("Failed with status {}: {}", status, body),
            });
        }

        Ok(())
    }

    /// Download bytes from GCS.
    ///
    /// # Arguments
    /// * `uri` - The GCS URI to download from
    ///
    /// # Errors
    /// Returns `GcsError::OperationFailed` if the download fails.
    pub async fn download(&self, uri: &GcsUri) -> Result<Vec<u8>, GcsError> {
        let token = self
            .auth
            .get_token(&["https://www.googleapis.com/auth/devstorage.read_only"])
            .await
            .map_err(|e| GcsError::AuthError(e.to_string()))?;

        let url = format!(
            "{}/storage/v1/b/{}/o/{}?alt=media",
            self.base_url,
            uri.bucket,
            urlencoding::encode(&uri.object)
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| GcsError::OperationFailed {
                uri: uri.to_string(),
                operation: GcsOperation::Download,
                message: format!("Download request failed: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(GcsError::OperationFailed {
                uri: uri.to_string(),
                operation: GcsOperation::Download,
                message: format!("Failed with status {}: {}", status, body),
            });
        }

        response.bytes().await.map(|b| b.to_vec()).map_err(|e| {
            GcsError::OperationFailed {
                uri: uri.to_string(),
                operation: GcsOperation::Download,
                message: format!("Failed to read response body: {}", e),
            }
        })
    }

    /// Check if an object exists in GCS.
    ///
    /// # Arguments
    /// * `uri` - The GCS URI to check
    ///
    /// # Errors
    /// Returns `GcsError::OperationFailed` if the check fails (other than 404).
    pub async fn exists(&self, uri: &GcsUri) -> Result<bool, GcsError> {
        let token = self
            .auth
            .get_token(&["https://www.googleapis.com/auth/devstorage.read_only"])
            .await
            .map_err(|e| GcsError::AuthError(e.to_string()))?;

        let url = format!(
            "{}/storage/v1/b/{}/o/{}",
            self.base_url,
            uri.bucket,
            urlencoding::encode(&uri.object)
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| GcsError::OperationFailed {
                uri: uri.to_string(),
                operation: GcsOperation::Exists,
                message: format!("Exists check request failed: {}", e),
            })?;

        match response.status().as_u16() {
            200 => Ok(true),
            404 => Ok(false),
            status => {
                let body = response.text().await.unwrap_or_default();
                Err(GcsError::OperationFailed {
                    uri: uri.to_string(),
                    operation: GcsOperation::Exists,
                    message: format!("Failed with status {}: {}", status, body),
                })
            }
        }
    }
}
