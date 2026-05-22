//! Explainer generation: step-by-step animated explanations → video.

use adk_rust_mcp_common::Config;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::Path;
use tempfile::TempDir;
use tokio::process::Command;
use tracing::info;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExplainerParams {
    /// Concept to explain
    pub topic: String,
    /// Number of steps (0 = auto)
    #[serde(default)]
    pub steps: u8,
    /// Target age range
    #[serde(default = "default_age")]
    pub age_group: String,
    /// Style: diagram, cartoon, realistic, infographic
    #[serde(default = "default_style")]
    pub style: String,
    /// Narrator voice
    #[serde(default = "default_voice")]
    pub voice: String,
    /// Pace: slow, normal, fast
    #[serde(default = "default_pace")]
    pub pace: String,
    /// Add summary slide at end
    #[serde(default = "default_true")]
    pub include_summary: bool,
    /// Optional background music prompt
    #[serde(default)]
    pub background_music: Option<String>,
    /// Output path (.mp4)
    pub output_file: String,
}

fn default_age() -> String { "8-10".into() }
fn default_style() -> String { "diagram".into() }
fn default_voice() -> String { "Kore".into() }
fn default_pace() -> String { "normal".into() }
fn default_true() -> bool { true }

pub async fn generate(config: &Config, params: ExplainerParams) -> Result<String, String> {
    let api_key = config.gemini_api_key.as_deref().ok_or("GEMINI_API_KEY required")?;
    let client = reqwest::Client::new();
    let base = config.gemini_base_url().to_string();
    let tmp = TempDir::new().map_err(|e| e.to_string())?;

    info!(topic = %params.topic, "Generating explainer");

    // Step 1: Break topic into steps
    let steps_hint = if params.steps > 0 { format!("exactly {} steps", params.steps) } else { "an appropriate number of steps (3-8)".into() };
    let text_url = format!("{}/models/gemini-2.5-flash:generateContent", base);
    let text_body = serde_json::json!({
        "contents": [{"parts": [{"text": format!(
            "Break down \"{}\" into {} for children aged {}. \
             Return ONLY a JSON array: [{{\"title\": \"Step N: ...\", \"explanation\": \"...\", \"image_desc\": \"...\"}}]. \
             explanation is 2-3 sentences for narration. image_desc describes a {}-style diagram for this step. No markdown.",
            params.topic, steps_hint, params.age_group, params.style
        )}]}],
        "generationConfig": {"responseMimeType": "application/json"}
    });

    let resp = client.post(&text_url).header("x-goog-api-key", api_key)
        .json(&text_body).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("Explainer gen error: {}", resp.text().await.unwrap_or_default()));
    }
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let text = json.pointer("/candidates/0/content/parts/0/text")
        .and_then(|t| t.as_str()).ok_or("No steps text")?;
    let steps: Vec<serde_json::Value> = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let total_steps = steps.len();

    // Step 2: Generate images + TTS for each step
    let mut slide_tasks = Vec::new();
    for (i, step) in steps.iter().enumerate() {
        let explanation = step["explanation"].as_str().unwrap_or("").to_string();
        let image_desc = step["image_desc"].as_str().unwrap_or("").to_string();
        let title = step["title"].as_str().unwrap_or("").to_string();
        let style = params.style.clone();
        let age = params.age_group.clone();
        let voice = params.voice.clone();
        let c = client.clone();
        let key = api_key.to_string();
        let b = base.clone();
        let img_path = tmp.path().join(format!("step_{}.png", i));
        let audio_path = tmp.path().join(format!("narration_{}.pcm", i));

        slide_tasks.push(tokio::spawn(async move {
            let img_url = format!("{}/models/gemini-2.5-flash-image:generateContent", b);
            let img_body = serde_json::json!({
                "contents": [{"parts": [{"text": format!(
                    "Educational {style} for children aged {age}. {title}: {image_desc}. \
                     Step {n} of {total}. Clear labels, numbered, educational.",
                    style = style, age = age, title = title, image_desc = image_desc,
                    n = i + 1, total = total_steps
                )}]}],
                "generationConfig": {"responseModalities": ["IMAGE", "TEXT"]}
            });
            let img_resp = c.post(&img_url).header("x-goog-api-key", &key)
                .json(&img_body).send().await.map_err(|e| e.to_string())?;
            let img_json: serde_json::Value = img_resp.json().await.map_err(|e| e.to_string())?;
            let data = img_json.pointer("/candidates/0/content/parts")
                .and_then(|p| p.as_array())
                .and_then(|parts| parts.iter().find_map(|p| p.pointer("/inlineData/data").and_then(|d| d.as_str())))
                .ok_or("No image data".to_string())?;
            tokio::fs::write(&img_path, BASE64.decode(data).map_err(|e| e.to_string())?)
                .await.map_err(|e| e.to_string())?;

            let tts_url = format!("{}/models/gemini-2.5-flash-preview-tts:generateContent", b);
            let tts_body = serde_json::json!({
                "contents": [{"parts": [{"text": format!("{title}. {explanation}")}]}],
                "generationConfig": {
                    "responseModalities": ["AUDIO"],
                    "speechConfig": {"voiceConfig": {"prebuiltVoiceConfig": {"voiceName": voice}}}
                }
            });
            let tts_resp = c.post(&tts_url).header("x-goog-api-key", &key)
                .json(&tts_body).send().await.map_err(|e| e.to_string())?;
            let tts_json: serde_json::Value = tts_resp.json().await.map_err(|e| e.to_string())?;
            let audio = tts_json.pointer("/candidates/0/content/parts/0/inlineData/data")
                .and_then(|d| d.as_str()).ok_or("No audio data".to_string())?;
            tokio::fs::write(&audio_path, BASE64.decode(audio).map_err(|e| e.to_string())?)
                .await.map_err(|e| e.to_string())?;

            Ok::<(String, String), String>((img_path.to_string_lossy().into(), audio_path.to_string_lossy().into()))
        }));
    }

    let mut slide_files = Vec::new();
    for task in slide_tasks {
        slide_files.push(task.await.map_err(|e| e.to_string())??);
    }

    // Step 3: Assemble video segments
    let pace_extra: f64 = match params.pace.as_str() {
        "slow" => 3.0,
        "fast" => 0.5,
        _ => 1.5,
    };

    let mut segment_paths = Vec::new();
    for (i, (img_path, pcm_path)) in slide_files.iter().enumerate() {
        let wav_path = tmp.path().join(format!("narration_{}.wav", i));
        let seg_path = tmp.path().join(format!("segment_{}.mp4", i));

        Command::new("ffmpeg").args([
            "-y", "-f", "s16le", "-ar", "24000", "-ac", "1", "-i", pcm_path,
            wav_path.to_str().unwrap()
        ]).output().await.map_err(|e| e.to_string())?;

        let probe = Command::new("ffprobe").args([
            "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0",
            wav_path.to_str().unwrap()
        ]).output().await.map_err(|e| e.to_string())?;
        let dur: f64 = String::from_utf8_lossy(&probe.stdout).trim().parse().unwrap_or(5.0);

        Command::new("ffmpeg").args([
            "-y", "-loop", "1", "-i", img_path,
            "-i", wav_path.to_str().unwrap(),
            "-c:v", "libx264", "-tune", "stillimage", "-c:a", "aac",
            "-b:a", "192k", "-pix_fmt", "yuv420p",
            "-t", &format!("{:.1}", dur + pace_extra),
            "-shortest", seg_path.to_str().unwrap()
        ]).output().await.map_err(|e| e.to_string())?;

        segment_paths.push(seg_path);
    }

    // Concatenate
    let concat_file = tmp.path().join("concat.txt");
    let concat_content: String = segment_paths.iter()
        .map(|p| format!("file '{}'", p.display())).collect::<Vec<_>>().join("\n");
    tokio::fs::write(&concat_file, &concat_content).await.map_err(|e| e.to_string())?;

    let concat_output = tmp.path().join("concat.mp4");
    Command::new("ffmpeg").args([
        "-y", "-f", "concat", "-safe", "0", "-i", concat_file.to_str().unwrap(),
        "-c", "copy", concat_output.to_str().unwrap()
    ]).output().await.map_err(|e| e.to_string())?;

    // Output with optional music
    if let Some(parent) = Path::new(&params.output_file).parent() {
        if !parent.as_os_str().is_empty() { tokio::fs::create_dir_all(parent).await.ok(); }
    }

    if let Some(ref music_prompt) = params.background_music {
        let music_url = format!("{}/models/lyria-3-clip-preview:generateContent", base);
        let music_body = serde_json::json!({
            "contents": [{"parts": [{"text": format!("{} Instrumental, educational.", music_prompt)}]}],
            "generationConfig": {"responseModalities": ["AUDIO", "TEXT"]}
        });
        let music_resp = client.post(&music_url).header("x-goog-api-key", api_key)
            .json(&music_body).send().await.map_err(|e| e.to_string())?;
        let music_json: serde_json::Value = music_resp.json().await.map_err(|e| e.to_string())?;

        if let Some(music_data) = music_json.pointer("/candidates/0/content/parts")
            .and_then(|p| p.as_array())
            .and_then(|parts| parts.iter().find_map(|p| p.pointer("/inlineData/data").and_then(|d| d.as_str())))
        {
            let music_path = tmp.path().join("music.mp3");
            tokio::fs::write(&music_path, BASE64.decode(music_data).map_err(|e| e.to_string())?)
                .await.map_err(|e| e.to_string())?;

            Command::new("ffmpeg").args([
                "-y", "-i", concat_output.to_str().unwrap(),
                "-stream_loop", "-1", "-i", music_path.to_str().unwrap(),
                "-filter_complex", "[1:a]volume=0.1[m];[0:a][m]amix=inputs=2:duration=first[a]",
                "-map", "0:v", "-map", "[a]", "-c:v", "copy", "-shortest",
                &params.output_file
            ]).output().await.map_err(|e| e.to_string())?;
        } else {
            tokio::fs::copy(&concat_output, &params.output_file).await.map_err(|e| e.to_string())?;
        }
    } else {
        tokio::fs::copy(&concat_output, &params.output_file).await.map_err(|e| e.to_string())?;
    }

    info!(path = %params.output_file, steps = total_steps, "Explainer generated");
    Ok(format!("Explainer saved to: {} ({} steps)", params.output_file, total_steps))
}
