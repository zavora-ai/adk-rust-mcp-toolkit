//! Short-form vertical video generation with optional captions.

use adk_rust_mcp_common::Config;
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::Path;
use tempfile::TempDir;
use tokio::process::Command;
use tracing::info;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ShortGenerateParams {
    /// Video content description
    pub prompt: String,
    /// Text overlay caption (bottom of screen)
    #[serde(default)]
    pub caption: Option<String>,
    /// Duration in seconds (4-8)
    #[serde(default = "default_duration")]
    pub duration_seconds: u8,
    /// Generate audio with the video
    #[serde(default = "default_true")]
    pub generate_audio: bool,
    /// Output file path
    #[serde(default)]
    pub output_file: Option<String>,
}

fn default_duration() -> u8 { 8 }
fn default_true() -> bool { true }

pub async fn generate(config: &Config, params: ShortGenerateParams) -> Result<String, String> {
    let api_key = config.gemini_api_key.as_deref().ok_or("GEMINI_API_KEY required")?;
    let model = "veo-3.1-generate-preview";
    let url = format!("{}/models/{}:predictLongRunning", config.gemini_base_url(), model);
    let output_path = params.output_file.clone().unwrap_or_else(|| "short.mp4".to_string());

    // Step 1: Generate vertical video
    info!(prompt = %params.prompt, "Generating short video (9:16)");
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "instances": [{"prompt": params.prompt}],
        "parameters": {
            "aspectRatio": "9:16",
            "durationSeconds": params.duration_seconds
        }
    });

    let resp = client.post(&url)
        .header("x-goog-api-key", api_key)
        .json(&body).send().await.map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("Veo API error: {}", resp.text().await.unwrap_or_default()));
    }

    let lro: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let op_name = lro["name"].as_str().ok_or("No operation name")?;

    // Poll LRO
    let poll_url = format!("{}/{}", config.gemini_base_url(), op_name);
    let video_data = loop {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let poll_resp = client.get(&poll_url)
            .header("x-goog-api-key", api_key)
            .send().await.map_err(|e| e.to_string())?;
        let status: serde_json::Value = poll_resp.json().await.map_err(|e| e.to_string())?;

        if status["done"].as_bool().unwrap_or(false) {
            if let Some(err) = status.get("error") {
                return Err(format!("Generation failed: {}", err));
            }
            let uri = status.pointer("/response/generateVideoResponse/generatedSamples/0/video/uri")
                .and_then(|v| v.as_str()).ok_or("No video URI")?;
            let dl = client.get(uri).header("x-goog-api-key", api_key)
                .send().await.map_err(|e| e.to_string())?;
            break dl.bytes().await.map(|b| b.to_vec()).map_err(|e| e.to_string())?;
        }
    };

    // Step 2: Add caption overlay if provided
    if let Some(parent) = Path::new(&output_path).parent() {
        if !parent.as_os_str().is_empty() { tokio::fs::create_dir_all(parent).await.ok(); }
    }

    if let Some(caption) = &params.caption {
        let tmp = TempDir::new().map_err(|e| e.to_string())?;
        let raw_path = tmp.path().join("raw.mp4");
        tokio::fs::write(&raw_path, &video_data).await.map_err(|e| e.to_string())?;

        let filter = format!(
            "drawtext=text='{}':fontsize=42:fontcolor=white:borderw=3:bordercolor=black:x=(w-text_w)/2:y=h-th-80",
            caption.replace('\'', "'\\''")
        );
        let output = Command::new("ffmpeg")
            .args(["-y", "-i", raw_path.to_str().unwrap(), "-vf", &filter, "-codec:a", "copy", &output_path])
            .output().await.map_err(|e| e.to_string())?;

        if !output.status.success() {
            // Fallback: save without caption if drawtext not available
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("drawtext") || stderr.contains("No such filter") {
                info!("drawtext filter unavailable, saving without caption");
                tokio::fs::write(&output_path, &video_data).await.map_err(|e| e.to_string())?;
            } else {
                return Err(format!("FFmpeg error: {}", stderr));
            }
        }
    } else {
        tokio::fs::write(&output_path, &video_data).await.map_err(|e| e.to_string())?;
    }

    info!(path = %output_path, "Short generated");
    Ok(format!("Short saved to: {}", output_path))
}
