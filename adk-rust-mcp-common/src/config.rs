//! Configuration module for loading environment variables and settings.

use crate::error::ConfigError;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Google Cloud project ID (required)
    pub project_id: String,
    /// Google Cloud location/region
    pub location: String,
    /// GCS bucket for media output
    pub gcs_bucket: Option<String>,
    /// HTTP server port
    pub port: u16,
}

impl Config {
    /// Load configuration from environment variables and .env file.
    ///
    /// # Errors
    /// Returns `ConfigError::MissingEnvVar` if PROJECT_ID is not set.
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env file if present (ignore errors if not found)
        let _ = dotenvy::dotenv();

        let project_id = std::env::var("PROJECT_ID")
            .map_err(|_| ConfigError::MissingEnvVar("PROJECT_ID".to_string()))?;

        let location = std::env::var("LOCATION").unwrap_or_else(|_| "us-central1".to_string());

        let gcs_bucket = std::env::var("GCS_BUCKET").ok();

        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        Ok(Self {
            project_id,
            location,
            gcs_bucket,
            port,
        })
    }

    /// Get the Vertex AI endpoint URL for a given API.
    pub fn vertex_ai_endpoint(&self, api: &str) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}",
            self.location, self.project_id, self.location, api
        )
    }
}
