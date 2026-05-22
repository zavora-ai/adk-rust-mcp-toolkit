//! Configuration module for loading environment variables and settings.

use crate::error::ConfigError;

/// API provider selection.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum ApiProvider {
    /// Google Vertex AI (requires PROJECT_ID, uses ADC auth)
    #[default]
    Vertex,
    /// Google Gemini API (requires GEMINI_API_KEY)
    Gemini,
}

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Google Cloud project ID (required for Vertex)
    pub project_id: String,
    /// Google Cloud location/region
    pub location: String,
    /// GCS bucket for media output
    pub gcs_bucket: Option<String>,
    /// HTTP server port
    pub port: u16,
    /// API provider (vertex or gemini)
    pub api_provider: ApiProvider,
    /// Gemini API key (required when api_provider is Gemini)
    pub gemini_api_key: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            project_id: String::new(),
            location: "us-central1".to_string(),
            gcs_bucket: None,
            port: 8080,
            api_provider: ApiProvider::default(),
            gemini_api_key: None,
        }
    }
}

impl Config {
    /// Load configuration from environment variables and .env file.
    ///
    /// # Errors
    /// Returns `ConfigError::MissingEnvVar` if required vars are not set.
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env file if present (ignore errors if not found)
        let _ = dotenvy::dotenv();

        let gemini_api_key = std::env::var("GEMINI_API_KEY")
            .or_else(|_| std::env::var("GOOGLE_API_KEY"))
            .ok();

        let api_provider = match std::env::var("API_PROVIDER").as_deref() {
            Ok("gemini") => ApiProvider::Gemini,
            Ok("vertex") => ApiProvider::Vertex,
            _ => {
                // Auto-detect: if GEMINI_API_KEY is set, use Gemini
                if gemini_api_key.is_some() {
                    ApiProvider::Gemini
                } else {
                    ApiProvider::Vertex
                }
            }
        };

        let project_id = std::env::var("PROJECT_ID")
            .unwrap_or_else(|_| "".to_string());

        // Require PROJECT_ID only for Vertex
        if api_provider == ApiProvider::Vertex && project_id.is_empty() {
            return Err(ConfigError::MissingEnvVar("PROJECT_ID".to_string()));
        }

        // Require GEMINI_API_KEY for Gemini
        if api_provider == ApiProvider::Gemini && gemini_api_key.is_none() {
            return Err(ConfigError::MissingEnvVar("GEMINI_API_KEY".to_string()));
        }

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
            api_provider,
            gemini_api_key,
        })
    }

    /// Get the Vertex AI endpoint URL for a given model.
    pub fn vertex_ai_endpoint(&self, api: &str) -> String {
        format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}",
            self.location, self.project_id, self.location, api
        )
    }

    /// Get the Gemini API base URL.
    pub fn gemini_base_url(&self) -> &str {
        "https://generativelanguage.googleapis.com/v1beta"
    }

    /// Check if using Gemini API.
    pub fn is_gemini(&self) -> bool {
        self.api_provider == ApiProvider::Gemini
    }
}
