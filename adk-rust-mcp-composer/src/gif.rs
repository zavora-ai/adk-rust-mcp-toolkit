//! GIF generation: Veo video → FFmpeg GIF conversion.

use adk_rust_mcp_common::Config;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::Path;
use tempfile::TempDir;
use tokio::process::Command;
use tracing::info;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GifGenerateParams {
    /// Text prompt describing the animation
    pub prompt: String,
    /// Duration in seconds (4-8)
    #[serde(default = "default_duration")]
    pub duration_seconds: u8,
    /// GIF frame rate
    #[serde(default = "default_fps")]
    pub fps: u8,
    /// Output width in pixels
    #[serde(default = "default_width")]
    pub width: u32,
    /// Output file path
    #[serde(default)]
    pub output_file: Option<String>,
}

fn default_duration() -> u8 { 4 }
fn default_fps() -> u8 { 12 }
fn default_width() -> u32 { 480 }

pub async fn generate(config: &Config, params: GifGenerateParams) -> Result<String, String> {
    let tmp = TempDir::new().map_err(|e| format!("Temp dir failed: {}", e))?;
    let video_path = tmp.path().join("input.mp4");
    let gif_path = params.output_file.clone()
        .unwrap_or_else(|| "output.gif".to_string());

    // Step 1: Generate video with Veo
    info!(prompt = %params.prompt, "Generating video for GIF");
    let video_data = generate_video(config, &params.prompt, params.duration_seconds).await?;
    tokio::fs::write(&video_path, &video_data).await
        .map_err(|e| format!("Write video failed: {}", e))?;

    // Step 2: Convert to GIF with FFmpeg
    info!("Converting video to GIF");
    if let Some(parent) = Path::new(&gif_path).parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
    }

    let output = Command::new("ffmpeg")
        .args([
            "-y", "-i", video_path.to_str().unwrap(),
            "-vf", &format!("fps={},scale={}:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse", params.fps, params.width),
            "-loop", "0",
            &gif_path,
        ])
        .output()
        .await
        .map_err(|e| format!("FFmpeg failed: {}", e))?;

    if !output.status.success() {
        return Err(format!("FFmpeg error: {}", String::from_utf8_lossy(&output.stderr)));
    }

    info!(path = %gif_path, "GIF generated");
    Ok(format!("GIF saved to: {}", gif_path))
}

async fn generate_video(config: &Config, prompt: &str, duration: u8) -> Result<Vec<u8>, String> {
    let api_key = config.gemini_api_key.as_deref()
        .ok_or("GEMINI_API_KEY required")?;
    let model = "veo-3.1-lite-generate-preview";
    let url = format!("{}/models/{}:predictLongRunning", config.gemini_base_url(), model);

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "instances": [{"prompt": prompt}],
        "parameters": {"aspectRatio": "16:9", "durationSeconds": duration}
    });

    let resp = client.post(&url)
        .header("x-goog-api-key", api_key)
        .json(&body).send().await
        .map_err(|e| format!("API request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Veo API error: {}", resp.text().await.unwrap_or_default()));
    }

    let lro: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let op_name = lro["name"].as_str().ok_or("No operation name")?;

    // Poll LRO
    let poll_url = format!("{}/{}", config.gemini_base_url(), op_name);
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let poll_resp = client.get(&poll_url)
            .header("x-goog-api-key", api_key)
            .send().await.map_err(|e| e.to_string())?;
        let status: serde_json::Value = poll_resp.json().await.map_err(|e| e.to_string())?;

        if status["done"].as_bool().unwrap_or(false) {
            if let Some(err) = status.get("error") {
                return Err(format!("Generation failed: {}", err));
            }
            // Get download URI
            let uri = status.pointer("/response/generateVideoResponse/generatedSamples/0/video/uri")
                .and_then(|v| v.as_str())
                .ok_or("No video URI in response")?;

            // Download
            let dl = client.get(uri)
                .header("x-goog-api-key", api_key)
                .send().await.map_err(|e| e.to_string())?;
            return dl.bytes().await.map(|b| b.to_vec()).map_err(|e| e.to_string());
        }
    }
}
