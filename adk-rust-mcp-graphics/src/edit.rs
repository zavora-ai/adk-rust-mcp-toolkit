//! graphics_edit — Edit an image with natural language instructions.

use adk_rust_mcp_common::Config;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EditParams {
    /// Source image (file path or base64 data)
    pub image: String,
    /// Natural language editing instruction
    pub instruction: String,
    /// Output file path
    #[serde(default)]
    pub output_file: Option<String>,
}

pub async fn generate(config: &Config, params: EditParams) -> Result<String, String> {
    let (b64, mime) = crate::load_image_as_base64(&params.image).await?;
    let parts = vec![
        serde_json::json!({"inline_data": {"mime_type": mime, "data": b64}}),
        serde_json::json!({"text": params.instruction}),
    ];
    let output = params.output_file.unwrap_or_else(|| "edited_output.png".into());
    let image_bytes = crate::call_gemini_image(config, "gemini-3.1-flash-image-preview", parts, None).await?;
    crate::save_image(&image_bytes, &output).await
}
