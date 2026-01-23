//! OpenTelemetry tracing integration for MCP GenMedia servers.
//!
//! This module provides optional OpenTelemetry tracing initialization with
//! Google Cloud Trace export support. It integrates with the existing `tracing`
//! infrastructure to provide distributed tracing capabilities.
//!
//! # Feature Flag
//!
//! This module is only available when the `otel` feature is enabled:
//!
//! ```toml
//! [dependencies]
//! adk-rust-mcp-common = { version = "*", features = ["otel"] }
//! ```
//!
//! # Usage
//!
//! ```no_run
//! use adk_rust_mcp_common::otel::{OtelConfig, init_otel_tracing};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize OpenTelemetry with Google Cloud Trace
//!     let config = OtelConfig::from_env()?;
//!     let guard = init_otel_tracing(config).await?;
//!
//!     // Your application code here...
//!     tracing::info!("Application started with OpenTelemetry tracing");
//!
//!     // Shutdown is handled automatically when guard is dropped
//!     Ok(())
//! }
//! ```
//!
//! # Environment Variables
//!
//! - `OTEL_ENABLED`: Set to "true" or "1" to enable OpenTelemetry (default: disabled)
//! - `PROJECT_ID`: Google Cloud project ID for trace export
//! - `OTEL_SERVICE_NAME`: Service name for traces (default: "adk-rust-mcp")
//! - `RUST_LOG`: Controls log level filtering (same as standard tracing)

use std::env;
use thiserror::Error;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter,
};

// Re-export types from opentelemetry-gcloud-trace to ensure version compatibility
use opentelemetry_gcloud_trace::GcpCloudTraceExporterBuilder;

/// Errors that can occur during OpenTelemetry initialization.
#[derive(Debug, Error)]
pub enum OtelError {
    /// OpenTelemetry is not enabled via environment variable.
    #[error("OpenTelemetry is not enabled. Set OTEL_ENABLED=true to enable.")]
    NotEnabled,

    /// Missing required PROJECT_ID for Google Cloud Trace export.
    #[error("PROJECT_ID environment variable is required for Google Cloud Trace export")]
    MissingProjectId,

    /// Failed to create the Google Cloud Trace exporter.
    #[error("Failed to create Google Cloud Trace exporter: {0}")]
    ExporterCreationFailed(String),

    /// Failed to install the tracer provider.
    #[error("Failed to install tracer provider: {0}")]
    TracerInstallFailed(String),

    /// Failed to set the global subscriber.
    #[error("Failed to set global tracing subscriber: {0}")]
    SubscriberSetFailed(String),
}

/// Configuration for OpenTelemetry tracing.
#[derive(Debug, Clone)]
pub struct OtelConfig {
    /// Whether OpenTelemetry is enabled.
    pub enabled: bool,
    /// Google Cloud project ID for trace export.
    pub project_id: Option<String>,
    /// Service name for traces.
    pub service_name: String,
    /// Default log level when RUST_LOG is not set.
    pub default_log_level: String,
}

impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            project_id: None,
            service_name: "adk-rust-mcp".to_string(),
            default_log_level: "info".to_string(),
        }
    }
}

impl OtelConfig {
    /// Create a new OtelConfig with the given project ID.
    pub fn new(project_id: impl Into<String>) -> Self {
        Self {
            enabled: true,
            project_id: Some(project_id.into()),
            ..Default::default()
        }
    }

    /// Load configuration from environment variables.
    ///
    /// # Environment Variables
    ///
    /// - `OTEL_ENABLED`: Set to "true" or "1" to enable (default: false)
    /// - `PROJECT_ID`: Google Cloud project ID
    /// - `OTEL_SERVICE_NAME`: Service name (default: "adk-rust-mcp")
    pub fn from_env() -> Result<Self, OtelError> {
        let enabled = env::var("OTEL_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        let project_id = env::var("PROJECT_ID").ok();
        let service_name = env::var("OTEL_SERVICE_NAME")
            .unwrap_or_else(|_| "adk-rust-mcp".to_string());

        Ok(Self {
            enabled,
            project_id,
            service_name,
            default_log_level: "info".to_string(),
        })
    }

    /// Set the service name.
    pub fn with_service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = name.into();
        self
    }

    /// Set the default log level.
    pub fn with_default_log_level(mut self, level: impl Into<String>) -> Self {
        self.default_log_level = level.into();
        self
    }

