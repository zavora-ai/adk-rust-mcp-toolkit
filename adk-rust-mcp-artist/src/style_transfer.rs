//! artist_style_transfer — Apply an art style to an image.

use adk_rust_mcp_common::Config;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StyleTransferParams {
    /// Source image (file path or base64 data)
    pub content_image: String,
    /// Description of the style to apply
    pub style_description: String,
    /// Output file path
    #[serde(default)]
    pub output_file: Option<String>,
}

pub async fn generate(config: &Config, params: StyleTransferParams) -> Result<String, String> {
    let (b64, mime) = crate::load_image_as_base64(&params.content_image).await?;
    let parts = vec![
        serde_json::json!({"inline_data": {"mime_type": mime, "data": b64}}),
        serde_json::json!({"text": format!("Transform this image into: {}. Preserve the composition but change the artistic style.", params.style_description)}),
    ];
    let output = params.output_file.unwrap_or_else(|| "style_transfer_output.png".into());
    let image_bytes = crate::call_gemini_image(config, "gemini-3.1-flash-image-preview", parts, None, None).await?;
    crate::save_image(&image_bytes, &output).await
}
