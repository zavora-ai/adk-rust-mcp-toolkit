//! Whiteboard generation: educational diagrams/math via Gemini image gen.

use adk_rust_mcp_common::Config;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::Path;
use tempfile::TempDir;
use tokio::process::Command;
use tracing::info;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WhiteboardParams {
    /// What to draw (math problem, diagram, concept map)
    pub content: String,
    /// Style: whiteboard, blackboard, notebook, colorful
    #[serde(default = "default_style")]
    pub style: String,
    /// Show step-by-step solution (for math)
    #[serde(default)]
    pub show_steps: bool,
    /// Generate TTS explaining the board
    #[serde(default)]
    pub narration: bool,
    /// Output file path (.png or .mp4 if narrated)
    #[serde(default)]
    pub output_file: Option<String>,
}

fn default_style() -> String { "whiteboard".into() }

pub async fn generate(config: &Config, params: WhiteboardParams) -> Result<String, String> {
    let api_key = config.gemini_api_key.as_deref().ok_or("GEMINI_API_KEY required")?;
    let client = reqwest::Client::new();
    let base = config.gemini_base_url();

    let style_desc = match params.style.as_str() {
        "blackboard" => "drawn with chalk on a dark green blackboard",
        "notebook" => "written neatly in a lined notebook with pen",
        "colorful" => "drawn with bright colorful markers, kid-friendly",
        _ => "drawn with dry-erase markers on a white whiteboard",
    };

    let steps_hint = if params.show_steps { " Show each step clearly numbered." } else { "" };

    let prompt = format!(
        "Educational diagram {style_desc}: {content}.{steps_hint} \
         Clear labels, large readable text, hand-drawn style. Educational and easy to understand.",
        style_desc = style_desc, content = params.content, steps_hint = steps_hint
    );

    info!(content = %params.content, style = %params.style, "Generating whiteboard");

    let url = format!("{}/models/gemini-2.5-flash-image:generateContent", base);
    let body = serde_json::json!({
        "contents": [{"parts": [{"text": prompt}]}],
        "generationConfig": {"responseModalities": ["IMAGE", "TEXT"]}
    });

    let resp = client.post(&url).header("x-goog-api-key", api_key)
        .json(&body).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("Gemini API error: {}", resp.text().await.unwrap_or_default()));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let img_data = json.pointer("/candidates/0/content/parts")
        .and_then(|p| p.as_array())
        .and_then(|parts| parts.iter().find_map(|p| p.pointer("/inlineData/data").and_then(|d| d.as_str())))
        .ok_or("No image in response")?;
    let img_bytes = BASE64.decode(img_data).map_err(|e| e.to_string())?;

    if !params.narration {
        let out = params.output_file.unwrap_or_else(|| "whiteboard.png".into());
        if let Some(parent) = Path::new(&out).parent() {
            if !parent.as_os_str().is_empty() { tokio::fs::create_dir_all(parent).await.ok(); }
        }
        tokio::fs::write(&out, &img_bytes).await.map_err(|e| e.to_string())?;
        info!(path = %out, "Whiteboard saved");
        return Ok(format!("Whiteboard saved to: {}", out));
    }

    // Narrated: generate TTS and combine with image into video
    let tmp = TempDir::new().map_err(|e| e.to_string())?;
    let img_path = tmp.path().join("board.png");
    tokio::fs::write(&img_path, &img_bytes).await.map_err(|e| e.to_string())?;

    let tts_url = format!("{}/models/gemini-2.5-flash-preview-tts:generateContent", base);
    let tts_body = serde_json::json!({
        "contents": [{"parts": [{"text": format!("Explain this: {}", params.content)}]}],
        "generationConfig": {
            "responseModalities": ["AUDIO"],
            "speechConfig": {"voiceConfig": {"prebuiltVoiceConfig": {"voiceName": "Kore"}}}
        }
    });
    let tts_resp = client.post(&tts_url).header("x-goog-api-key", api_key)
        .json(&tts_body).send().await.map_err(|e| e.to_string())?;
    let tts_json: serde_json::Value = tts_resp.json().await.map_err(|e| e.to_string())?;
    let audio_data = tts_json.pointer("/candidates/0/content/parts/0/inlineData/data")
        .and_then(|d| d.as_str()).ok_or("No audio in TTS response")?;
    let pcm_bytes = BASE64.decode(audio_data).map_err(|e| e.to_string())?;

    let pcm_path = tmp.path().join("narration.pcm");
    let wav_path = tmp.path().join("narration.wav");
    tokio::fs::write(&pcm_path, &pcm_bytes).await.map_err(|e| e.to_string())?;

    Command::new("ffmpeg").args([
        "-y", "-f", "s16le", "-ar", "24000", "-ac", "1", "-i", pcm_path.to_str().unwrap(),
        wav_path.to_str().unwrap()
    ]).output().await.map_err(|e| e.to_string())?;

    let out = params.output_file.unwrap_or_else(|| "whiteboard.mp4".into());
    if let Some(parent) = Path::new(&out).parent() {
        if !parent.as_os_str().is_empty() { tokio::fs::create_dir_all(parent).await.ok(); }
    }

    Command::new("ffmpeg").args([
        "-y", "-loop", "1", "-i", img_path.to_str().unwrap(),
        "-i", wav_path.to_str().unwrap(),
        "-c:v", "libx264", "-tune", "stillimage", "-c:a", "aac",
        "-pix_fmt", "yuv420p", "-shortest", &out
    ]).output().await.map_err(|e| e.to_string())?;

    info!(path = %out, "Narrated whiteboard saved");
    Ok(format!("Narrated whiteboard saved to: {}", out))
}
