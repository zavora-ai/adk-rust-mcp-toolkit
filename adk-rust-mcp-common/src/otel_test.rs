//! Unit tests for OpenTelemetry initialization module.
//!
//! These tests verify:
//! - Configuration loading from environment variables
//! - Graceful handling when OpenTelemetry is disabled
//! - Error handling for missing configuration

use super::otel::*;
use std::env;

/// Helper to temporarily set environment variables for testing.
struct EnvGuard {
    vars: Vec<(String, Option<String>)>,
}

impl EnvGuard {
    fn new() -> Self {
        Self { vars: Vec::new() }
    }

    fn set(&mut self, key: &str, value: &str) {
        let old_value = env::var(key).ok();
        self.vars.push((key.to_string(), old_value));
        // SAFETY: We're in a test environment and restore the original value on drop
        unsafe { env::set_var(key, value) };
    }

    fn remove(&mut self, key: &str) {
        let old_value = env::var(key).ok();
        self.vars.push((key.to_string(), old_value));
        // SAFETY: We're in a test environment and restore the original value on drop
        unsafe { env::remove_var(key) };
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in self.vars.drain(..).rev() {
            // SAFETY: We're restoring the original environment state
            match value {
                Some(v) => unsafe { env::set_var(&key, v) },
                None => unsafe { env::remove_var(&key) },
            }
        }
    }
}

#[test]
fn test_otel_config_default() {
    let config = OtelConfig::default();
    
    assert!(!config.enabled);
    assert!(config.project_id.is_none());
    assert_eq!(config.service_name, "adk-rust-mcp");
    assert_eq!(config.default_log_level, "info");
}

#[test]
fn test_otel_config_new_with_project_id() {
    let config = OtelConfig::new("my-project");
    
    assert!(config.enabled);
    assert_eq!(config.project_id, Some("my-project".to_string()));
    assert_eq!(config.service_name, "adk-rust-mcp");
}

#[test]
fn test_otel_config_builder_methods() {
    let config = OtelConfig::new("my-project")
        .with_service_name("custom-service")
        .with_default_log_level("debug")
        .with_enabled(false);
    
    assert!(!config.enabled);
    assert_eq!(config.project_id, Some("my-project".to_string()));
    assert_eq!(config.service_name, "custom-service");
    assert_eq!(config.default_log_level, "debug");
}

#[test]
fn test_otel_config_from_env_disabled_by_default() {
    let mut guard = EnvGuard::new();
    guard.remove("OTEL_ENABLED");
    guard.remove("PROJECT_ID");
    guard.remove("OTEL_SERVICE_NAME");
    
    let config = OtelConfig::from_env().expect("Should load config");
    
    assert!(!config.enabled);
    assert!(config.project_id.is_none());
    assert_eq!(config.service_name, "adk-rust-mcp");
}

#[test]
fn test_otel_config_from_env_enabled_true() {
    let mut guard = EnvGuard::new();
    guard.set("OTEL_ENABLED", "true");
    guard.set("PROJECT_ID", "test-project");
    guard.set("OTEL_SERVICE_NAME", "test-service");
    
    let config = OtelConfig::from_env().expect("Should load config");
    
    assert!(config.enabled);
    assert_eq!(config.project_id, Some("test-project".to_string()));
    assert_eq!(config.service_name, "test-service");
}

#[test]
fn test_otel_config_from_env_enabled_one() {
    let mut guard = EnvGuard::new();
    guard.set("OTEL_ENABLED", "1");
    guard.remove("PROJECT_ID");
    
    let config = OtelConfig::from_env().expect("Should load config");
    
    assert!(config.enabled);
}

#[test]
fn test_otel_config_from_env_enabled_false_string() {
    let mut guard = EnvGuard::new();
    guard.set("OTEL_ENABLED", "false");
    
    let config = OtelConfig::from_env().expect("Should load config");
    
    assert!(!config.enabled);
}

#[test]
fn test_otel_config_from_env_enabled_invalid_string() {
    let mut guard = EnvGuard::new();
    guard.set("OTEL_ENABLED", "yes");
    
    let config = OtelConfig::from_env().expect("Should load config");
    
    // "yes" is not "true" or "1", so it should be disabled
    assert!(!config.enabled);
}

#[test]
fn test_is_otel_enabled_true() {
    let mut guard = EnvGuard::new();
    guard.set("OTEL_ENABLED", "true");
    
    assert!(is_otel_enabled());
}

#[test]
fn test_is_otel_enabled_one() {
    let mut guard = EnvGuard::new();
    guard.set("OTEL_ENABLED", "1");
    
    assert!(is_otel_enabled());
}

#[test]
fn test_is_otel_enabled_false() {
    let mut guard = EnvGuard::new();
    guard.set("OTEL_ENABLED", "false");
    
    assert!(!is_otel_enabled());
}

#[test]
fn test_is_otel_enabled_not_set() {
    let mut guard = EnvGuard::new();
    guard.remove("OTEL_ENABLED");
    
    assert!(!is_otel_enabled());
}

#[test]
fn test_otel_error_display() {
    let errors = vec![
        (OtelError::NotEnabled, "OpenTelemetry is not enabled"),
        (OtelError::MissingProjectId, "PROJECT_ID environment variable is required"),
        (OtelError::ExporterCreationFailed("test error".to_string()), "Failed to create Google Cloud Trace exporter: test error"),
        (OtelError::TracerInstallFailed("install error".to_string()), "Failed to install tracer provider: install error"),
        (OtelError::SubscriberSetFailed("subscriber error".to_string()), "Failed to set global tracing subscriber: subscriber error"),
    ];
    
    for (error, expected_substring) in errors {
        let error_string = error.to_string();
        assert!(
            error_string.contains(expected_substring),
            "Error '{}' should contain '{}'",
            error_string,
            expected_substring
        );
    }
}

#[tokio::test]
async fn test_init_otel_tracing_not_enabled() {
    let config = OtelConfig::default();
    
    let result = init_otel_tracing(config).await;
    
    assert!(result.is_err());
    match result {
        Err(OtelError::NotEnabled) => {}
        Err(e) => panic!("Expected NotEnabled error, got: {:?}", e),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[tokio::test]
async fn test_init_otel_tracing_missing_project_id() {
    let config = OtelConfig {
        enabled: true,
        project_id: None,
        service_name: "test".to_string(),
        default_log_level: "info".to_string(),
    };
    
    let result = init_otel_tracing(config).await;
    
    assert!(result.is_err());
    match result {
        Err(OtelError::MissingProjectId) => {}
        Err(e) => panic!("Expected MissingProjectId error, got: {:?}", e),
        Ok(_) => panic!("Expected error, got Ok"),
    }
}

#[tokio::test]
async fn test_init_tracing_with_optional_otel_disabled() {
    let config = OtelConfig::default();
    
    // This should not panic and should return None
    let guard = init_tracing_with_optional_otel(config).await;
    
    assert!(guard.is_none());
}

// Note: We cannot easily test successful initialization in unit tests because:
// 1. It requires valid GCP credentials
// 2. The global tracing subscriber can only be set once per process
// 3. Integration tests with real GCP would be more appropriate for full initialization testing
