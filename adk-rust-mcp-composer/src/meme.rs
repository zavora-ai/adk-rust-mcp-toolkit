//! Meme generation: Gemini image + text overlay.

use adk_rust_mcp_common::Config;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::Path;
use tempfile::TempDir;
use tokio::process::Command;
use tracing::info;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MemeGenerateParams {
    /// Image description (what the meme image should show)
    pub prompt: String,
    /// Top text
    #[serde(default)]
    pub top_text: Option<String>,
    /// Bottom text
    #[serde(default)]
    pub bottom_text: Option<String>,
    /// Font size
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    /// Output file path
    #[serde(default)]
    pub output_file: Option<String>,
}

fn default_font_size() -> u32 { 48 }

pub async fn generate(config: &Config, params: MemeGenerateParams) -> Result<String, String> {
    let api_key = config.gemini_api_key.as_deref().ok_or("GEMINI_API_KEY required")?;
    let output_path = params.output_file.clone().unwrap_or_else(|| "meme.png".to_string());

    // Step 1: Generate image with Gemini
    info!(prompt = %params.prompt, "Generating meme image");
    let model = "gemini-2.5-flash-image";
    let url = format!("{}/models/{}:generateContent", config.gemini_base_url(), model);

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "contents": [{"parts": [{"text": format!("Generate a meme image of: {}", params.prompt)}]}],
        "generationConfig": {
            "responseModalities": ["IMAGE", "TEXT"],
            "imageConfig": {"aspectRatio": "1:1"}
        }
    });

    let resp = client.post(&url)
        .header("x-goog-api-key", api_key)
        .json(&body).send().await.map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("Gemini API error: {}", resp.text().await.unwrap_or_default()));
    }

    let response: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    // Extract image data
    let image_data = response.pointer("/candidates/0/content/parts")
        .and_then(|parts| parts.as_array())
        .and_then(|parts| parts.iter().find_map(|p| p.pointer("/inlineData/data").and_then(|d| d.as_str())))
        .ok_or("No image in response")?;

    let image_bytes = BASE64.decode(image_data).map_err(|e| e.to_string())?;

    // Step 2: Add text overlay if provided
    if params.top_text.is_some() || params.bottom_text.is_some() {
        let tmp = TempDir::new().map_err(|e| e.to_string())?;
        let raw_path = tmp.path().join("raw.png");
        tokio::fs::write(&raw_path, &image_bytes).await.map_err(|e| e.to_string())?;

        if let Some(parent) = Path::new(&output_path).parent() {
            if !parent.as_os_str().is_empty() { tokio::fs::create_dir_all(parent).await.ok(); }
        }

        let mut filters = Vec::new();
        let fs = params.font_size;

        if let Some(ref top) = params.top_text {
            filters.push(format!(
                "drawtext=text='{}':fontsize={}:fontcolor=white:borderw=4:bordercolor=black:x=(w-text_w)/2:y=20",
                top.replace('\'', "'\\''"), fs
            ));
        }
        if let Some(ref bottom) = params.bottom_text {
            filters.push(format!(
                "drawtext=text='{}':fontsize={}:fontcolor=white:borderw=4:bordercolor=black:x=(w-text_w)/2:y=h-th-20",
                bottom.replace('\'', "'\\''"), fs
            ));
        }

        let filter_str = filters.join(",");
        let output = Command::new("ffmpeg")
            .args(["-y", "-i", raw_path.to_str().unwrap(), "-vf", &filter_str, &output_path])
            .output().await.map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(format!("FFmpeg error: {}", String::from_utf8_lossy(&output.stderr)));
        }
    } else {
        if let Some(parent) = Path::new(&output_path).parent() {
            if !parent.as_os_str().is_empty() { tokio::fs::create_dir_all(parent).await.ok(); }
        }
        tokio::fs::write(&output_path, &image_bytes).await.map_err(|e| e.to_string())?;
    }

    info!(path = %output_path, "Meme generated");
    Ok(format!("Meme saved to: {}", output_path))
}
