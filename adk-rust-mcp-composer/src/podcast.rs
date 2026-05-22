//! Podcast generation: multi-speaker TTS + background music.

use adk_rust_mcp_common::Config;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::Path;
use tempfile::TempDir;
use tokio::process::Command;
use tracing::info;

const VOICES: &[&str] = &["Kore", "Puck", "Zephyr", "Charon", "Aoede", "Fenrir"];

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PodcastGenerateParams {
    /// Dialogue script segments
    pub script: Vec<ScriptSegment>,
    /// Background music prompt
    #[serde(default)]
    pub background_music: Option<String>,
    /// Music volume (0.0-1.0)
    #[serde(default = "default_music_vol")]
    pub music_volume: f32,
    /// Add 3s music intro
    #[serde(default)]
    pub intro_music: bool,
    /// Output file path
    pub output_file: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScriptSegment {
    /// Speaker name
    pub speaker: String,
    /// What they say
    pub text: String,
    /// Voice name (auto-assigned if omitted)
    #[serde(default)]
    pub voice: Option<String>,
    /// Delivery style (e.g., "cheerful", "serious")
    #[serde(default)]
    pub style: Option<String>,
}

fn default_music_vol() -> f32 { 0.1 }

pub async fn generate(config: &Config, params: PodcastGenerateParams) -> Result<String, String> {
    let api_key = config.gemini_api_key.as_deref().ok_or("GEMINI_API_KEY required")?;
    let client = reqwest::Client::new();
    let tmp = TempDir::new().map_err(|e| e.to_string())?;
    let base = config.gemini_base_url().to_string();

    info!(segments = params.script.len(), "Generating podcast");

    // Auto-assign voices to speakers
    let mut speaker_voices: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut voice_idx = 0;
    for seg in &params.script {
        if !speaker_voices.contains_key(&seg.speaker) {
            let voice = seg.voice.clone().unwrap_or_else(|| {
                let v = VOICES[voice_idx % VOICES.len()].to_string();
                voice_idx += 1;
                v
            });
            speaker_voices.insert(seg.speaker.clone(), voice);
        }
    }

    // Generate TTS for each segment
    let mut audio_paths = Vec::new();
    for (i, seg) in params.script.iter().enumerate() {
        let voice = speaker_voices.get(&seg.speaker).unwrap();
        let prompt = if let Some(ref style) = seg.style {
            format!("Say in a {} tone: {}", style, seg.text)
        } else {
            seg.text.clone()
        };

        let tts_url = format!("{}/models/gemini-2.5-flash-preview-tts:generateContent", base);
        let tts_body = serde_json::json!({
            "contents": [{"parts": [{"text": prompt}]}],
            "generationConfig": {
                "responseModalities": ["AUDIO"],
                "speechConfig": {"voiceConfig": {"prebuiltVoiceConfig": {"voiceName": voice}}}
            }
        });

        let resp = client.post(&tts_url).header("x-goog-api-key", api_key)
            .json(&tts_body).send().await.map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("TTS failed for segment {}: {}", i, resp.text().await.unwrap_or_default()));
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        let audio_data = json.pointer("/candidates/0/content/parts/0/inlineData/data")
            .and_then(|d| d.as_str()).ok_or(format!("No audio for segment {}", i))?;
        let audio_bytes = BASE64.decode(audio_data).map_err(|e| e.to_string())?;

        // Save as PCM then convert to WAV
        let pcm_path = tmp.path().join(format!("seg_{}.pcm", i));
        let wav_path = tmp.path().join(format!("seg_{}.wav", i));
        tokio::fs::write(&pcm_path, &audio_bytes).await.map_err(|e| e.to_string())?;

        Command::new("ffmpeg").args([
            "-y", "-f", "s16le", "-ar", "24000", "-ac", "1", "-i", pcm_path.to_str().unwrap(),
            wav_path.to_str().unwrap()
        ]).output().await.map_err(|e| e.to_string())?;

        audio_paths.push(wav_path);
    }

    // Concatenate all segments with 0.3s silence between
    let silence_path = tmp.path().join("silence.wav");
    Command::new("ffmpeg").args([
        "-y", "-f", "lavfi", "-i", "anullsrc=r=24000:cl=mono", "-t", "0.3", &silence_path.to_str().unwrap()
    ]).output().await.map_err(|e| e.to_string())?;

    let concat_file = tmp.path().join("concat.txt");
    let mut concat_content = String::new();
    for (i, path) in audio_paths.iter().enumerate() {
        concat_content.push_str(&format!("file '{}'\n", path.display()));
        if i < audio_paths.len() - 1 {
            concat_content.push_str(&format!("file '{}'\n", silence_path.display()));
        }
    }
    tokio::fs::write(&concat_file, &concat_content).await.map_err(|e| e.to_string())?;

    let dialogue_path = tmp.path().join("dialogue.wav");
    Command::new("ffmpeg").args([
        "-y", "-f", "concat", "-safe", "0", "-i", concat_file.to_str().unwrap(),
        "-c", "copy", dialogue_path.to_str().unwrap()
    ]).output().await.map_err(|e| e.to_string())?;

    // Add background music if requested
    if let Some(parent) = Path::new(&params.output_file).parent() {
        if !parent.as_os_str().is_empty() { tokio::fs::create_dir_all(parent).await.ok(); }
    }

    if let Some(ref music_prompt) = params.background_music {
        let music_url = format!("{}/models/lyria-3-clip-preview:generateContent", base);
        let music_body = serde_json::json!({
            "contents": [{"parts": [{"text": format!("{} Instrumental only, no vocals.", music_prompt)}]}],
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

            let vol = params.music_volume;
            let filter = if params.intro_music {
                // 3s music intro then mix
                format!("[1:a]volume={}[m];[0:a]adelay=3000|3000[d];[d][m]amix=inputs=2:duration=longest[a]", vol)
            } else {
                format!("[1:a]volume={}[m];[0:a][m]amix=inputs=2:duration=first[a]", vol)
            };

            Command::new("ffmpeg").args([
                "-y", "-i", dialogue_path.to_str().unwrap(),
                "-stream_loop", "-1", "-i", music_path.to_str().unwrap(),
                "-filter_complex", &filter, "-map", "[a]", "-shortest",
                &params.output_file
            ]).output().await.map_err(|e| e.to_string())?;
        } else {
            tokio::fs::copy(&dialogue_path, &params.output_file).await.map_err(|e| e.to_string())?;
        }
    } else {
        tokio::fs::copy(&dialogue_path, &params.output_file).await.map_err(|e| e.to_string())?;
    }

    info!(path = %params.output_file, segments = params.script.len(), "Podcast generated");
    Ok(format!("Podcast saved to: {} ({} segments)", params.output_file, params.script.len()))
}
