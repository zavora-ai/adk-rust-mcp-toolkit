# RMCP Server Implementation Patterns

This document captures patterns and best practices for implementing MCP servers using the `rmcp` crate (v0.14) in this workspace.

## Overview

The workspace uses `rmcp` v0.14 for implementing Model Context Protocol servers. Each server follows a consistent pattern with handlers, resources, and the MCP server trait implementation.

## Project Structure

Each MCP server crate follows this structure:

```
adk-rust-mcp-{name}/
├── Cargo.toml
├── tests/
│   └── integration_test.rs  # Integration tests with real APIs
└── src/
    ├── lib.rs       # Library exports for integration tests
    ├── main.rs      # Entry point with CLI args and server startup
    ├── handler.rs   # Business logic, parameter types, API interactions
    ├── resources.rs # MCP resource definitions
    └── server.rs    # ServerHandler implementation
```

## Environment Configuration

Use a `.env` file in the workspace root for configuration:

```bash
# Google Cloud Configuration
PROJECT_ID=your-project-id
LOCATION=us-central1

# Storage buckets (provider-specific)
GCS_BUCKET=your-gcs-bucket
# S3_BUCKET=your-s3-bucket  # Future

# Server port (default: 8080)
PORT=8080

# Logging level
RUST_LOG=info
```

## Key rmcp 0.14 API Patterns

### ServerHandler Implementation

The `ServerHandler` trait requires implementing several methods. Here's the correct pattern:

```rust
use rmcp::{
    model::{
        CallToolResult, Content, ListResourcesResult, ReadResourceResult,
        ResourceContents, ServerCapabilities, ServerInfo,
    },
    ErrorData as McpError, ServerHandler,
};

impl ServerHandler for MyServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Server description".to_string()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            ..Default::default()
        }
    }

    fn list_tools(
        &self,
        _params: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::ListToolsResult, McpError>> + Send + '_ {
        async move {
            // Implementation
        }
    }

    fn call_tool(
        &self,
        params: rmcp::model::CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        async move {
            // Implementation
        }
    }

    fn list_resources(
        &self,
        _params: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        async move {
            // Implementation
        }
    }

    fn read_resource(
        &self,
        params: rmcp::model::ReadResourceRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        async move {
            // Implementation
        }
    }
}
```

### Tool Definition

Tools are defined manually in `list_tools()`:

```rust
use rmcp::model::{ListToolsResult, Tool};
use schemars::schema_for;
use std::borrow::Cow;
use std::sync::Arc;

fn list_tools(&self, ...) -> ... {
    async move {
        let schema = schema_for!(MyToolParams);
        let schema_value = serde_json::to_value(&schema).unwrap_or_default();
        
        // Convert to Arc<Map> as required by rmcp
        let input_schema = match schema_value {
            serde_json::Value::Object(map) => Arc::new(map),
            _ => Arc::new(serde_json::Map::new()),
        };

        Ok(ListToolsResult {
            tools: vec![Tool {
                name: Cow::Borrowed("tool_name"),
                description: Some(Cow::Borrowed("Tool description")),
                input_schema,
                annotations: None,
                icons: None,
                meta: None,
                output_schema: None,
                title: None,
            }],
            next_cursor: None,
            meta: None,
        })
    }
}
```

### Tool Invocation

Handle tool calls by matching on the tool name:

```rust
fn call_tool(&self, params: rmcp::model::CallToolRequestParams, ...) -> ... {
    async move {
        match params.name.as_ref() {
            "tool_name" => {
                let tool_params: MyToolParams = params
                    .arguments
                    .map(|args| serde_json::from_value(serde_json::Value::Object(args)))
                    .transpose()
                    .map_err(|e| McpError::invalid_params(format!("Invalid parameters: {}", e), None))?
                    .ok_or_else(|| McpError::invalid_params("Missing parameters", None))?;

                self.handle_tool(tool_params).await
            }
            _ => Err(McpError::invalid_params(format!("Unknown tool: {}", params.name), None)),
        }
    }
}
```

### Resource Definition

Resources use the `RawResource` struct:

