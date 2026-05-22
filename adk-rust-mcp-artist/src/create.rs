//! artist_create — Create art from text in a specific style.

use adk_rust_mcp_common::Config;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ArtistCreateParams {
    /// Text prompt describing the artwork
    pub prompt: String,
    /// Art style: oil_painting, watercolor, impressionist, abstract, pop_art, pencil_sketch, digital_art, pixel_art
    #[serde(default = "default_style")]
    pub style: String,
    /// Aspect ratio (e.g. "1:1", "16:9", "9:16")
    #[serde(default = "default_aspect")]
    pub aspect_ratio: String,
    /// Resolution: "1K", "2K", "4K"
    #[serde(default = "default_resolution")]
    pub resolution: String,
    /// Output file path
    #[serde(default)]
    pub output_file: Option<String>,
}

fn default_style() -> String { "digital_art".into() }
fn default_aspect() -> String { "1:1".into() }
fn default_resolution() -> String { "2K".into() }

pub async fn generate(config: &Config, params: ArtistCreateParams) -> Result<String, String> {
    let style_instruction = match params.style.as_str() {
        "oil_painting" => "in the style of a classical oil painting with visible brushstrokes and rich colors",
        "watercolor" => "in watercolor style with soft edges, translucent washes, and paper texture",
        "impressionist" => "in impressionist style with loose brushwork and emphasis on light",
        "abstract" => "in abstract art style with bold shapes, colors, and non-representational forms",
        "pop_art" => "in pop art style with bold outlines, bright colors, and halftone dots",
        "pencil_sketch" => "as a detailed pencil sketch with shading and cross-hatching",
        "digital_art" => "as polished digital art with clean lines and vibrant colors",
        "pixel_art" => "in pixel art style with visible pixels and retro aesthetic",
        _ => "as digital art",
    };

    let prompt = format!("Create artwork {}: {}", style_instruction, params.prompt);
    let parts = vec![serde_json::json!({"text": prompt})];
    let output = params.output_file.unwrap_or_else(|| "artist_output.png".into());

    let image_bytes = crate::call_gemini_image(
        config, "gemini-3.1-flash-image-preview",
        parts, Some(&params.aspect_ratio), Some(&params.resolution),
    ).await?;

    crate::save_image(&image_bytes, &output).await
}
