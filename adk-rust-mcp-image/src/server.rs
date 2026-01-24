//! MCP Server implementation for the Image server.
//!
//! This module provides the MCP server handler that exposes:
//! - `image_generate` tool for text-to-image generation
//! - `image_upscale` tool for image upscaling
//! - Resources for models, segmentation classes, and providers

use crate::handler::{ImageGenerateParams, ImageGenerateResult, ImageHandler, ImageUpscaleParams, ImageUpscaleResult};
use crate::resources;
use adk_rust_mcp_common::config::Config;
use adk_rust_mcp_common::error::Error;
use rmcp::{
    model::{
        CallToolResult, Content, ListResourcesResult, ReadResourceResult,
        ResourceContents, ServerCapabilities, ServerInfo,
    },
    ErrorData as McpError, ServerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::borrow::Cow;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// MCP Server for image generation.
#[derive(Clone)]
pub struct ImageServer {
    /// Handler for image generation operations
    handler: Arc<RwLock<Option<ImageHandler>>>,
    /// Server configuration
    config: Config,
}

/// Tool parameters wrapper for image_generate.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImageGenerateToolParams {
    /// Text prompt describing the image to generate
    pub prompt: String,
    /// Negative prompt - what to avoid in the generated image
    #[serde(default)]
    pub negative_prompt: Option<String>,
    /// Model to use for generation (default: imagen-4.0-generate-preview-05-20)
    #[serde(default)]
    pub model: Option<String>,
    /// Aspect ratio (1:1, 3:4, 4:3, 9:16, 16:9)
    #[serde(default)]
    pub aspect_ratio: Option<String>,
    /// Number of images to generate (1-4)
    #[serde(default)]
    pub number_of_images: Option<u8>,
    /// Random seed for reproducibility
    #[serde(default)]
    pub seed: Option<i64>,
    /// Output file path for saving locally
    #[serde(default)]
    pub output_file: Option<String>,
    /// Output storage URI (e.g., gs://bucket/path)
    #[serde(default)]
    pub output_uri: Option<String>,
}

impl From<ImageGenerateToolParams> for ImageGenerateParams {
    fn from(params: ImageGenerateToolParams) -> Self {
        Self {
            prompt: params.prompt,
            negative_prompt: params.negative_prompt,
            model: params.model.unwrap_or_else(|| crate::handler::DEFAULT_MODEL.to_string()),
            aspect_ratio: params.aspect_ratio.unwrap_or_else(|| "1:1".to_string()),
            number_of_images: params.number_of_images.unwrap_or(1),
            seed: params.seed,
            output_file: params.output_file,
            output_uri: params.output_uri,
        }
    }
}

/// Tool parameters wrapper for image_upscale.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImageUpscaleToolParams {
    /// Source image to upscale (base64 data, local path, or GCS URI)
    pub image: String,
    /// Upscale factor: "x2" or "x4" (default: "x2")
    #[serde(default)]
    pub upscale_factor: Option<String>,
    /// Output file path for saving locally
    #[serde(default)]
    pub output_file: Option<String>,
    /// Output storage URI (e.g., gs://bucket/path)
    #[serde(default)]
    pub output_uri: Option<String>,
}

impl From<ImageUpscaleToolParams> for ImageUpscaleParams {
    fn from(params: ImageUpscaleToolParams) -> Self {
        Self {
            image: params.image,
            upscale_factor: params.upscale_factor.unwrap_or_else(|| "x2".to_string()),
            output_file: params.output_file,
            output_uri: params.output_uri,
        }
    }
}

impl ImageServer {
    /// Create a new ImageServer with the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            handler: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// Initialize the handler (called lazily on first use).
    async fn ensure_handler(&self) -> Result<(), Error> {
        let mut handler = self.handler.write().await;
        if handler.is_none() {
            *handler = Some(ImageHandler::new(self.config.clone()).await?);
        }
        Ok(())
    }

