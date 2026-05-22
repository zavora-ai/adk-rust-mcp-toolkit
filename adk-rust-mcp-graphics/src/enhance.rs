//! graphics_enhance — Enhance image quality/details.

use adk_rust_mcp_common::Config;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EnhanceParams {
    /// Source image (file path or base64 data)
    pub image: String,
    /// Enhancement type: sharpen, denoise, upscale, color_correct, hdr
    pub enhancement: String,
    /// Output file path
    #[serde(default)]
    pub output_file: Option<String>,
}

pub async fn generate(config: &Config, params: EnhanceParams) -> Result<String, String> {
    let (b64, mime) = crate::load_image_as_base64(&params.image).await?;
    let instruction = match params.enhancement.as_str() {
        "sharpen" => "Sharpen this image. Enhance edges and fine details while preserving the original content.",
        "denoise" => "Remove noise from this image. Clean up grain while preserving details.",
        "upscale" => "Upscale this image. Enhance resolution and add fine details.",
        "color_correct" => "Color correct this image. Fix white balance, improve contrast and saturation.",
        "hdr" => "Apply HDR enhancement to this image. Expand dynamic range, bring out shadow and highlight details.",
        _ => "Enhance this image. Improve quality while preserving the original content.",
    };
    let parts = vec![
        serde_json::json!({"inline_data": {"mime_type": mime, "data": b64}}),
        serde_json::json!({"text": instruction}),
    ];
    let output = params.output_file.unwrap_or_else(|| "enhanced_output.png".into());
    let image_bytes = crate::call_gemini_image(config, "gemini-3.1-flash-image-preview", parts, None).await?;
    crate::save_image(&image_bytes, &output).await
}
