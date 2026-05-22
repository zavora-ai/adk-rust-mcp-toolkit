//! graphics_remove_object — Remove an object from an image.

use adk_rust_mcp_common::Config;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RemoveParams {
    /// Source image (file path or base64 data)
    pub image: String,
    /// Object to remove from the image
    pub object_to_remove: String,
    /// Output file path
    #[serde(default)]
    pub output_file: Option<String>,
}

pub async fn generate(config: &Config, params: RemoveParams) -> Result<String, String> {
    let (b64, mime) = crate::load_image_as_base64(&params.image).await?;
    let parts = vec![
        serde_json::json!({"inline_data": {"mime_type": mime, "data": b64}}),
        serde_json::json!({"text": format!("Remove the {} from this image. Fill the area naturally.", params.object_to_remove)}),
    ];
    let output = params.output_file.unwrap_or_else(|| "remove_output.png".into());
    let image_bytes = crate::call_gemini_image(config, "gemini-3.1-flash-image-preview", parts, None).await?;
    crate::save_image(&image_bytes, &output).await
}
