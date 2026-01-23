//! Property-based tests for error module.
//!
//! These tests validate that error messages include the required context
//! as specified in the design document.

use proptest::prelude::*;

use crate::error::{Error, GcsError, GcsOperation};

/// Generate valid HTTP status codes (100-599)
fn http_status_strategy() -> impl Strategy<Value = u16> {
    100u16..600u16
}

/// Generate valid API endpoint URLs
fn endpoint_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("https://[a-z]+\\.googleapis\\.com/v[0-9]+/[a-z]+")
        .unwrap()
        .prop_filter("endpoint must be non-empty", |s| !s.is_empty())
}

/// Generate error messages
fn message_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 ]{1,100}"
}

/// Generate valid GCS URIs
fn gcs_uri_strategy() -> impl Strategy<Value = String> {
    ("[a-z][a-z0-9-]{2,20}", "[a-z0-9/._-]{1,50}").prop_map(|(bucket, object)| {
        format!("gs://{}/{}", bucket, object)
    })
}

/// Generate GCS operations
fn gcs_operation_strategy() -> impl Strategy<Value = GcsOperation> {
    prop_oneof![
        Just(GcsOperation::Upload),
        Just(GcsOperation::Download),
        Just(GcsOperation::Exists),
        Just(GcsOperation::Delete),
    ]
}

proptest! {
    /// **Property 16: Error Context Inclusion (API Errors)**
    ///
    /// *For any* API error, the error message SHALL include the API endpoint
    /// that failed and the HTTP status code.
    ///
    /// **Validates: Requirements 10.5**
    #[test]
    fn api_error_includes_endpoint_and_status(
        endpoint in endpoint_strategy(),
        status_code in http_status_strategy(),
        message in message_strategy()
    ) {
        let err = Error::api(&endpoint, status_code, &message);
        let err_string = err.to_string();

        // Error message must include the endpoint
        prop_assert!(
            err_string.contains(&endpoint),
            "API error should include endpoint '{}' in message: {}",
            endpoint,
            err_string
        );

        // Error message must include the status code
        prop_assert!(
            err_string.contains(&status_code.to_string()),
            "API error should include status code '{}' in message: {}",
            status_code,
            err_string
        );
    }

    /// **Property 16: Error Context Inclusion (GCS Errors)**
    ///
    /// *For any* GCS error, the error message SHALL include the GCS URI
    /// and the operation type (upload/download/exists/delete).
    ///
    /// **Validates: Requirements 10.6**
    #[test]
    fn gcs_error_includes_uri_and_operation(
        uri in gcs_uri_strategy(),
        operation in gcs_operation_strategy(),
        message in message_strategy()
    ) {
        let err = GcsError::operation_failed(&uri, operation, &message);
        let err_string = err.to_string();

        // Error message must include the GCS URI
        prop_assert!(
            err_string.contains(&uri),
            "GCS error should include URI '{}' in message: {}",
            uri,
            err_string
        );

        // Error message must include the operation type
        let operation_str = operation.to_string();
        prop_assert!(
            err_string.contains(&operation_str),
            "GCS error should include operation '{}' in message: {}",
            operation_str,
            err_string
        );
    }

    /// Test that GCS operation types are correctly displayed
    #[test]
    fn gcs_operation_display_is_lowercase(operation in gcs_operation_strategy()) {
        let display = operation.to_string();
        prop_assert!(
            display.chars().all(|c| c.is_lowercase()),
            "GCS operation display should be lowercase: {}",
            display
        );
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn api_error_format_is_readable() {
        let err = Error::api(
            "https://vertex.googleapis.com/v1/projects/my-project/locations/us-central1/publishers/google/models/imagen-3.0:predict",
            500,
            "Internal server error"
        );
        let msg = err.to_string();

        // Should be human-readable
        assert!(msg.contains("API error"), "Should indicate it's an API error");
        assert!(msg.contains("HTTP"), "Should mention HTTP");
        assert!(msg.contains("500"), "Should include status code");
        assert!(msg.contains("vertex.googleapis.com"), "Should include endpoint");
    }

    #[test]
    fn gcs_error_format_is_readable() {
        let err = GcsError::operation_failed(
            "gs://my-bucket/path/to/file.png",
            GcsOperation::Upload,
            "Permission denied",
        );
        let msg = err.to_string();

        // Should be human-readable
        assert!(msg.contains("GCS"), "Should indicate it's a GCS error");
        assert!(msg.contains("upload"), "Should include operation type");
        assert!(msg.contains("gs://my-bucket"), "Should include URI");
        assert!(msg.contains("Permission denied"), "Should include message");
    }

    #[test]
    fn all_gcs_operations_have_distinct_display() {
        let operations = [
            GcsOperation::Upload,
            GcsOperation::Download,
            GcsOperation::Exists,
            GcsOperation::Delete,
        ];

        let displays: Vec<String> = operations.iter().map(|op| op.to_string()).collect();

        // All displays should be unique
        for (i, d1) in displays.iter().enumerate() {
            for (j, d2) in displays.iter().enumerate() {
                if i != j {
                    assert_ne!(d1, d2, "Operations should have distinct display strings");
                }
            }
        }
    }

    #[test]
    fn error_conversion_preserves_context() {
        // GCS error converted to unified Error should preserve context
        let gcs_err = GcsError::operation_failed(
            "gs://bucket/object",
            GcsOperation::Download,
            "Not found",
        );
        let unified_err: Error = gcs_err.into();
        let msg = unified_err.to_string();

        assert!(msg.contains("gs://bucket/object"), "Should preserve URI");
        assert!(msg.contains("download"), "Should preserve operation");
    }
}
