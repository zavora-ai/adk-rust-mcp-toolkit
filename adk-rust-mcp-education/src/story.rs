//! Story generation: illustrated children's stories with narration → video.

use adk_rust_mcp_common::Config;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::Path;
use tempfile::TempDir;
use tokio::process::Command;
use tracing::info;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StoryParams {
    /// Story idea or theme
    pub prompt: String,
    /// Number of pages/scenes (3-12)
    #[serde(default = "default_pages")]
    pub pages: u8,
    /// Target age range
    #[serde(default = "default_age")]
    pub age_group: String,
    /// Art style: watercolor, cartoon, pixel_art, storybook
    #[serde(default = "default_style")]
    pub style: String,
    /// Narrator voice
    #[serde(default = "default_voice")]
    pub voice: String,
    /// Optional moral/lesson
    #[serde(default)]
    pub moral: Option<String>,
    /// Background music prompt
    #[serde(default = "default_music")]
    pub background_music: String,
    /// Output path (.mp4)
    pub output_file: String,
}

fn default_pages() -> u8 { 5 }
fn default_age() -> String { "5-7".into() }
fn default_style() -> String { "watercolor".into() }
fn default_voice() -> String { "Aoede".into() }
fn default_music() -> String { "gentle lullaby".into() }

pub async fn generate(config: &Config, params: StoryParams) -> Result<String, String> {
    let api_key = config.gemini_api_key.as_deref().ok_or("GEMINI_API_KEY required")?;
    let client = reqwest::Client::new();
    let base = config.gemini_base_url().to_string();
    let tmp = TempDir::new().map_err(|e| e.to_string())?;

    info!(prompt = %params.prompt, pages = params.pages, "Generating story");

    // Step 1: Generate story text page by page
    let moral_hint = params.moral.as_deref().map(|m| format!(" Include the moral: {m}.")).unwrap_or_default();
    let text_url = format!("{}/models/gemini-2.5-flash:generateContent", base);
    let text_body = serde_json::json!({
        "contents": [{"parts": [{"text": format!(
            "Write a {pages}-page children's story for ages {age} about: {prompt}.{moral} \
             Return ONLY a JSON array of objects: [{{\"page_text\": \"...\", \"image_desc\": \"...\"}}]. \
             page_text is narration (2-3 sentences). image_desc describes the illustration. No markdown.",
            pages = params.pages, age = params.age_group, prompt = params.prompt, moral = moral_hint
        )}]}],
        "generationConfig": {"responseMimeType": "application/json"}
    });

    let resp = client.post(&text_url).header("x-goog-api-key", api_key)
        .json(&text_body).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("Story gen error: {}", resp.text().await.unwrap_or_default()));
    }
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let text = json.pointer("/candidates/0/content/parts/0/text")
        .and_then(|t| t.as_str()).ok_or("No story text")?;
    let pages: Vec<serde_json::Value> = serde_json::from_str(text).map_err(|e| e.to_string())?;

    // Step 2: Generate images + TTS for each page
    let mut slide_tasks = Vec::new();
    for (i, page) in pages.iter().enumerate() {
        let page_text = page["page_text"].as_str().unwrap_or("").to_string();
        let image_desc = page["image_desc"].as_str().unwrap_or("").to_string();
        let style = params.style.clone();
        let age = params.age_group.clone();
        let voice = params.voice.clone();
        let c = client.clone();
        let key = api_key.to_string();
        let b = base.clone();
        let img_path = tmp.path().join(format!("page_{}.png", i));
        let audio_path = tmp.path().join(format!("narration_{}.pcm", i));

        slide_tasks.push(tokio::spawn(async move {
            // Image
            let img_url = format!("{}/models/gemini-2.5-flash-image:generateContent", b);
            let img_body = serde_json::json!({
                "contents": [{"parts": [{"text": format!(
                    "Children's book illustration, {style} style, for ages {age}: {image_desc}. \
                     Bright, friendly, no text overlay."
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

            // TTS
            let tts_url = format!("{}/models/gemini-2.5-flash-preview-tts:generateContent", b);
            let tts_body = serde_json::json!({
                "contents": [{"parts": [{"text": page_text}]}],
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
            "-t", &format!("{:.1}", dur + 2.0),
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

    // Background music
    if let Some(parent) = Path::new(&params.output_file).parent() {
        if !parent.as_os_str().is_empty() { tokio::fs::create_dir_all(parent).await.ok(); }
    }

    let music_url = format!("{}/models/lyria-3-clip-preview:generateContent", base);
    let music_body = serde_json::json!({
        "contents": [{"parts": [{"text": format!("{} Instrumental, gentle, for children.", params.background_music)}]}],
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
            "-filter_complex", "[1:a]volume=0.12[m];[0:a][m]amix=inputs=2:duration=first[a]",
            "-map", "0:v", "-map", "[a]", "-c:v", "copy", "-shortest",
            &params.output_file
        ]).output().await.map_err(|e| e.to_string())?;
    } else {
        tokio::fs::copy(&concat_output, &params.output_file).await.map_err(|e| e.to_string())?;
    }

    info!(path = %params.output_file, pages = pages.len(), "Story generated");
    Ok(format!("Story saved to: {} ({} pages)", params.output_file, pages.len()))
}
