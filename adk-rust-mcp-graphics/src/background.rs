//! graphics_replace_background — Replace the background of an image.

use adk_rust_mcp_common::Config;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BackgroundParams {
    /// Source image (file path or base64 data)
    pub image: String,
    /// Description of the new background
    pub new_background: String,
    /// Output file path
    #[serde(default)]
    pub output_file: Option<String>,
}

pub async fn generate(config: &Config, params: BackgroundParams) -> Result<String, String> {
    let (b64, mime) = crate::load_image_as_base64(&params.image).await?;
    let parts = vec![
        serde_json::json!({"inline_data": {"mime_type": mime, "data": b64}}),
        serde_json::json!({"text": format!("Keep the main subject but replace the background with: {}", params.new_background)}),
    ];
    let output = params.output_file.unwrap_or_else(|| "background_output.png".into());
    let image_bytes = crate::call_gemini_image(config, "gemini-3.1-flash-image-preview", parts, None).await?;
    crate::save_image(&image_bytes, &output).await
}