    /// Enable or disable OpenTelemetry.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Guard that ensures proper shutdown of OpenTelemetry when dropped.
///
/// This struct holds the tracer provider and ensures it is properly
/// shut down when the guard goes out of scope.
pub struct OtelGuard {
    #[allow(dead_code)]
    provider: opentelemetry_sdk::trace::SdkTracerProvider,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        tracing::debug!("Shutting down OpenTelemetry tracer provider");
        if let Err(e) = self.provider.shutdown() {
            tracing::error!("Failed to shutdown OpenTelemetry tracer provider: {:?}", e);
        }
    }
}

/// Initialize OpenTelemetry tracing with Google Cloud Trace export.
///
/// This function sets up the tracing subscriber with:
/// - Environment-based filtering via `RUST_LOG`
/// - Console output formatting
/// - OpenTelemetry layer for distributed tracing (when enabled)
/// - Google Cloud Trace export
///
/// # Arguments
///
/// * `config` - OpenTelemetry configuration
///
/// # Returns
///
/// Returns an `OtelGuard` that must be kept alive for the duration of the
/// application. When dropped, it will properly shut down the tracer provider.
///
/// # Errors
///
/// Returns an error if:
/// - OpenTelemetry is not enabled in the config
/// - PROJECT_ID is missing when OpenTelemetry is enabled
/// - Failed to create the Google Cloud Trace exporter
/// - Failed to set the global subscriber
///
/// # Example
///
/// ```no_run
/// use adk_rust_mcp_common::otel::{OtelConfig, init_otel_tracing};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = OtelConfig::new("my-gcp-project");
///     let _guard = init_otel_tracing(config).await?;
///     
///     tracing::info!("Tracing initialized");
///     Ok(())
/// }
/// ```
pub async fn init_otel_tracing(config: OtelConfig) -> Result<OtelGuard, OtelError> {
    if !config.enabled {
        return Err(OtelError::NotEnabled);
    }

    let project_id = config.project_id.ok_or(OtelError::MissingProjectId)?;

    // Create the Google Cloud Trace exporter
    let exporter = GcpCloudTraceExporterBuilder::new(project_id);

    let provider = exporter
        .create_provider()
        .await
        .map_err(|e| OtelError::ExporterCreationFailed(e.to_string()))?;

    let tracer = exporter
        .install(&provider)
        .await
        .map_err(|e| OtelError::TracerInstallFailed(e.to_string()))?;

    // Set the global tracer provider
    opentelemetry::global::set_tracer_provider(provider.clone());

    // Create the OpenTelemetry layer
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Create the env filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.default_log_level));

    // Create the fmt layer for console output
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_span_events(FmtSpan::NONE);

    // Build and set the subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(otel_layer)
        .try_init()
        .map_err(|e| OtelError::SubscriberSetFailed(e.to_string()))?;

    tracing::info!(
        service_name = %config.service_name,
        "OpenTelemetry tracing initialized with Google Cloud Trace export"
    );

    Ok(OtelGuard { provider })
}

/// Initialize tracing with optional OpenTelemetry support.
///
/// This is a convenience function that:
/// - Initializes OpenTelemetry if enabled in the config
/// - Falls back to standard console tracing if OpenTelemetry is disabled
///
/// # Arguments
///
/// * `config` - OpenTelemetry configuration
///
/// # Returns
///
/// Returns `Some(OtelGuard)` if OpenTelemetry was initialized, `None` otherwise.
///
/// # Example
///
/// ```no_run
/// use adk_rust_mcp_common::otel::{OtelConfig, init_tracing_with_optional_otel};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = OtelConfig::from_env()?;
///     let _guard = init_tracing_with_optional_otel(config).await;
///     
///     tracing::info!("Application started");
///     Ok(())
/// }
/// ```
pub async fn init_tracing_with_optional_otel(config: OtelConfig) -> Option<OtelGuard> {
    if config.enabled {
        match init_otel_tracing(config).await {
            Ok(guard) => Some(guard),
            Err(e) => {
                // Fall back to standard tracing
                init_fallback_tracing();
                tracing::warn!("Failed to initialize OpenTelemetry, using standard tracing: {}", e);
                None
            }
        }
    } else {
        init_fallback_tracing();
        tracing::debug!("OpenTelemetry disabled, using standard tracing");
        None
    }
}

/// Initialize standard tracing without OpenTelemetry.
///
/// This is used as a fallback when OpenTelemetry is disabled or fails to initialize.
fn init_fallback_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_span_events(FmtSpan::NONE);

    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init();
}

/// Check if OpenTelemetry is enabled via environment variable.
///
/// This is a quick check that doesn't require loading the full config.
pub fn is_otel_enabled() -> bool {
    env::var("OTEL_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}