```rust
fn list_resources(&self, ...) -> ... {
    async move {
        let resource = rmcp::model::Resource {
            raw: rmcp::model::RawResource {
                uri: "scheme://path".to_string(),
                name: "Resource Name".to_string(),
                title: None,
                description: Some("Description".to_string()),
                mime_type: Some("application/json".to_string()),
                size: None,
                icons: None,
                meta: None,
            },
            annotations: None,
        };

        Ok(ListResourcesResult {
            resources: vec![resource],
            next_cursor: None,
            meta: None,
        })
    }
}
```

### Resource Reading

```rust
fn read_resource(&self, params: rmcp::model::ReadResourceRequestParams, ...) -> ... {
    async move {
        let uri = &params.uri;
        
        let content = match uri.as_str() {
            "scheme://path" => get_resource_content(),
            _ => return Err(McpError::resource_not_found(
                format!("Unknown resource: {}", uri),
                None,
            )),
        };

        Ok(ReadResourceResult {
            contents: vec![ResourceContents::text(content, uri.clone())],
        })
    }
}
```

### Content Types

Use the builder methods on `Content`:

```rust
// Text content
Content::text("Some text")

// Image content (base64 data)
Content::image(base64_data, "image/png")

// Success result
CallToolResult::success(vec![Content::text("Result")])
```

### Error Handling

Use `McpError` (alias for `ErrorData`):

```rust
use rmcp::ErrorData as McpError;

// Invalid parameters
McpError::invalid_params("Error message", None)

// Internal error
McpError::internal_error("Error message", None)

// Resource not found
McpError::resource_not_found("Error message", None)
```

## Handler Pattern

### Lazy Initialization

Use `Arc<RwLock<Option<Handler>>>` for lazy handler initialization:

```rust
pub struct MyServer {
    handler: Arc<RwLock<Option<MyHandler>>>,
    config: Config,
}

impl MyServer {
    pub fn new(config: Config) -> Self {
        Self {
            handler: Arc::new(RwLock::new(None)),
            config,
        }
    }

    async fn ensure_handler(&self) -> Result<(), Error> {
        let mut handler = self.handler.write().await;
        if handler.is_none() {
            *handler = Some(MyHandler::new(self.config.clone()).await?);
        }
        Ok(())
    }
}
```

### Parameter Types

Define parameter types with serde and schemars:

```rust
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MyToolParams {
    /// Required field with description
    pub required_field: String,
    
    /// Optional field with default
    #[serde(default)]
    pub optional_field: Option<String>,
    
    /// Field with custom default
    #[serde(default = "default_value")]
    pub with_default: String,
}

fn default_value() -> String {
    "default".to_string()
}
```

### Validation

Implement validation that returns all errors:

```rust
impl MyParams {
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        if self.field.is_empty() {
            errors.push(ValidationError {
                field: "field".to_string(),
                message: "cannot be empty".to_string(),
            });
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

## Main Entry Point

```rust
use adk_rust_mcp_common::{Config, McpServerBuilder, TransportArgs};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "adk-rust-mcp-{name}")]
#[command(about = "MCP server description")]
struct Args {
    #[command(flatten)]
    transport: TransportArgs,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let config = Config::from_env()?;
    let server = MyServer::new(config);
    let transport = args.transport.into_transport();

    McpServerBuilder::new(server)
        .with_transport(transport)
        .run()
        .await?;

    Ok(())
}
```

## Testing Patterns

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config {
            project_id: "test-project".to_string(),
            location: "us-central1".to_string(),
            gcs_bucket: None,
            port: 8080,
        }
    }

    #[test]
    fn test_validation() {
        let params = MyParams { ... };
        assert!(params.validate().is_ok());
    }
}
```

### Property-Based Tests

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: rust-mcp-genmedia, Property N: Description
    // **Validates: Requirements X.Y**

    proptest! {
        #[test]
        fn property_name(value in strategy()) {
            // Property assertion
            prop_assert!(condition);
        }
    }
}
```

### Integration Tests

Integration tests live in `tests/integration_test.rs` and test against real APIs:

```rust
//! Integration tests for adk-rust-mcp-{name} server.
//!
//! Run with: `cargo test --package adk-rust-mcp-{name} --test integration_test`
//! Skip in CI: `cargo test --package adk-rust-mcp-{name} --lib`

use std::env;
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize environment from .env file once
fn init_env() {
    INIT.call_once(|| {
        let _ = dotenvy::dotenv();
    });
}

