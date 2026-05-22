//! Presentation generation: images + TTS + music → video.

use adk_rust_mcp_common::Config;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::Path;
use tempfile::TempDir;
use tokio::process::Command;
use tracing::info;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PresentationGenerateParams {
    /// Array of slides
    pub slides: Vec<Slide>,
    /// Visual style
    #[serde(default = "default_style")]
    pub style: String,
    /// TTS voice name
    #[serde(default = "default_voice")]
    pub voice: String,
    /// Background music prompt
    #[serde(default)]
    pub background_music: Option<String>,
    /// Music volume (0.0-1.0)
    #[serde(default = "default_music_vol")]
    pub music_volume: f32,
    /// Output file path
    pub output_file: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct Slide {
    /// Slide title
    pub title: String,
    /// Narration text
    pub content: String,
    /// Custom image prompt
    #[serde(default)]
    pub image_prompt: Option<String>,
}

fn default_style() -> String { "professional".into() }
fn default_voice() -> String { "Kore".into() }
fn default_music_vol() -> f32 { 0.15 }

pub async fn generate(config: &Config, params: PresentationGenerateParams) -> Result<String, String> {
    let api_key = config.gemini_api_key.as_deref().ok_or("GEMINI_API_KEY required")?;
    let client = reqwest::Client::new();
    let tmp = TempDir::new().map_err(|e| e.to_string())?;
    let base = config.gemini_base_url().to_string();

    info!(slides = params.slides.len(), "Generating presentation");

    // Generate images and TTS for each slide in parallel
    let mut slide_tasks = Vec::new();
    for (i, slide) in params.slides.iter().enumerate() {
        let img_prompt = slide.image_prompt.clone().unwrap_or_else(|| {
            format!("{} illustration for a presentation about: {}. {}", params.style, slide.title, slide.content)
        });
        let tts_text = format!("{}. {}", slide.title, slide.content);
        let voice = params.voice.clone();
        let c = client.clone();
        let key = api_key.to_string();
        let base_url = base.clone();
        let img_path = tmp.path().join(format!("slide_{}.png", i));
        let audio_path = tmp.path().join(format!("narration_{}.wav", i));

        slide_tasks.push(tokio::spawn(async move {
            // Generate image
            let img_url = format!("{}/models/gemini-2.5-flash-image:generateContent", base_url);
            let img_body = serde_json::json!({
                "contents": [{"parts": [{"text": img_prompt}]}],
                "generationConfig": {"responseModalities": ["IMAGE", "TEXT"]}
            });
            let img_resp = c.post(&img_url).header("x-goog-api-key", &key)
                .json(&img_body).send().await.map_err(|e| e.to_string())?;
            let img_json: serde_json::Value = img_resp.json().await.map_err(|e| e.to_string())?;
            let img_data = img_json.pointer("/candidates/0/content/parts")
                .and_then(|p| p.as_array())
                .and_then(|parts| parts.iter().find_map(|p| p.pointer("/inlineData/data").and_then(|d| d.as_str())))
                .ok_or("No image data".to_string())?;
            let img_bytes = BASE64.decode(img_data).map_err(|e| e.to_string())?;
            tokio::fs::write(&img_path, &img_bytes).await.map_err(|e| e.to_string())?;

            // Generate TTS
            let tts_url = format!("{}/models/gemini-2.5-flash-preview-tts:generateContent", base_url);
            let tts_body = serde_json::json!({
                "contents": [{"parts": [{"text": tts_text}]}],
                "generationConfig": {
                    "responseModalities": ["AUDIO"],
                    "speechConfig": {"voiceConfig": {"prebuiltVoiceConfig": {"voiceName": voice}}}
                }
            });
            let tts_resp = c.post(&tts_url).header("x-goog-api-key", &key)
                .json(&tts_body).send().await.map_err(|e| e.to_string())?;
            let tts_json: serde_json::Value = tts_resp.json().await.map_err(|e| e.to_string())?;
            let audio_data = tts_json.pointer("/candidates/0/content/parts/0/inlineData/data")
                .and_then(|d| d.as_str()).ok_or("No audio data".to_string())?;
            let audio_bytes = BASE64.decode(audio_data).map_err(|e| e.to_string())?;
            // Write raw PCM, convert to WAV with ffmpeg later
            tokio::fs::write(&audio_path, &audio_bytes).await.map_err(|e| e.to_string())?;

            Ok::<(String, String), String>((img_path.to_string_lossy().into(), audio_path.to_string_lossy().into()))
        }));
    }

    // Collect results
    let mut slide_files = Vec::new();
    for task in slide_tasks {
        slide_files.push(task.await.map_err(|e| e.to_string())??);
    }

    // Convert PCM narrations to WAV and create video segments
    let mut segment_paths = Vec::new();
    for (i, (img_path, pcm_path)) in slide_files.iter().enumerate() {
        let wav_path = tmp.path().join(format!("narration_{}.converted.wav", i));
        let seg_path = tmp.path().join(format!("segment_{}.mp4", i));

        // PCM to WAV
        Command::new("ffmpeg").args([
            "-y", "-f", "s16le", "-ar", "24000", "-ac", "1", "-i", pcm_path,
            wav_path.to_str().unwrap()
        ]).output().await.map_err(|e| e.to_string())?;

        // Get audio duration
        let probe = Command::new("ffprobe").args([
            "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0",
            wav_path.to_str().unwrap()
        ]).output().await.map_err(|e| e.to_string())?;
        let dur: f64 = String::from_utf8_lossy(&probe.stdout).trim().parse().unwrap_or(5.0);
        let slide_dur = dur + 1.0; // extra second of padding

        // Image + audio → video segment
        Command::new("ffmpeg").args([
            "-y", "-loop", "1", "-i", img_path,
            "-i", wav_path.to_str().unwrap(),
            "-c:v", "libx264", "-tune", "stillimage", "-c:a", "aac",
            "-b:a", "192k", "-pix_fmt", "yuv420p",
            "-t", &format!("{:.1}", slide_dur),
            "-shortest", seg_path.to_str().unwrap()
        ]).output().await.map_err(|e| e.to_string())?;

        segment_paths.push(seg_path);
    }

    // Concatenate segments
    let concat_file = tmp.path().join("concat.txt");
    let concat_content: String = segment_paths.iter()
        .map(|p| format!("file '{}'", p.display()))
        .collect::<Vec<_>>().join("\n");
    tokio::fs::write(&concat_file, &concat_content).await.map_err(|e| e.to_string())?;

    if let Some(parent) = Path::new(&params.output_file).parent() {
        if !parent.as_os_str().is_empty() { tokio::fs::create_dir_all(parent).await.ok(); }
    }

    let concat_output = tmp.path().join("concat.mp4");
    Command::new("ffmpeg").args([
        "-y", "-f", "concat", "-safe", "0", "-i", concat_file.to_str().unwrap(),
        "-c", "copy", concat_output.to_str().unwrap()
    ]).output().await.map_err(|e| e.to_string())?;

    // Optional: add background music
    if let Some(ref music_prompt) = params.background_music {
        let music_url = format!("{}/models/lyria-3-clip-preview:generateContent", base);
        let music_body = serde_json::json!({
            "contents": [{"parts": [{"text": format!("{} Instrumental only.", music_prompt)}]}],
            "generationConfig": {"responseModalities": ["AUDIO", "TEXT"]}
        });
        let music_resp = client.post(&music_url).header("x-goog-api-key", api_key)
            .json(&music_body).send().await.map_err(|e| e.to_string())?;
        let music_json: serde_json::Value = music_resp.json().await.map_err(|e| e.to_string())?;

        if let Some(music_data) = music_json.pointer("/candidates/0/content/parts")
            .and_then(|p| p.as_array())
            .and_then(|parts| parts.iter().find_map(|p| p.pointer("/inlineData/data").and_then(|d| d.as_str())))
        {
            let music_bytes = BASE64.decode(music_data).map_err(|e| e.to_string())?;
            let music_path = tmp.path().join("music.mp3");
            tokio::fs::write(&music_path, &music_bytes).await.map_err(|e| e.to_string())?;

            // Mix music with video
            let vol = params.music_volume;
            Command::new("ffmpeg").args([
                "-y", "-i", concat_output.to_str().unwrap(),
                "-stream_loop", "-1", "-i", music_path.to_str().unwrap(),
                "-filter_complex", &format!("[1:a]volume={}[m];[0:a][m]amix=inputs=2:duration=first[a]", vol),
                "-map", "0:v", "-map", "[a]", "-c:v", "copy", "-shortest",
                &params.output_file
            ]).output().await.map_err(|e| e.to_string())?;
        } else {
            tokio::fs::copy(&concat_output, &params.output_file).await.map_err(|e| e.to_string())?;
        }
    } else {
        tokio::fs::copy(&concat_output, &params.output_file).await.map_err(|e| e.to_string())?;
    }

    info!(path = %params.output_file, slides = params.slides.len(), "Presentation generated");
    Ok(format!("Presentation saved to: {} ({} slides)", params.output_file, params.slides.len()))
}