    /// Generate images from a text prompt.
    pub async fn generate_image(&self, params: ImageGenerateToolParams) -> Result<CallToolResult, McpError> {
        info!(prompt = %params.prompt, "Generating image");

        // Ensure handler is initialized
        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard.as_ref().ok_or_else(|| {
            McpError::internal_error("Handler not initialized", None)
        })?;

        let gen_params: ImageGenerateParams = params.into();
        let result = handler.generate_image(gen_params).await.map_err(|e| {
            McpError::internal_error(format!("Image generation failed: {}", e), None)
        })?;

        // Convert result to MCP content
        let content = match result {
            ImageGenerateResult::Base64(images) => {
                images
                    .into_iter()
                    .map(|img| Content::image(img.data, img.mime_type))
                    .collect()
            }
            ImageGenerateResult::LocalFiles(paths) => {
                vec![Content::text(format!("Images saved to: {}", paths.join(", ")))]
            }
            ImageGenerateResult::StorageUris(uris) => {
                vec![Content::text(format!("Images uploaded to: {}", uris.join(", ")))]
            }
        };

        Ok(CallToolResult::success(content))
    }

    /// Upscale an image.
    pub async fn upscale_image(&self, params: ImageUpscaleToolParams) -> Result<CallToolResult, McpError> {
        info!(upscale_factor = ?params.upscale_factor, "Upscaling image");

        // Ensure handler is initialized
        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard.as_ref().ok_or_else(|| {
            McpError::internal_error("Handler not initialized", None)
        })?;

        let upscale_params: ImageUpscaleParams = params.into();
        let result = handler.upscale_image(upscale_params).await.map_err(|e| {
            McpError::internal_error(format!("Image upscaling failed: {}", e), None)
        })?;

        // Convert result to MCP content
        let content = match result {
            ImageUpscaleResult::Base64(image) => {
                vec![Content::image(image.data, image.mime_type)]
            }
            ImageUpscaleResult::LocalFile(path) => {
                vec![Content::text(format!("Upscaled image saved to: {}", path))]
            }
            ImageUpscaleResult::StorageUri(uri) => {
                vec![Content::text(format!("Upscaled image uploaded to: {}", uri))]
            }
        };

        Ok(CallToolResult::success(content))
    }
}