/// Helper to get test configuration from environment.
fn get_test_config() -> Option<Config> {
    init_env();
    
    let project_id = env::var("PROJECT_ID").ok()?;
    
    Some(Config {
        project_id,
        location: env::var("LOCATION").unwrap_or_else(|_| "us-central1".to_string()),
        gcs_bucket: env::var("GCS_BUCKET").ok(),
        port: 8080,
    })
}

/// Check if integration tests should run.
fn should_run_integration_tests() -> bool {
    if env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        return false;
    }
    get_test_config().is_some()
}

/// Macro to skip test if integration tests are disabled.
macro_rules! skip_if_no_integration {
    () => {
        if !should_run_integration_tests() {
            eprintln!("Skipping integration test: no valid configuration");
            return;
        }
    };
}

#[tokio::test]
async fn test_api_call() {
    skip_if_no_integration!();
    
    let config = get_test_config().unwrap();
    let handler = MyHandler::new(config).await.expect("Failed to create handler");
    
    // Test actual API call
    let result = handler.do_something().await;
    assert!(result.is_ok());
}
```

### Test Output Directory

Save generated files to a persistent directory for inspection:

```rust
const TEST_OUTPUT_DIR: &str = "test_output";

fn get_test_output_dir() -> PathBuf {
    let dir = PathBuf::from(TEST_OUTPUT_DIR);
    if !dir.exists() {
        std::fs::create_dir_all(&dir).expect("Failed to create test output directory");
    }
    dir
}
```

Add to `.gitignore`:
```
test_output/
```

## Common Dependencies

All server crates should include:

```toml
[package]
name = "adk-rust-mcp-{name}"
# ...

[lib]
name = "adk_rust_mcp_{name}"
path = "src/lib.rs"

[[bin]]
name = "adk-rust-mcp-{name}"
path = "src/main.rs"

[dependencies]
adk-rust-mcp-common.workspace = true
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
anyhow.workspace = true
rmcp.workspace = true
reqwest.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
schemars.workspace = true
async-trait.workspace = true
base64.workspace = true
clap.workspace = true

[dev-dependencies]
proptest.workspace = true
tempfile = "3"
dotenvy.workspace = true
```

## API Request/Response Types

For Vertex AI APIs, use camelCase serialization:

```rust
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiRequest {
    pub sample_count: u8,  // Serializes as "sampleCount"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional_field: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse {
    pub bytes_base64_encoded: Option<String>,  // Deserializes from "bytesBase64Encoded"
}
```

## Vertex AI API Patterns

### Imagen API (Image Generation)

```rust
// Endpoint format
let endpoint = format!(
    "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:predict",
    location, project_id, location, model_id
);

