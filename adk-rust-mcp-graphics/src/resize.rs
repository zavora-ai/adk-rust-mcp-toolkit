//! graphics_resize — Resize/reframe an image to a new aspect ratio (outpainting).

use adk_rust_mcp_common::Config;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ResizeParams {
    /// Source image (file path or base64 data)
    pub image: String,
    /// Target aspect ratio (e.g. "16:9", "9:16", "1:1")
    pub aspect_ratio: String,
    /// Output file path
    #[serde(default)]
    pub output_file: Option<String>,
}

pub async fn generate(config: &Config, params: ResizeParams) -> Result<String, String> {
    let (b64, mime) = crate::load_image_as_base64(&params.image).await?;
    let parts = vec![
        serde_json::json!({"inline_data": {"mime_type": mime, "data": b64}}),
        serde_json::json!({"text": format!("Extend this image to fill a {} frame. Generate new content to fill the extended areas naturally.", params.aspect_ratio)}),
    ];
    let output = params.output_file.unwrap_or_else(|| "resize_output.png".into());
    let image_bytes = crate::call_gemini_image(config, "gemini-3.1-flash-image-preview", parts, Some(&params.aspect_ratio)).await?;
    crate::save_image(&image_bytes, &output).await
}