impl ServerHandler for ImageServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Image generation and processing server using Google Vertex AI Imagen API. \
                 Use image_generate to create images from text prompts, \
                 and image_upscale to upscale existing images."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            ..Default::default()
        }
    }

    fn list_tools(
        &self,
        _params: Option<rmcp::model::PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::ListToolsResult, McpError>> + Send + '_ {
        async move {
            use rmcp::model::{ListToolsResult, Tool};
            use schemars::schema_for;

            // image_generate tool
            let gen_schema = schema_for!(ImageGenerateToolParams);
            let gen_schema_value = serde_json::to_value(&gen_schema).unwrap_or_default();
            let gen_input_schema = match gen_schema_value {
                serde_json::Value::Object(map) => Arc::new(map),
                _ => Arc::new(serde_json::Map::new()),
            };

            // image_upscale tool
            let upscale_schema = schema_for!(ImageUpscaleToolParams);
            let upscale_schema_value = serde_json::to_value(&upscale_schema).unwrap_or_default();
            let upscale_input_schema = match upscale_schema_value {
                serde_json::Value::Object(map) => Arc::new(map),
                _ => Arc::new(serde_json::Map::new()),
            };

            Ok(ListToolsResult {
                tools: vec![
                    Tool {
                        name: Cow::Borrowed("image_generate"),
                        description: Some(Cow::Borrowed(
                            "Generate images from a text prompt using Google's Imagen API. \
                             Returns base64-encoded image data, local file paths, or storage URIs \
                             depending on output parameters."
                        )),
                        input_schema: gen_input_schema,
                        annotations: None,
                        icons: None,
                        meta: None,
                        output_schema: None,
                        title: None,
                    },
                    Tool {
                        name: Cow::Borrowed("image_upscale"),
                        description: Some(Cow::Borrowed(
                            "Upscale an image using Google's Imagen 4.0 Upscale API. \
                             Supports x2 and x4 upscale factors. \
                             Accepts base64 image data, local file path, or GCS URI as input. \
                             Returns base64-encoded image data, local file path, or storage URI."
                        )),
                        input_schema: upscale_input_schema,
                        annotations: None,
                        icons: None,
                        meta: None,
                        output_schema: None,
                        title: None,
                    },
                ],
                next_cursor: None,
                meta: None,
            })
        }
    }

    fn call_tool(
        &self,
        params: rmcp::model::CallToolRequestParam,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        async move {
            match params.name.as_ref() {
                "image_generate" => {
                    let tool_params: ImageGenerateToolParams = params
                        .arguments
                        .map(|args| serde_json::from_value(serde_json::Value::Object(args)))
                        .transpose()
                        .map_err(|e| McpError::invalid_params(format!("Invalid parameters: {}", e), None))?
                        .ok_or_else(|| McpError::invalid_params("Missing parameters", None))?;

                    self.generate_image(tool_params).await
                }
                "image_upscale" => {
                    let tool_params: ImageUpscaleToolParams = params
                        .arguments
                        .map(|args| serde_json::from_value(serde_json::Value::Object(args)))
                        .transpose()
                        .map_err(|e| McpError::invalid_params(format!("Invalid parameters: {}", e), None))?
                        .ok_or_else(|| McpError::invalid_params("Missing parameters", None))?;

                    self.upscale_image(tool_params).await
                }
                _ => Err(McpError::invalid_params(format!("Unknown tool: {}", params.name), None)),
            }
        }
    }

    fn list_resources(
        &self,
        _params: Option<rmcp::model::PaginatedRequestParam>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        async move {
            debug!("Listing resources");
            
            // Build resources using the raw struct approach
            let models_resource = rmcp::model::Resource {
                raw: rmcp::model::RawResource {
                    uri: "image://models".to_string(),
                    name: "Available Image Models".to_string(),
                    title: None,
                    description: Some("List of available image generation models".to_string()),
                    mime_type: Some("application/json".to_string()),
                    size: None,
                    icons: None,
                    meta: None,
                },
                annotations: None,
            };

            let segmentation_resource = rmcp::model::Resource {
                raw: rmcp::model::RawResource {
                    uri: "image://segmentation_classes".to_string(),
                    name: "Segmentation Classes".to_string(),
                    title: None,
                    description: Some("List of segmentation classes for image editing (Google provider)".to_string()),
                    mime_type: Some("application/json".to_string()),
                    size: None,
                    icons: None,
                    meta: None,
                },
                annotations: None,
            };

            let providers_resource = rmcp::model::Resource {
                raw: rmcp::model::RawResource {
                    uri: "image://providers".to_string(),
                    name: "Available Providers".to_string(),
                    title: None,
                    description: Some("List of available image generation providers".to_string()),
                    mime_type: Some("application/json".to_string()),
                    size: None,
                    icons: None,
                    meta: None,
                },
                annotations: None,
            };

            Ok(ListResourcesResult {
                resources: vec![models_resource, segmentation_resource, providers_resource],
                next_cursor: None,
                meta: None,
            })
        }
    }

    fn read_resource(
        &self,
        params: rmcp::model::ReadResourceRequestParam,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        async move {
            let uri = &params.uri;
            debug!(uri = %uri, "Reading resource");

            let content = match uri.as_str() {
                "image://models" => resources::models_resource_json(),
                "image://segmentation_classes" => resources::segmentation_classes_resource_json(),
                "image://providers" => resources::providers_resource_json(),
                _ => {
                    return Err(McpError::resource_not_found(
                        format!("Unknown resource: {}", uri),
                        None,
                    ));
                }
            };

            Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(content, uri.clone())],
            })
        }
    }
}

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
    fn test_server_info() {
        let server = ImageServer::new(test_config());
        let info = server.get_info();
        assert!(info.instructions.is_some());
    }

    #[test]
    fn test_tool_params_conversion() {
        let tool_params = ImageGenerateToolParams {
            prompt: "A cat".to_string(),
            negative_prompt: Some("blurry".to_string()),
            model: Some("imagen-4".to_string()),
            aspect_ratio: Some("16:9".to_string()),
            number_of_images: Some(2),
            seed: Some(42),
            output_file: None,
            output_uri: None,
        };

        let gen_params: ImageGenerateParams = tool_params.into();
        assert_eq!(gen_params.prompt, "A cat");
        assert_eq!(gen_params.negative_prompt, Some("blurry".to_string()));
        assert_eq!(gen_params.model, "imagen-4");
        assert_eq!(gen_params.aspect_ratio, "16:9");
        assert_eq!(gen_params.number_of_images, 2);
        assert_eq!(gen_params.seed, Some(42));
    }

    #[test]
    fn test_tool_params_defaults() {
        let tool_params = ImageGenerateToolParams {
            prompt: "A cat".to_string(),
            negative_prompt: None,
            model: None,
            aspect_ratio: None,
            number_of_images: None,
            seed: None,
            output_file: None,
            output_uri: None,
        };

        let gen_params: ImageGenerateParams = tool_params.into();
        assert_eq!(gen_params.model, crate::handler::DEFAULT_MODEL);
        assert_eq!(gen_params.aspect_ratio, "1:1");
        assert_eq!(gen_params.number_of_images, 1);
    }
}