// Request structure
#[derive(Debug, Serialize)]
pub struct ImagenRequest {
    pub instances: Vec<ImagenInstance>,
    pub parameters: ImagenParameters,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagenInstance {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagenParameters {
    pub sample_count: u8,
    pub aspect_ratio: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,  // Note: Not supported when watermark is enabled
}

// Response structure
#[derive(Debug, Deserialize)]
pub struct ImagenResponse {
    pub predictions: Vec<ImagenPrediction>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagenPrediction {
    pub bytes_base64_encoded: Option<String>,
    pub mime_type: Option<String>,
}
```

### Model IDs

Model IDs change over time. Current models (as of January 2025):

- **Imagen 4**: `imagen-4.0-generate-preview-06-06` (aliases: `imagen-4`, `imagen-4.0`)
- **Imagen 3**: `imagen-3.0-generate-002` (aliases: `imagen-3`, `imagen-3.0`)
- **Imagen 3 Fast**: `imagen-3.0-fast-generate-001`

### API Limitations

- **Seed parameter**: Not supported when watermark is enabled (default for Imagen 4)
- **Rate limits**: Be mindful of API quotas in integration tests

## GCS Storage Pattern

```rust
use adk_rust_mcp_common::gcs::{GcsClient, GcsUri};

// Parse GCS URI
let uri = GcsUri::parse("gs://bucket/path/to/file.png")?;

// Upload
let gcs = GcsClient::with_auth(auth_provider);
gcs.upload(&uri, &data, "image/png").await?;

// Download
let data = gcs.download(&uri).await?;

// Check existence
let exists = gcs.exists(&uri).await?;
```

## Vertex AI Veo API (Video Generation)

The Veo API uses Long-Running Operations (LRO) for video generation since it can take 2-5 minutes.

### Veo API Endpoints

```rust
// Start video generation (returns LRO)
let generate_endpoint = format!(
    "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:predictLongRunning",
    location, project_id, location, model_id
);

// Poll LRO status (uses fetchPredictOperation, NOT the operation name directly)
let poll_endpoint = format!(
    "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:fetchPredictOperation",
    location, project_id, location, model_id
);
```

### Veo Request Structure

```rust
#[derive(Debug, Serialize)]
pub struct VeoT2vRequest {
    pub instances: Vec<VeoT2vInstance>,
    pub parameters: VeoParameters,
}

#[derive(Debug, Serialize)]
pub struct VeoT2vInstance {
    pub prompt: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VeoParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    #[serde(rename = "storageUri")]
    pub storage_uri: String,  // GCS URI for output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,  // Veo 3.x only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
}
```

### CRITICAL: Veo LRO Response Structure

**The Veo API returns `videos` NOT `generatedSamples` in the LRO response.**

This is a common pitfall - always verify the actual API response structure against the official documentation.

```rust
// CORRECT response structure for Veo LRO polling
#[derive(Debug, Deserialize)]
pub struct LroStatusResponse {
    pub done: Option<bool>,
    pub error: Option<LroError>,
    pub response: Option<LroResultResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LroResultResponse {
    // API returns "videos" array, NOT "generatedSamples"
    pub videos: Option<Vec<VideoOutput>>,
    #[serde(default)]
    pub rai_media_filtered_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoOutput {
    pub gcs_uri: Option<String>,
    pub mime_type: Option<String>,
}
```

Sample API response:
```json
{
   "done": true,
   "response": {
      "@type": "type.googleapis.com/cloud.ai.large_models.vision.GenerateVideoResponse",
      "raiMediaFilteredCount": 0,
      "videos": [
         {
           "gcsUri": "gs://bucket/path/sample_0.mp4",
           "mimeType": "video/mp4"
         }
      ]
   }
}
```

### LRO Polling with Exponential Backoff

```rust
pub const LRO_INITIAL_DELAY_MS: u64 = 5000;    // 5 seconds
pub const LRO_MAX_DELAY_MS: u64 = 60000;       // 60 seconds max between polls
pub const LRO_BACKOFF_MULTIPLIER: f64 = 1.5;
pub const LRO_MAX_ATTEMPTS: u32 = 120;         // ~30 minutes total timeout

// Polling uses POST to fetchPredictOperation with operation name in body
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchOperationRequest {
    pub operation_name: String,
}
```

### Veo Model IDs (as of January 2026)

- **Veo 3**: `veo-3.0-generate-preview` (supports audio generation)
- **Veo 2**: `veo-2.0-generate-001`

### Veo API Constraints

- **Duration**: 4, 6, or 8 seconds (model-dependent)
- **Aspect ratios**: "16:9", "9:16"
- **Audio generation**: Only supported on Veo 3.x models
- **Output**: Requires GCS URI (`storageUri` parameter)

## Lessons Learned

### Always Verify API Response Structures

When integrating with external APIs:
1. **Check official documentation** for the exact response field names
2. **Log raw responses** during development to verify structure
3. **Don't assume** field names match request parameters (e.g., `generatedSamples` vs `videos`)
4. **Test with real API calls** early to catch deserialization issues

### LRO Polling Best Practices

1. Use exponential backoff to avoid rate limiting
2. Cap maximum delay to ensure reasonable polling frequency
3. Set a total timeout appropriate for the operation (video gen: 30+ minutes)
4. Use the correct polling endpoint (Veo uses `fetchPredictOperation`, not direct operation URL)

### Debugging API Integration Issues

When you get "No data generated" or similar errors:
1. Check if the response structure matches your deserialization types
2. Verify field names match exactly (including camelCase vs snake_case)
3. Look for `@type` fields in responses that indicate the actual response schema
4. Add debug logging to capture raw API responses

## Reference Implementation

See `adk-rust-mcp-image` for a complete reference implementation of these patterns, including:
- Full ServerHandler implementation
- Integration tests with Vertex AI Imagen API
- GCS upload/download tests
- Local file output tests
- Property-based tests for validation

See `adk-rust-mcp-video` for LRO-based video generation patterns, including:
- Long-running operation polling with exponential backoff
- Correct Veo API response parsing
- Video download from GCS
