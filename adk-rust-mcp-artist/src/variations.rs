//! artist_variations — Generate style variations of an existing image.

use adk_rust_mcp_common::Config;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct VariationsParams {
    /// Source image (file path or base64 data)
    pub image: String,
    /// Number of variations to generate (1-4)
    #[serde(default = "default_variations")]
    pub variations: u8,
    /// Output directory
    #[serde(default)]
    pub output_dir: Option<String>,
}

fn default_variations() -> u8 { 4 }

const STYLES: &[(&str, &str)] = &[
    ("oil_painting", "Recreate this image as a classical oil painting with visible brushstrokes and rich colors."),
    ("watercolor", "Recreate this image as a watercolor painting with soft edges and translucent washes."),
    ("pencil_sketch", "Recreate this image as a detailed pencil sketch with shading and cross-hatching."),
    ("pop_art", "Recreate this image in pop art style with bold outlines, bright colors, and halftone dots."),
];

pub async fn generate(config: &Config, params: VariationsParams) -> Result<String, String> {
    let (b64, mime) = crate::load_image_as_base64(&params.image).await?;
    let dir = params.output_dir.unwrap_or_else(|| "variations".into());
    tokio::fs::create_dir_all(&dir).await.ok();

    let count = (params.variations.min(4).max(1)) as usize;
    let mut outputs = Vec::new();

    for (i, (name, prompt)) in STYLES.iter().take(count).enumerate() {
        let parts = vec![
            serde_json::json!({"inline_data": {"mime_type": &mime, "data": &b64}}),
            serde_json::json!({"text": prompt}),
        ];
        let image_bytes = crate::call_gemini_image(config, "gemini-3.1-flash-image-preview", parts, None, None).await?;
        let path = format!("{}/variation_{}_{}.png", dir, i + 1, name);
        crate::save_image(&image_bytes, &path).await?;
        outputs.push(path);
    }

    Ok(format!("Generated {} variations: {}", outputs.len(), outputs.join(", ")))
}
