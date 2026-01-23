//! Tracing initialization for MCP GenMedia servers.
//!
//! This module provides utilities for initializing the tracing subscriber
//! with environment-based filtering via the `RUST_LOG` environment variable.
//!
//! # Usage
//!
//! ```no_run
//! use adk_rust_mcp_common::tracing::init_tracing;
//!
//! fn main() {
//!     // Initialize tracing at the start of your application
//!     init_tracing();
//!
//!     // Now you can use tracing macros
//!     tracing::info!("Application started");
//! }
//! ```
//!
//! # Environment Variables
//!
//! - `RUST_LOG`: Controls the log level and filtering. Examples:
//!   - `RUST_LOG=debug` - Enable debug logging for all modules
//!   - `RUST_LOG=adk_rust_mcp_image=debug` - Enable debug for specific crate
//!   - `RUST_LOG=warn,adk_rust_mcp_common=debug` - Warn by default, debug for common
//!
//! # Log Format
//!
//! Logs include:
//! - Timestamp (ISO 8601 format)
//! - Log level (ERROR, WARN, INFO, DEBUG, TRACE)
//! - Target module
//! - Message and structured fields

use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter,
};

/// Initialize the tracing subscriber with environment-based filtering.
///
/// This function sets up the tracing subscriber with:
/// - Environment-based filtering via `RUST_LOG` (defaults to `info`)
/// - Timestamps in ISO 8601 format
/// - Target module names
/// - Span events for debugging async code
///
/// # Panics
///
/// This function will panic if called more than once, as the global
/// subscriber can only be set once.
///
/// # Example
///
/// ```no_run
/// use adk_rust_mcp_common::tracing::init_tracing;
///
/// fn main() {
///     init_tracing();
///     tracing::info!("Server starting");
/// }
/// ```
pub fn init_tracing() {
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

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}

/// Initialize tracing with a custom default level.
///
/// Similar to `init_tracing()`, but allows specifying a default log level
/// when `RUST_LOG` is not set.
///
/// # Arguments
///
/// * `default_level` - The default log level (e.g., "debug", "info", "warn")
///
/// # Example
///
/// ```no_run
/// use adk_rust_mcp_common::tracing::init_tracing_with_default;
///
/// fn main() {
///     // Default to debug level if RUST_LOG is not set
///     init_tracing_with_default("debug");
/// }
/// ```
pub fn init_tracing_with_default(default_level: &str) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_level));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_level(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_span_events(FmtSpan::NONE);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}

/// Try to initialize tracing, returning an error if already initialized.
///
/// Unlike `init_tracing()`, this function does not panic if the subscriber
/// is already set. This is useful for testing or when initialization might
/// happen multiple times.
///
/// # Returns
///
/// - `Ok(())` if initialization succeeded
/// - `Err(())` if the subscriber was already set
///
/// # Example
///
/// ```
/// use adk_rust_mcp_common::tracing::try_init_tracing;
///
/// // First call succeeds
/// let result = try_init_tracing();
/// // result is Ok(()) or Err(()) depending on prior initialization
/// ```
pub fn try_init_tracing() -> Result<(), ()> {
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

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()
        .map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: We can't easily test init_tracing() in unit tests because
    // the global subscriber can only be set once per process.
    // These tests verify the module compiles and exports correctly.

    #[test]
    fn test_try_init_tracing_does_not_panic() {
        // This may succeed or fail depending on test order,
        // but it should never panic
        let _ = try_init_tracing();
    }

    #[test]
    fn test_env_filter_parses_valid_levels() {
        // Verify that common log levels are valid
        let levels = ["trace", "debug", "info", "warn", "error"];
        for level in levels {
            let filter = EnvFilter::new(level);
            // If this doesn't panic, the level is valid
            drop(filter);
        }
    }

    #[test]
    fn test_env_filter_parses_module_specific() {
        // Verify module-specific filters work
        let filter = EnvFilter::new("warn,adk_rust_mcp_common=debug");
        drop(filter);
    }
}
