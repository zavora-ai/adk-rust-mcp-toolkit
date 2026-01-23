//! Property-based tests for GCS module.

use proptest::prelude::*;

use crate::gcs::GcsUri;

/// Generate valid GCS bucket names.
/// Bucket names must be 3-63 characters, lowercase letters, numbers, hyphens, underscores.
fn bucket_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_-]{2,62}".prop_filter("bucket name must be valid", |s| {
        !s.is_empty() && s.len() >= 3 && s.len() <= 63
    })
}

/// Generate valid GCS object paths.
/// Object paths can contain most characters except newlines.
fn object_path_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_/.-]{1,100}".prop_filter("object path must be non-empty", |s| !s.is_empty())
}

proptest! {
    /// **Property 2: GCS URI Round-Trip Parsing**
    ///
    /// *For any* valid GCS URI in the format `gs://bucket/path/to/object`,
    /// parsing the URI into a `GcsUri` struct and then formatting it back
    /// to a string SHALL produce an equivalent URI string.
    ///
    /// **Validates: Requirements 2.9**
    #[test]
    fn gcs_uri_round_trip(
        bucket in bucket_name_strategy(),
        object in object_path_strategy()
    ) {
        // Construct a valid GCS URI
        let original_uri = format!("gs://{}/{}", bucket, object);

        // Parse the URI
        let parsed = GcsUri::parse(&original_uri)
            .expect("Valid URI should parse successfully");

        // Verify parsed components
        prop_assert_eq!(&parsed.bucket, &bucket, "Bucket should match");
        prop_assert_eq!(&parsed.object, &object, "Object path should match");

        // Format back to string
        let formatted = parsed.to_string();

        // Round-trip should produce equivalent URI
        prop_assert_eq!(formatted, original_uri, "Round-trip should preserve URI");
    }

    /// Test that invalid URIs are rejected.
    #[test]
    fn invalid_uri_rejected(uri in "[^g].*|g[^s].*|gs[^:].*") {
        // URIs not starting with "gs://" should fail
        let result = GcsUri::parse(&uri);
        prop_assert!(result.is_err(), "Invalid URI should be rejected: {}", uri);
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn parse_valid_uri() {
        let uri = GcsUri::parse("gs://my-bucket/path/to/object.txt").unwrap();
        assert_eq!(uri.bucket, "my-bucket");
        assert_eq!(uri.object, "path/to/object.txt");
    }

    #[test]
    fn parse_uri_with_empty_object() {
        // Empty object path is valid (root of bucket)
        let uri = GcsUri::parse("gs://my-bucket/").unwrap();
        assert_eq!(uri.bucket, "my-bucket");
        assert_eq!(uri.object, "");
    }

    #[test]
    fn parse_uri_missing_prefix() {
        let result = GcsUri::parse("s3://bucket/path");
        assert!(result.is_err());
    }

    #[test]
    fn parse_uri_missing_path() {
        let result = GcsUri::parse("gs://bucket");
        assert!(result.is_err());
    }

    #[test]
    fn parse_uri_empty_bucket() {
        let result = GcsUri::parse("gs:///path");
        assert!(result.is_err());
    }

    #[test]
    fn to_string_formats_correctly() {
        let uri = GcsUri {
            bucket: "test-bucket".to_string(),
            object: "folder/file.txt".to_string(),
        };
        assert_eq!(uri.to_string(), "gs://test-bucket/folder/file.txt");
    }
}

/// Unit tests for GcsClient with mocked API.
/// **Validates: Requirements 2.7, 2.8, 2.10**
#[cfg(test)]
mod gcs_client_tests {
    use wiremock::matchers::{header, method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::auth::AuthProvider;
    use crate::gcs::{GcsClient, GcsUri};

    const TEST_TOKEN: &str = "test-token-12345";

    #[tokio::test]
    async fn upload_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path_regex(r"/upload/storage/v1/b/.*/o.*"))
            .and(header("Authorization", format!("Bearer {}", TEST_TOKEN)))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "name": "test-object.txt",
                "bucket": "test-bucket"
            })))
            .mount(&mock_server)
            .await;

        let auth = AuthProvider::mock(TEST_TOKEN);
        let client = GcsClient::with_base_url(auth, mock_server.uri());

        let uri = GcsUri {
            bucket: "test-bucket".to_string(),
            object: "test-object.txt".to_string(),
        };

        let result = client.upload(&uri, b"test data", "text/plain").await;
        assert!(result.is_ok(), "Upload should succeed: {:?}", result);
    }

    #[tokio::test]
    async fn upload_failure_returns_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path_regex(r"/upload/storage/v1/b/.*/o.*"))
            .respond_with(ResponseTemplate::new(403).set_body_string("Access denied"))
            .mount(&mock_server)
            .await;

        let auth = AuthProvider::mock(TEST_TOKEN);
        let client = GcsClient::with_base_url(auth, mock_server.uri());

        let uri = GcsUri {
            bucket: "test-bucket".to_string(),
            object: "test-object.txt".to_string(),
        };

        let result = client.upload(&uri, b"test data", "text/plain").await;
        assert!(result.is_err(), "Upload should fail");

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("403") && err_msg.contains("gs://test-bucket/test-object.txt"),
            "Error should include status and URI: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn download_success() {
        let mock_server = MockServer::start().await;
        let test_data = b"downloaded content";

        Mock::given(method("GET"))
            .and(path_regex(r"/storage/v1/b/.*/o/.*"))
            .and(header("Authorization", format!("Bearer {}", TEST_TOKEN)))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(test_data.to_vec()))
            .mount(&mock_server)
            .await;

        let auth = AuthProvider::mock(TEST_TOKEN);
        let client = GcsClient::with_base_url(auth, mock_server.uri());

        let uri = GcsUri {
            bucket: "test-bucket".to_string(),
            object: "test-object.txt".to_string(),
        };

        let result = client.download(&uri).await;
        assert!(result.is_ok(), "Download should succeed: {:?}", result);
        assert_eq!(result.unwrap(), test_data.to_vec());
    }

    #[tokio::test]
    async fn download_failure_returns_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path_regex(r"/storage/v1/b/.*/o/.*"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not found"))
            .mount(&mock_server)
            .await;

        let auth = AuthProvider::mock(TEST_TOKEN);
        let client = GcsClient::with_base_url(auth, mock_server.uri());

        let uri = GcsUri {
            bucket: "test-bucket".to_string(),
            object: "nonexistent.txt".to_string(),
        };

        let result = client.download(&uri).await;
        assert!(result.is_err(), "Download should fail");

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("404") && err_msg.contains("gs://test-bucket/nonexistent.txt"),
            "Error should include status and URI: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn exists_returns_true_when_object_exists() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path_regex(r"/storage/v1/b/.*/o/[^?]+$"))
            .and(header("Authorization", format!("Bearer {}", TEST_TOKEN)))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "name": "test-object.txt",
                "bucket": "test-bucket"
            })))
            .mount(&mock_server)
            .await;

        let auth = AuthProvider::mock(TEST_TOKEN);
        let client = GcsClient::with_base_url(auth, mock_server.uri());

        let uri = GcsUri {
            bucket: "test-bucket".to_string(),
            object: "test-object.txt".to_string(),
        };

        let result = client.exists(&uri).await;
        assert!(result.is_ok(), "Exists check should succeed: {:?}", result);
        assert!(result.unwrap(), "Object should exist");
    }

    #[tokio::test]
    async fn exists_returns_false_when_object_not_found() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path_regex(r"/storage/v1/b/.*/o/[^?]+$"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let auth = AuthProvider::mock(TEST_TOKEN);
        let client = GcsClient::with_base_url(auth, mock_server.uri());

        let uri = GcsUri {
            bucket: "test-bucket".to_string(),
            object: "nonexistent.txt".to_string(),
        };

        let result = client.exists(&uri).await;
        assert!(result.is_ok(), "Exists check should succeed: {:?}", result);
        assert!(!result.unwrap(), "Object should not exist");
    }

    #[tokio::test]
    async fn exists_returns_error_on_server_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path_regex(r"/storage/v1/b/.*/o/[^?]+$"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal server error"))
            .mount(&mock_server)
            .await;

        let auth = AuthProvider::mock(TEST_TOKEN);
        let client = GcsClient::with_base_url(auth, mock_server.uri());

        let uri = GcsUri {
            bucket: "test-bucket".to_string(),
            object: "test-object.txt".to_string(),
        };

        let result = client.exists(&uri).await;
        assert!(result.is_err(), "Exists check should fail on server error");
    }
}
