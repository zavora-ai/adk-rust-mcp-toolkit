//! artist_sketch_to_art — Turn a rough sketch into finished artwork.

use adk_rust_mcp_common::Config;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SketchToArtParams {
    /// Sketch image (file path or base64 data)
    pub sketch_image: String,
    /// Description of what the sketch depicts
    pub description: String,
    /// Art style for the finished piece
    #[serde(default = "default_style")]
    pub style: String,
    /// Output file path
    #[serde(default)]
    pub output_file: Option<String>,
}

fn default_style() -> String { "digital_art".into() }

pub async fn generate(config: &Config, params: SketchToArtParams) -> Result<String, String> {
    let (b64, mime) = crate::load_image_as_base64(&params.sketch_image).await?;
    let parts = vec![
        serde_json::json!({"inline_data": {"mime_type": mime, "data": b64}}),
        serde_json::json!({"text": format!("Turn this rough sketch into a polished {} artwork: {}", params.style, params.description)}),
    ];
    let output = params.output_file.unwrap_or_else(|| "sketch_to_art_output.png".into());
    let image_bytes = crate::call_gemini_image(config, "gemini-3.1-flash-image-preview", parts, None, None).await?;
    crate::save_image(&image_bytes, &output).await
}
