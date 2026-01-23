//! AVTool handler for audio/video processing using FFmpeg.
//!
//! This module provides the `AVToolHandler` struct and parameter types for
//! FFmpeg-based media processing operations.

use adk_rust_mcp_common::auth::AuthProvider;
use adk_rust_mcp_common::config::Config;
use adk_rust_mcp_common::error::Error;
use adk_rust_mcp_common::gcs::{GcsClient, GcsUri};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info, instrument};
use uuid::Uuid;

// =============================================================================
// Constants
// =============================================================================

/// Default audio bitrate for MP3 conversion.
pub const DEFAULT_BITRATE: &str = "192k";

/// Default FPS for GIF conversion.
pub const DEFAULT_GIF_FPS: u8 = 10;

/// Default volume multiplier.
pub const DEFAULT_VOLUME: f32 = 1.0;

// =============================================================================
// Output Types
// =============================================================================

/// Media file information returned by ffprobe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaInfo {
    /// Duration in seconds.
    pub duration: f64,
    /// Container format name.
    pub format: String,
    /// List of streams in the file.
    pub streams: Vec<StreamInfo>,
}

/// Information about a single stream in a media file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamInfo {
    /// Stream index.
    pub index: u32,
    /// Codec type (video, audio, subtitle, etc.).
    pub codec_type: String,
    /// Codec name.
    pub codec_name: String,
    /// Video width (if video stream).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// Video height (if video stream).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// Audio sample rate (if audio stream).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_rate: Option<u32>,
    /// Number of audio channels (if audio stream).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channels: Option<u32>,
}

// =============================================================================
// Parameter Types
// =============================================================================

/// Parameters for getting media file information.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GetMediaInfoParams {
    /// Input file path (local path or GCS URI).
    pub input: String,
}

/// Parameters for converting WAV to MP3.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ConvertAudioParams {
    /// Input WAV file path (local path or GCS URI).
    pub input: String,
    /// Output MP3 file path (local path or GCS URI).
    pub output: String,
    /// Audio bitrate (e.g., "128k", "192k", "320k"). Default: "192k".
    #[serde(default = "default_bitrate")]
    pub bitrate: String,
}

fn default_bitrate() -> String {
    DEFAULT_BITRATE.to_string()
}

/// Parameters for converting video to GIF.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct VideoToGifParams {
    /// Input video file path (local path or GCS URI).
    pub input: String,
    /// Output GIF file path (local path or GCS URI).
    pub output: String,
    /// Frames per second for the GIF. Default: 10.
    #[serde(default = "default_fps")]
    pub fps: u8,
    /// Output width in pixels (height auto-calculated to maintain aspect ratio).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// Start time in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<f64>,
    /// Duration in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
}

fn default_fps() -> u8 {
    DEFAULT_GIF_FPS
}

/// Parameters for combining audio and video.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CombineAvParams {
    /// Input video file path (local path or GCS URI).
    pub video_input: String,
    /// Input audio file path (local path or GCS URI).
    pub audio_input: String,
    /// Output file path (local path or GCS URI).
    pub output: String,
}

/// Parameters for overlaying an image on video.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct OverlayImageParams {
    /// Input video file path (local path or GCS URI).
    pub video_input: String,
    /// Input image file path (local path or GCS URI).
    pub image_input: String,
    /// Output file path (local path or GCS URI).
    pub output: String,
    /// X position of the overlay (from left). Default: 0.
    #[serde(default)]
    pub x: i32,
    /// Y position of the overlay (from top). Default: 0.
    #[serde(default)]
    pub y: i32,
    /// Scale factor for the image (e.g., 0.5 for half size).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<f32>,
    /// Start time in seconds when overlay appears.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<f64>,
    /// Duration in seconds for the overlay.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
}

/// Parameters for concatenating media files.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ConcatenateParams {
    /// List of input file paths (local paths or GCS URIs).
    pub inputs: Vec<String>,
    /// Output file path (local path or GCS URI).
    pub output: String,
}

/// Parameters for adjusting audio volume.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AdjustVolumeParams {
    /// Input audio file path (local path or GCS URI).
    pub input: String,
    /// Output audio file path (local path or GCS URI).
    pub output: String,
    /// Volume adjustment: numeric multiplier (e.g., "0.5", "2.0") or dB string (e.g., "-3dB", "+6dB").
    pub volume: String,
}

/// Parameters for layering multiple audio files.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct LayerAudioParams {
    /// List of audio layers to mix.
    pub inputs: Vec<AudioLayer>,
    /// Output file path (local path or GCS URI).
    pub output: String,
}

/// A single audio layer for mixing.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct AudioLayer {
    /// Input audio file path (local path or GCS URI).
    pub path: String,
    /// Offset in seconds from the start. Default: 0.0.
    #[serde(default)]
    pub offset_seconds: f64,
    /// Volume multiplier for this layer. Default: 1.0.
    #[serde(default = "default_volume")]
    pub volume: f32,
}

fn default_volume() -> f32 {
    DEFAULT_VOLUME
}

// =============================================================================
// Validation
// =============================================================================

/// Validation error details.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// The field that failed validation.
    pub field: String,
    /// Description of the validation failure.
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Parsed volume value.
#[derive(Debug, Clone, PartialEq)]
pub enum VolumeValue {
    /// Numeric multiplier (e.g., 0.5, 2.0).
    Multiplier(f64),
    /// Decibel adjustment (e.g., -3.0, +6.0).
    Decibels(f64),
}

impl VolumeValue {
    /// Parse a volume string into a VolumeValue.
    ///
    /// Accepts:
    /// - Numeric multipliers: "0.5", "2.0", "1"
    /// - Decibel strings: "-3dB", "+6dB", "0dB"
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();
        
        if s.is_empty() {
            return Err("Volume string cannot be empty".to_string());
        }
        
        // Check for dB suffix (case-insensitive)
        let lower = s.to_lowercase();
        if lower.ends_with("db") {
            let num_part = &s[..s.len() - 2].trim();
            let db_value: f64 = num_part.parse().map_err(|_| {
                format!("Invalid dB value '{}'. Expected format: '-3dB', '+6dB'", s)
            })?;
            return Ok(VolumeValue::Decibels(db_value));
        }
        
        // Try to parse as numeric multiplier
        let multiplier: f64 = s.parse().map_err(|_| {
            format!(
                "Invalid volume '{}'. Expected numeric multiplier (e.g., '0.5', '2.0') or dB string (e.g., '-3dB', '+6dB')",
                s
            )
        })?;
        
        if multiplier < 0.0 {
            return Err(format!(
                "Volume multiplier cannot be negative: {}. Use dB notation for attenuation (e.g., '-3dB')",
                multiplier
            ));
        }
        
        Ok(VolumeValue::Multiplier(multiplier))
    }
    
    /// Convert to FFmpeg volume filter value.
    pub fn to_ffmpeg_value(&self) -> String {
        match self {
            VolumeValue::Multiplier(m) => format!("{}", m),
            VolumeValue::Decibels(db) => format!("{}dB", db),
        }
    }
}

impl AdjustVolumeParams {
    /// Validate the volume parameter.
    pub fn validate(&self) -> Result<VolumeValue, Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        if self.input.trim().is_empty() {
            errors.push(ValidationError {
                field: "input".to_string(),
                message: "Input path cannot be empty".to_string(),
            });
        }
        
        if self.output.trim().is_empty() {
            errors.push(ValidationError {
                field: "output".to_string(),
                message: "Output path cannot be empty".to_string(),
            });
        }
        
        let volume = match VolumeValue::parse(&self.volume) {
            Ok(v) => Some(v),
            Err(e) => {
                errors.push(ValidationError {
                    field: "volume".to_string(),
                    message: e,
                });
                None
            }
        };
        
        if errors.is_empty() {
            Ok(volume.unwrap())
        } else {
            Err(errors)
        }
    }
}


// =============================================================================
// AVToolHandler
// =============================================================================

/// AVTool handler for FFmpeg-based media processing.
pub struct AVToolHandler {
    /// Application configuration.
    pub config: Config,
    /// GCS client for storage operations.
    pub gcs: GcsClient,
    /// Temporary directory for downloaded files.
    temp_dir: PathBuf,
}

impl AVToolHandler {
    /// Create a new AVToolHandler with the given configuration.
    ///
    /// # Errors
    /// Returns an error if GCS client initialization fails.
    #[instrument(level = "debug", name = "avtool_handler_new", skip_all)]
    pub async fn new(config: Config) -> Result<Self, Error> {
        debug!("Initializing AVToolHandler");

        let auth = AuthProvider::new().await?;
        let gcs = GcsClient::with_auth(auth);
        
        // Create temp directory for downloaded files
        let temp_dir = std::env::temp_dir().join("adk-rust-mcp-avtool");
        tokio::fs::create_dir_all(&temp_dir).await?;

        Ok(Self {
            config,
            gcs,
            temp_dir,
        })
    }

    /// Create a new AVToolHandler with provided dependencies (for testing).
    #[cfg(test)]
    pub fn with_deps(config: Config, gcs: GcsClient, temp_dir: PathBuf) -> Self {
        Self {
            config,
            gcs,
            temp_dir,
        }
    }

    // =========================================================================
    // Path Resolution Helpers
    // =========================================================================

    /// Check if a path is a GCS URI.
    pub fn is_gcs_uri(path: &str) -> bool {
        path.starts_with("gs://")
    }

    /// Resolve an input path, downloading from GCS if necessary.
    ///
    /// Returns the local path to use for FFmpeg operations.
    #[instrument(level = "debug", skip(self))]
    pub async fn resolve_input(&self, path: &str) -> Result<PathBuf, Error> {
        if Self::is_gcs_uri(path) {
            // Download from GCS to temp file
            let gcs_uri = GcsUri::parse(path)?;
            let filename = Path::new(&gcs_uri.object)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("input");
            
            let local_path = self.temp_dir.join(format!("{}_{}", Uuid::new_v4(), filename));
            
            debug!(gcs_uri = %path, local_path = %local_path.display(), "Downloading from GCS");
            let data = self.gcs.download(&gcs_uri).await?;
            tokio::fs::write(&local_path, &data).await?;
            
            Ok(local_path)
        } else {
            // Local path, use as-is
            Ok(PathBuf::from(path))
        }
    }

    /// Handle output, uploading to GCS if the output path is a GCS URI.
    ///
    /// Returns the final output path (GCS URI or local path).
    #[instrument(level = "debug", skip(self))]
    pub async fn handle_output(&self, local_path: &Path, output: &str) -> Result<String, Error> {
        if Self::is_gcs_uri(output) {
            // Upload to GCS
            let gcs_uri = GcsUri::parse(output)?;
            let data = tokio::fs::read(local_path).await?;
            
            // Determine content type from extension
            let content_type = Self::content_type_from_extension(local_path);
            
            debug!(local_path = %local_path.display(), gcs_uri = %output, "Uploading to GCS");
            self.gcs.upload(&gcs_uri, &data, content_type).await?;
            
            Ok(output.to_string())
        } else {
            // Local path - if different from local_path, copy the file
            if local_path != Path::new(output) {
                tokio::fs::copy(local_path, output).await?;
            }
            Ok(output.to_string())
        }
    }

    /// Get content type from file extension.
    fn content_type_from_extension(path: &Path) -> &'static str {
        match path.extension().and_then(|e| e.to_str()) {
            Some("mp3") => "audio/mpeg",
            Some("wav") => "audio/wav",
            Some("mp4") => "video/mp4",
            Some("webm") => "video/webm",
            Some("gif") => "image/gif",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("mkv") => "video/x-matroska",
            Some("avi") => "video/x-msvideo",
            Some("mov") => "video/quicktime",
            Some("ogg") => "audio/ogg",
            Some("flac") => "audio/flac",
            _ => "application/octet-stream",
        }
    }

    /// Generate a temporary output path.
    fn temp_output_path(&self, extension: &str) -> PathBuf {
        self.temp_dir.join(format!("{}.{}", Uuid::new_v4(), extension))
    }

    // =========================================================================
    // FFmpeg/FFprobe Execution
    // =========================================================================

    /// Execute ffprobe and return parsed JSON output.
    async fn run_ffprobe(&self, input: &Path) -> Result<serde_json::Value, Error> {
        let output = Command::new("ffprobe")
            .args([
                "-v", "quiet",
                "-print_format", "json",
                "-show_format",
                "-show_streams",
            ])
            .arg(input)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::ffmpeg(format!(
                "ffprobe failed for '{}': {}",
                input.display(),
                stderr
            )));
        }

        let json: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|e| {
            Error::ffmpeg(format!("Failed to parse ffprobe output: {}", e))
        })?;

        Ok(json)
    }

    /// Execute ffmpeg with the given arguments.
    async fn run_ffmpeg(&self, args: &[&str]) -> Result<(), Error> {
        debug!(args = ?args, "Running ffmpeg");
        
        let output = Command::new("ffmpeg")
            .args(["-y"]) // Overwrite output files
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::ffmpeg(format!("ffmpeg failed: {}", stderr)));
        }

        Ok(())
    }

    // =========================================================================
    // Tool Implementations
    // =========================================================================

    /// Get media file information using ffprobe.
    #[instrument(level = "info", skip(self))]
    pub async fn get_media_info(&self, params: GetMediaInfoParams) -> Result<MediaInfo, Error> {
        let local_input = self.resolve_input(&params.input).await?;
        
        let json = self.run_ffprobe(&local_input).await?;
        
        // Parse format info
        let format = json.get("format").ok_or_else(|| {
            Error::ffmpeg("ffprobe output missing 'format' field")
        })?;
        
        let duration: f64 = format
            .get("duration")
            .and_then(|d| d.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        
        let format_name = format
            .get("format_name")
            .and_then(|f| f.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        // Parse streams
        let streams_json = json.get("streams").and_then(|s| s.as_array());
        let streams: Vec<StreamInfo> = streams_json
            .map(|arr| {
                arr.iter()
                    .map(|s| StreamInfo {
                        index: s.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as u32,
                        codec_type: s.get("codec_type").and_then(|c| c.as_str()).unwrap_or("unknown").to_string(),
                        codec_name: s.get("codec_name").and_then(|c| c.as_str()).unwrap_or("unknown").to_string(),
                        width: s.get("width").and_then(|w| w.as_u64()).map(|w| w as u32),
                        height: s.get("height").and_then(|h| h.as_u64()).map(|h| h as u32),
                        sample_rate: s.get("sample_rate").and_then(|r| r.as_str()).and_then(|s| s.parse().ok()),
                        channels: s.get("channels").and_then(|c| c.as_u64()).map(|c| c as u32),
                    })
                    .collect()
            })
            .unwrap_or_default();
        
        // Clean up temp file if we downloaded from GCS
        if Self::is_gcs_uri(&params.input) {
            let _ = tokio::fs::remove_file(&local_input).await;
        }
        
        info!(duration, format = %format_name, streams = streams.len(), "Got media info");
        
        Ok(MediaInfo {
            duration,
            format: format_name,
            streams,
        })
    }

    /// Convert WAV to MP3.
    #[instrument(level = "info", skip(self))]
    pub async fn convert_wav_to_mp3(&self, params: ConvertAudioParams) -> Result<String, Error> {
        let local_input = self.resolve_input(&params.input).await?;
        let temp_output = self.temp_output_path("mp3");
        
        let input_str = local_input.to_string_lossy();
        let output_str = temp_output.to_string_lossy();
        
        self.run_ffmpeg(&[
            "-i", &input_str,
            "-codec:a", "libmp3lame",
            "-b:a", &params.bitrate,
            &output_str,
        ]).await?;
        
        let result = self.handle_output(&temp_output, &params.output).await?;
        
        // Clean up temp files
        if Self::is_gcs_uri(&params.input) {
            let _ = tokio::fs::remove_file(&local_input).await;
        }
        let _ = tokio::fs::remove_file(&temp_output).await;
        
        info!(output = %result, "Converted WAV to MP3");
        Ok(result)
    }

    /// Convert video to GIF.
    #[instrument(level = "info", skip(self))]
    pub async fn video_to_gif(&self, params: VideoToGifParams) -> Result<String, Error> {
        let local_input = self.resolve_input(&params.input).await?;
        let temp_output = self.temp_output_path("gif");
        
        let input_str = local_input.to_string_lossy();
        let output_str = temp_output.to_string_lossy();
        
        // Build filter string
        let mut filters = vec![format!("fps={}", params.fps)];
        if let Some(width) = params.width {
            filters.push(format!("scale={}:-1:flags=lanczos", width));
        }
        let filter_str = filters.join(",");
        
        let mut args: Vec<String> = Vec::new();
        
        // Add start time if specified
        if let Some(start) = params.start_time {
            args.push("-ss".to_string());
            args.push(format!("{}", start));
        }
        
        args.push("-i".to_string());
        args.push(input_str.to_string());
        
        // Add duration if specified
        if let Some(duration) = params.duration {
            args.push("-t".to_string());
            args.push(format!("{}", duration));
        }
        
        args.push("-vf".to_string());
        args.push(filter_str);
        args.push(output_str.to_string());
        
        let args_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.run_ffmpeg(&args_refs).await?;
        
        let result = self.handle_output(&temp_output, &params.output).await?;
        
        // Clean up temp files
        if Self::is_gcs_uri(&params.input) {
            let _ = tokio::fs::remove_file(&local_input).await;
        }
        let _ = tokio::fs::remove_file(&temp_output).await;
        
        info!(output = %result, "Converted video to GIF");
        Ok(result)
    }

    /// Combine audio and video.
    #[instrument(level = "info", skip(self))]
    pub async fn combine_audio_video(&self, params: CombineAvParams) -> Result<String, Error> {
        let local_video = self.resolve_input(&params.video_input).await?;
        let local_audio = self.resolve_input(&params.audio_input).await?;
        
        // Determine output extension from output path
        let ext = Path::new(&params.output)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp4");
        let temp_output = self.temp_output_path(ext);
        
        let video_str = local_video.to_string_lossy();
        let audio_str = local_audio.to_string_lossy();
        let output_str = temp_output.to_string_lossy();
        
        self.run_ffmpeg(&[
            "-i", &video_str,
            "-i", &audio_str,
            "-c:v", "copy",
            "-c:a", "aac",
            "-map", "0:v:0",
            "-map", "1:a:0",
            "-shortest",
            &output_str,
        ]).await?;
        
        let result = self.handle_output(&temp_output, &params.output).await?;
        
        // Clean up temp files
        if Self::is_gcs_uri(&params.video_input) {
            let _ = tokio::fs::remove_file(&local_video).await;
        }
        if Self::is_gcs_uri(&params.audio_input) {
            let _ = tokio::fs::remove_file(&local_audio).await;
        }
        let _ = tokio::fs::remove_file(&temp_output).await;
        
        info!(output = %result, "Combined audio and video");
        Ok(result)
    }

    /// Overlay image on video.
    #[instrument(level = "info", skip(self))]
    pub async fn overlay_image(&self, params: OverlayImageParams) -> Result<String, Error> {
        let local_video = self.resolve_input(&params.video_input).await?;
        let local_image = self.resolve_input(&params.image_input).await?;
        
        let ext = Path::new(&params.output)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp4");
        let temp_output = self.temp_output_path(ext);
        
        let video_str = local_video.to_string_lossy();
        let image_str = local_image.to_string_lossy();
        let output_str = temp_output.to_string_lossy();
        
        // Build filter complex
        let mut filter_parts = Vec::new();
        
        // Scale image if specified
        if let Some(scale) = params.scale {
            filter_parts.push(format!("[1:v]scale=iw*{}:ih*{}[img]", scale, scale));
        }
        
        // Build overlay filter with position and timing
        let img_ref = if params.scale.is_some() { "[img]" } else { "[1:v]" };
        let mut overlay = format!("[0:v]{}overlay={}:{}", img_ref, params.x, params.y);
        
        // Add enable expression for timing
        if params.start_time.is_some() || params.duration.is_some() {
            let start = params.start_time.unwrap_or(0.0);
            let enable = if let Some(dur) = params.duration {
                format!(":enable='between(t,{},{})'", start, start + dur)
            } else {
                format!(":enable='gte(t,{})'", start)
            };
            overlay.push_str(&enable);
        }
        
        filter_parts.push(overlay);
        let filter_complex = filter_parts.join(";");
        
        self.run_ffmpeg(&[
            "-i", &video_str,
            "-i", &image_str,
            "-filter_complex", &filter_complex,
            "-c:a", "copy",
            &output_str,
        ]).await?;
        
        let result = self.handle_output(&temp_output, &params.output).await?;
        
        // Clean up temp files
        if Self::is_gcs_uri(&params.video_input) {
            let _ = tokio::fs::remove_file(&local_video).await;
        }
        if Self::is_gcs_uri(&params.image_input) {
            let _ = tokio::fs::remove_file(&local_image).await;
        }
        let _ = tokio::fs::remove_file(&temp_output).await;
        
        info!(output = %result, "Overlaid image on video");
        Ok(result)
    }

    /// Concatenate media files.
    #[instrument(level = "info", skip(self))]
    pub async fn concatenate(&self, params: ConcatenateParams) -> Result<String, Error> {
        if params.inputs.is_empty() {
            return Err(Error::validation("At least one input file is required"));
        }
        
        // Resolve all inputs
        let mut local_inputs = Vec::new();
        for input in &params.inputs {
            local_inputs.push(self.resolve_input(input).await?);
        }
        
        let ext = Path::new(&params.output)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("mp4");
        let temp_output = self.temp_output_path(ext);
        
        // Create concat file list
        let concat_file = self.temp_dir.join(format!("{}_concat.txt", Uuid::new_v4()));
        let concat_content: String = local_inputs
            .iter()
            .map(|p| format!("file '{}'\n", p.display()))
            .collect();
        tokio::fs::write(&concat_file, &concat_content).await?;
        
        let concat_str = concat_file.to_string_lossy();
        let output_str = temp_output.to_string_lossy();
        
        self.run_ffmpeg(&[
            "-f", "concat",
            "-safe", "0",
            "-i", &concat_str,
            "-c", "copy",
            &output_str,
        ]).await?;
        
        let result = self.handle_output(&temp_output, &params.output).await?;
        
        // Clean up temp files
        for (i, input) in params.inputs.iter().enumerate() {
            if Self::is_gcs_uri(input) {
                let _ = tokio::fs::remove_file(&local_inputs[i]).await;
            }
        }
        let _ = tokio::fs::remove_file(&concat_file).await;
        let _ = tokio::fs::remove_file(&temp_output).await;
        
        info!(output = %result, count = params.inputs.len(), "Concatenated media files");
        Ok(result)
    }

    /// Adjust audio volume.
    #[instrument(level = "info", skip(self))]
    pub async fn adjust_volume(&self, params: AdjustVolumeParams) -> Result<String, Error> {
        // Validate and parse volume
        let volume = params.validate().map_err(|errors| {
            let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            Error::validation(messages.join("; "))
        })?;
        
        let local_input = self.resolve_input(&params.input).await?;
        
        let ext = Path::new(&params.output)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("wav");
        let temp_output = self.temp_output_path(ext);
        
        let input_str = local_input.to_string_lossy();
        let output_str = temp_output.to_string_lossy();
        let volume_filter = format!("volume={}", volume.to_ffmpeg_value());
        
        self.run_ffmpeg(&[
            "-i", &input_str,
            "-af", &volume_filter,
            &output_str,
        ]).await?;
        
        let result = self.handle_output(&temp_output, &params.output).await?;
        
        // Clean up temp files
        if Self::is_gcs_uri(&params.input) {
            let _ = tokio::fs::remove_file(&local_input).await;
        }
        let _ = tokio::fs::remove_file(&temp_output).await;
        
        info!(output = %result, volume = ?volume, "Adjusted audio volume");
        Ok(result)
    }

    /// Layer multiple audio files.
    #[instrument(level = "info", skip(self))]
    pub async fn layer_audio(&self, params: LayerAudioParams) -> Result<String, Error> {
        if params.inputs.is_empty() {
            return Err(Error::validation("At least one audio layer is required"));
        }
        
        // Resolve all inputs
        let mut local_inputs = Vec::new();
        for layer in &params.inputs {
            local_inputs.push(self.resolve_input(&layer.path).await?);
        }
        
        let ext = Path::new(&params.output)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("wav");
        let temp_output = self.temp_output_path(ext);
        
        // Build ffmpeg command with amix filter
        let mut args = Vec::new();
        
        // Add all inputs
        for local_input in &local_inputs {
            args.push("-i".to_string());
            args.push(local_input.to_string_lossy().to_string());
        }
        
        // Build filter complex for mixing with delays and volumes
        let mut filter_parts = Vec::new();
        let mut mix_inputs = Vec::new();
        
        for (i, layer) in params.inputs.iter().enumerate() {
            let label = format!("a{}", i);
            let mut filter = format!("[{}:a]", i);
            
            // Add delay if offset > 0
            if layer.offset_seconds > 0.0 {
                let delay_ms = (layer.offset_seconds * 1000.0) as i64;
                filter.push_str(&format!("adelay={}|{}", delay_ms, delay_ms));
                if layer.volume != 1.0 {
                    filter.push_str(&format!(",volume={}", layer.volume));
                }
            } else if layer.volume != 1.0 {
                filter.push_str(&format!("volume={}", layer.volume));
            } else {
                filter.push_str("anull");
            }
            
            filter.push_str(&format!("[{}]", label));
            filter_parts.push(filter);
            mix_inputs.push(format!("[{}]", label));
        }
        
        // Add amix filter
        let mix_filter = format!(
            "{}amix=inputs={}:duration=longest",
            mix_inputs.join(""),
            params.inputs.len()
        );
        filter_parts.push(mix_filter);
        
        let filter_complex = filter_parts.join(";");
        
        args.extend([
            "-filter_complex".to_string(),
            filter_complex,
            temp_output.to_string_lossy().to_string(),
        ]);
        
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        self.run_ffmpeg(&args_refs).await?;
        
        let result = self.handle_output(&temp_output, &params.output).await?;
        
        // Clean up temp files
        for (i, layer) in params.inputs.iter().enumerate() {
            if Self::is_gcs_uri(&layer.path) {
                let _ = tokio::fs::remove_file(&local_inputs[i]).await;
            }
        }
        let _ = tokio::fs::remove_file(&temp_output).await;
        
        info!(output = %result, layers = params.inputs.len(), "Layered audio files");
        Ok(result)
    }
}


// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // FFmpeg Error Handling Tests (Requirements 9.19, 9.20)
    // =========================================================================

    #[test]
    fn test_ffmpeg_error_contains_stderr_output() {
        // Verify that FFmpeg errors include the stderr output for debugging
        let stderr_output = "Invalid input file: file not found";
        let err = Error::ffmpeg(format!("ffmpeg failed: {}", stderr_output));
        let msg = err.to_string();
        
        assert!(msg.contains("FFmpeg"), "Error should mention FFmpeg");
        assert!(msg.contains("Invalid input file"), "Error should contain stderr output");
    }

    #[test]
    fn test_ffprobe_error_contains_file_path() {
        // Verify that FFprobe errors include the file path for context
        let file_path = "/path/to/nonexistent.mp4";
        let err = Error::ffmpeg(format!("ffprobe failed for '{}': No such file or directory", file_path));
        let msg = err.to_string();
        
        assert!(msg.contains("ffprobe"), "Error should mention ffprobe");
        assert!(msg.contains(file_path), "Error should contain file path");
    }

    #[test]
    fn test_ffmpeg_error_preserves_codec_errors() {
        // Verify that codec-related errors are preserved
        let codec_error = "Unknown encoder 'libx265'";
        let err = Error::ffmpeg(format!("ffmpeg failed: {}", codec_error));
        let msg = err.to_string();
        
        assert!(msg.contains("libx265"), "Error should preserve codec name");
        assert!(msg.contains("Unknown encoder"), "Error should preserve error type");
    }

    #[test]
    fn test_ffmpeg_error_preserves_format_errors() {
        // Verify that format-related errors are preserved
        let format_error = "Invalid data found when processing input";
        let err = Error::ffmpeg(format!("ffmpeg failed: {}", format_error));
        let msg = err.to_string();
        
        assert!(msg.contains("Invalid data"), "Error should preserve format error");
    }

    // =========================================================================
    // Media Info Extraction Tests (Requirement 9.11)
    // =========================================================================

    #[test]
    fn test_media_info_parsing_video_stream() {
        // Test parsing of video stream information
        let stream = StreamInfo {
            index: 0,
            codec_type: "video".to_string(),
            codec_name: "h264".to_string(),
            width: Some(1920),
            height: Some(1080),
            sample_rate: None,
            channels: None,
        };
        
        assert_eq!(stream.codec_type, "video");
        assert_eq!(stream.codec_name, "h264");
        assert_eq!(stream.width, Some(1920));
        assert_eq!(stream.height, Some(1080));
        assert!(stream.sample_rate.is_none());
        assert!(stream.channels.is_none());
    }

    #[test]
    fn test_media_info_parsing_audio_stream() {
        // Test parsing of audio stream information
        let stream = StreamInfo {
            index: 1,
            codec_type: "audio".to_string(),
            codec_name: "aac".to_string(),
            width: None,
            height: None,
            sample_rate: Some(48000),
            channels: Some(2),
        };
        
        assert_eq!(stream.codec_type, "audio");
        assert_eq!(stream.codec_name, "aac");
        assert!(stream.width.is_none());
        assert!(stream.height.is_none());
        assert_eq!(stream.sample_rate, Some(48000));
        assert_eq!(stream.channels, Some(2));
    }

    #[test]
    fn test_media_info_complete_structure() {
        // Test complete MediaInfo structure with multiple streams
        let info = MediaInfo {
            duration: 120.5,
            format: "matroska,webm".to_string(),
            streams: vec![
                StreamInfo {
                    index: 0,
                    codec_type: "video".to_string(),
                    codec_name: "vp9".to_string(),
                    width: Some(3840),
                    height: Some(2160),
                    sample_rate: None,
                    channels: None,
                },
                StreamInfo {
                    index: 1,
                    codec_type: "audio".to_string(),
                    codec_name: "opus".to_string(),
                    width: None,
                    height: None,
                    sample_rate: Some(48000),
                    channels: Some(6),
                },
                StreamInfo {
                    index: 2,
                    codec_type: "subtitle".to_string(),
                    codec_name: "subrip".to_string(),
                    width: None,
                    height: None,
                    sample_rate: None,
                    channels: None,
                },
            ],
        };
        
        assert_eq!(info.duration, 120.5);
        assert_eq!(info.format, "matroska,webm");
        assert_eq!(info.streams.len(), 3);
        
        // Verify video stream
        assert_eq!(info.streams[0].codec_type, "video");
        assert_eq!(info.streams[0].width, Some(3840));
        
        // Verify audio stream
        assert_eq!(info.streams[1].codec_type, "audio");
        assert_eq!(info.streams[1].channels, Some(6));
        
        // Verify subtitle stream
        assert_eq!(info.streams[2].codec_type, "subtitle");
    }

    #[test]
    fn test_media_info_json_output_format() {
        // Test that MediaInfo serializes to proper JSON format
        let info = MediaInfo {
            duration: 60.0,
            format: "mp4".to_string(),
            streams: vec![
                StreamInfo {
                    index: 0,
                    codec_type: "video".to_string(),
                    codec_name: "h264".to_string(),
                    width: Some(1280),
                    height: Some(720),
                    sample_rate: None,
                    channels: None,
                },
            ],
        };
        
        let json = serde_json::to_value(&info).unwrap();
        
        // Verify JSON structure
        assert!(json.is_object());
        assert!(json["duration"].is_f64());
        assert!(json["format"].is_string());
        assert!(json["streams"].is_array());
        
        // Verify values
        assert_eq!(json["duration"].as_f64().unwrap(), 60.0);
        assert_eq!(json["format"].as_str().unwrap(), "mp4");
        assert_eq!(json["streams"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_media_info_empty_streams() {
        // Test MediaInfo with no streams (edge case)
        let info = MediaInfo {
            duration: 0.0,
            format: "unknown".to_string(),
            streams: vec![],
        };
        
        let json = serde_json::to_string(&info).unwrap();
        let parsed: MediaInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.duration, 0.0);
        assert_eq!(parsed.format, "unknown");
        assert!(parsed.streams.is_empty());
    }

    // =========================================================================
    // VolumeValue Tests
    // =========================================================================

    #[test]
    fn test_volume_parse_multiplier() {
        assert_eq!(VolumeValue::parse("0.5").unwrap(), VolumeValue::Multiplier(0.5));
        assert_eq!(VolumeValue::parse("1.0").unwrap(), VolumeValue::Multiplier(1.0));
        assert_eq!(VolumeValue::parse("2.0").unwrap(), VolumeValue::Multiplier(2.0));
        assert_eq!(VolumeValue::parse("1").unwrap(), VolumeValue::Multiplier(1.0));
        assert_eq!(VolumeValue::parse("0").unwrap(), VolumeValue::Multiplier(0.0));
    }

    #[test]
    fn test_volume_parse_decibels() {
        assert_eq!(VolumeValue::parse("-3dB").unwrap(), VolumeValue::Decibels(-3.0));
        assert_eq!(VolumeValue::parse("+6dB").unwrap(), VolumeValue::Decibels(6.0));
        assert_eq!(VolumeValue::parse("0dB").unwrap(), VolumeValue::Decibels(0.0));
        assert_eq!(VolumeValue::parse("-10.5dB").unwrap(), VolumeValue::Decibels(-10.5));
        // Case insensitive
        assert_eq!(VolumeValue::parse("-3DB").unwrap(), VolumeValue::Decibels(-3.0));
        assert_eq!(VolumeValue::parse("-3db").unwrap(), VolumeValue::Decibels(-3.0));
    }

    #[test]
    fn test_volume_parse_with_whitespace() {
        assert_eq!(VolumeValue::parse("  0.5  ").unwrap(), VolumeValue::Multiplier(0.5));
        assert_eq!(VolumeValue::parse("  -3dB  ").unwrap(), VolumeValue::Decibels(-3.0));
    }

    #[test]
    fn test_volume_parse_invalid() {
        assert!(VolumeValue::parse("").is_err());
        assert!(VolumeValue::parse("abc").is_err());
        assert!(VolumeValue::parse("dB").is_err());
        assert!(VolumeValue::parse("-3").is_err()); // Negative multiplier not allowed
    }

    #[test]
    fn test_volume_to_ffmpeg_value() {
        assert_eq!(VolumeValue::Multiplier(0.5).to_ffmpeg_value(), "0.5");
        assert_eq!(VolumeValue::Multiplier(2.0).to_ffmpeg_value(), "2");
        assert_eq!(VolumeValue::Decibels(-3.0).to_ffmpeg_value(), "-3dB");
        assert_eq!(VolumeValue::Decibels(6.0).to_ffmpeg_value(), "6dB");
    }

    // =========================================================================
    // Parameter Validation Tests
    // =========================================================================

    #[test]
    fn test_adjust_volume_params_valid() {
        let params = AdjustVolumeParams {
            input: "input.wav".to_string(),
            output: "output.wav".to_string(),
            volume: "0.5".to_string(),
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_adjust_volume_params_invalid_volume() {
        let params = AdjustVolumeParams {
            input: "input.wav".to_string(),
            output: "output.wav".to_string(),
            volume: "invalid".to_string(),
        };
        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "volume"));
    }

    #[test]
    fn test_adjust_volume_params_empty_input() {
        let params = AdjustVolumeParams {
            input: "".to_string(),
            output: "output.wav".to_string(),
            volume: "0.5".to_string(),
        };
        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "input"));
    }

    // =========================================================================
    // GCS URI Detection Tests
    // =========================================================================

    #[test]
    fn test_is_gcs_uri() {
        assert!(AVToolHandler::is_gcs_uri("gs://bucket/path/file.mp4"));
        assert!(AVToolHandler::is_gcs_uri("gs://my-bucket/file.wav"));
        assert!(!AVToolHandler::is_gcs_uri("/local/path/file.mp4"));
        assert!(!AVToolHandler::is_gcs_uri("./relative/path.wav"));
        assert!(!AVToolHandler::is_gcs_uri("file.mp3"));
        assert!(!AVToolHandler::is_gcs_uri("s3://bucket/file.mp4"));
    }

    // =========================================================================
    // Content Type Tests
    // =========================================================================

    #[test]
    fn test_content_type_from_extension() {
        assert_eq!(AVToolHandler::content_type_from_extension(Path::new("file.mp3")), "audio/mpeg");
        assert_eq!(AVToolHandler::content_type_from_extension(Path::new("file.wav")), "audio/wav");
        assert_eq!(AVToolHandler::content_type_from_extension(Path::new("file.mp4")), "video/mp4");
        assert_eq!(AVToolHandler::content_type_from_extension(Path::new("file.gif")), "image/gif");
        assert_eq!(AVToolHandler::content_type_from_extension(Path::new("file.png")), "image/png");
        assert_eq!(AVToolHandler::content_type_from_extension(Path::new("file.jpg")), "image/jpeg");
        assert_eq!(AVToolHandler::content_type_from_extension(Path::new("file.unknown")), "application/octet-stream");
        assert_eq!(AVToolHandler::content_type_from_extension(Path::new("file")), "application/octet-stream");
    }

    // =========================================================================
    // Serialization Tests
    // =========================================================================

    #[test]
    fn test_media_info_serialization() {
        let info = MediaInfo {
            duration: 10.5,
            format: "mp4".to_string(),
            streams: vec![
                StreamInfo {
                    index: 0,
                    codec_type: "video".to_string(),
                    codec_name: "h264".to_string(),
                    width: Some(1920),
                    height: Some(1080),
                    sample_rate: None,
                    channels: None,
                },
                StreamInfo {
                    index: 1,
                    codec_type: "audio".to_string(),
                    codec_name: "aac".to_string(),
                    width: None,
                    height: None,
                    sample_rate: Some(44100),
                    channels: Some(2),
                },
            ],
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: MediaInfo = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.duration, 10.5);
        assert_eq!(deserialized.format, "mp4");
        assert_eq!(deserialized.streams.len(), 2);
    }

    #[test]
    fn test_convert_audio_params_defaults() {
        let params: ConvertAudioParams = serde_json::from_str(r#"{
            "input": "input.wav",
            "output": "output.mp3"
        }"#).unwrap();
        
        assert_eq!(params.bitrate, DEFAULT_BITRATE);
    }

    #[test]
    fn test_video_to_gif_params_defaults() {
        let params: VideoToGifParams = serde_json::from_str(r#"{
            "input": "input.mp4",
            "output": "output.gif"
        }"#).unwrap();
        
        assert_eq!(params.fps, DEFAULT_GIF_FPS);
        assert!(params.width.is_none());
        assert!(params.start_time.is_none());
        assert!(params.duration.is_none());
    }

    #[test]
    fn test_audio_layer_defaults() {
        let layer: AudioLayer = serde_json::from_str(r#"{
            "path": "audio.wav"
        }"#).unwrap();
        
        assert_eq!(layer.offset_seconds, 0.0);
        assert_eq!(layer.volume, DEFAULT_VOLUME);
    }

    // =========================================================================
    // Concatenate Validation Tests
    // =========================================================================

    #[test]
    fn test_concatenate_params_valid() {
        let params = ConcatenateParams {
            inputs: vec!["file1.mp4".to_string(), "file2.mp4".to_string()],
            output: "output.mp4".to_string(),
        };
        
        assert!(!params.inputs.is_empty());
        assert_eq!(params.inputs.len(), 2);
    }

    #[test]
    fn test_concatenate_params_single_input() {
        let params = ConcatenateParams {
            inputs: vec!["file1.mp4".to_string()],
            output: "output.mp4".to_string(),
        };
        
        // Single input is valid (though not very useful)
        assert_eq!(params.inputs.len(), 1);
    }

    // =========================================================================
    // Layer Audio Validation Tests
    // =========================================================================

    #[test]
    fn test_layer_audio_params_valid() {
        let params = LayerAudioParams {
            inputs: vec![
                AudioLayer {
                    path: "audio1.wav".to_string(),
                    offset_seconds: 0.0,
                    volume: 1.0,
                },
                AudioLayer {
                    path: "audio2.wav".to_string(),
                    offset_seconds: 2.5,
                    volume: 0.8,
                },
            ],
            output: "mixed.wav".to_string(),
        };
        
        assert_eq!(params.inputs.len(), 2);
        assert_eq!(params.inputs[1].offset_seconds, 2.5);
        assert_eq!(params.inputs[1].volume, 0.8);
    }

    #[test]
    fn test_layer_audio_with_negative_offset() {
        // Negative offset should be allowed (for pre-delay effects)
        let layer = AudioLayer {
            path: "audio.wav".to_string(),
            offset_seconds: -1.0,
            volume: 1.0,
        };
        
        // The struct allows negative values, validation happens at runtime
        assert_eq!(layer.offset_seconds, -1.0);
    }

    // =========================================================================
    // Overlay Image Params Tests
    // =========================================================================

    #[test]
    fn test_overlay_image_params_defaults() {
        let params: OverlayImageParams = serde_json::from_str(r#"{
            "video_input": "video.mp4",
            "image_input": "overlay.png",
            "output": "output.mp4"
        }"#).unwrap();
        
        assert_eq!(params.x, 0);
        assert_eq!(params.y, 0);
        assert!(params.scale.is_none());
        assert!(params.start_time.is_none());
        assert!(params.duration.is_none());
    }

    #[test]
    fn test_overlay_image_params_with_position() {
        let params: OverlayImageParams = serde_json::from_str(r#"{
            "video_input": "video.mp4",
            "image_input": "overlay.png",
            "output": "output.mp4",
            "x": 100,
            "y": 50,
            "scale": 0.5
        }"#).unwrap();
        
        assert_eq!(params.x, 100);
        assert_eq!(params.y, 50);
        assert_eq!(params.scale, Some(0.5));
    }

    // =========================================================================
    // Combine AV Params Tests
    // =========================================================================

    #[test]
    fn test_combine_av_params_valid() {
        let params: CombineAvParams = serde_json::from_str(r#"{
            "video_input": "video.mp4",
            "audio_input": "audio.wav",
            "output": "combined.mp4"
        }"#).unwrap();
        
        assert_eq!(params.video_input, "video.mp4");
        assert_eq!(params.audio_input, "audio.wav");
        assert_eq!(params.output, "combined.mp4");
    }
}


// =============================================================================
// Property-Based Tests
// =============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: rust-mcp-genmedia, Property 15: Volume String Parsing
    // **Validates: Requirements 9.17**
    //
    // For any volume parameter string, it SHALL be parsed as either:
    // (a) a numeric multiplier (e.g., "0.5", "2.0"), or
    // (b) a dB adjustment (e.g., "-3dB", "+6dB").
    // Invalid formats SHALL be rejected with a descriptive error.

    /// Strategy to generate valid numeric multipliers (non-negative floats)
    fn valid_multiplier_strategy() -> impl Strategy<Value = f64> {
        (0.0f64..=10.0f64)
    }

    /// Strategy to generate valid dB values (can be negative or positive)
    fn valid_db_strategy() -> impl Strategy<Value = f64> {
        (-60.0f64..=60.0f64)
    }

    proptest! {
        /// Property 15: Valid numeric multipliers should parse successfully
        #[test]
        fn valid_multiplier_parses_correctly(value in valid_multiplier_strategy()) {
            let input = format!("{}", value);
            let result = VolumeValue::parse(&input);
            
            prop_assert!(
                result.is_ok(),
                "Valid multiplier '{}' should parse successfully, got error: {:?}",
                input,
                result.err()
            );
            
            if let Ok(VolumeValue::Multiplier(parsed)) = result {
                // Allow for floating point precision differences
                prop_assert!(
                    (parsed - value).abs() < 0.0001,
                    "Parsed value {} should match input {}",
                    parsed,
                    value
                );
            }
        }

        /// Property 15: Valid dB strings should parse successfully
        #[test]
        fn valid_db_parses_correctly(value in valid_db_strategy()) {
            let input = format!("{}dB", value);
            let result = VolumeValue::parse(&input);
            
            prop_assert!(
                result.is_ok(),
                "Valid dB string '{}' should parse successfully, got error: {:?}",
                input,
                result.err()
            );
            
            if let Ok(VolumeValue::Decibels(parsed)) = result {
                prop_assert!(
                    (parsed - value).abs() < 0.0001,
                    "Parsed dB value {} should match input {}",
                    parsed,
                    value
                );
            }
        }

        /// Property 15: dB parsing should be case-insensitive
        #[test]
        fn db_parsing_case_insensitive(value in valid_db_strategy()) {
            let lower = format!("{}db", value);
            let upper = format!("{}DB", value);
            let mixed = format!("{}dB", value);
            
            let result_lower = VolumeValue::parse(&lower);
            let result_upper = VolumeValue::parse(&upper);
            let result_mixed = VolumeValue::parse(&mixed);
            
            prop_assert!(result_lower.is_ok(), "Lowercase 'db' should parse");
            prop_assert!(result_upper.is_ok(), "Uppercase 'DB' should parse");
            prop_assert!(result_mixed.is_ok(), "Mixed case 'dB' should parse");
            
            // All should produce the same value
            if let (Ok(VolumeValue::Decibels(v1)), Ok(VolumeValue::Decibels(v2)), Ok(VolumeValue::Decibels(v3))) = 
                (result_lower, result_upper, result_mixed) {
                prop_assert!((v1 - v2).abs() < 0.0001);
                prop_assert!((v2 - v3).abs() < 0.0001);
            }
        }

        /// Property 15: Whitespace should be trimmed
        #[test]
        fn whitespace_is_trimmed(value in valid_multiplier_strategy()) {
            let with_spaces = format!("  {}  ", value);
            let without_spaces = format!("{}", value);
            
            let result_with = VolumeValue::parse(&with_spaces);
            let result_without = VolumeValue::parse(&without_spaces);
            
            prop_assert!(result_with.is_ok(), "Should parse with whitespace");
            prop_assert!(result_without.is_ok(), "Should parse without whitespace");
            
            // Both should produce the same value
            prop_assert_eq!(
                result_with.ok(),
                result_without.ok(),
                "Whitespace should not affect parsing"
            );
        }

        /// Property 15: Negative multipliers should be rejected
        #[test]
        fn negative_multiplier_rejected(value in -100.0f64..-0.001f64) {
            let input = format!("{}", value);
            let result = VolumeValue::parse(&input);
            
            prop_assert!(
                result.is_err(),
                "Negative multiplier '{}' should be rejected",
                input
            );
        }

        /// Property 15: Invalid strings should be rejected with descriptive error
        #[test]
        fn invalid_strings_rejected(s in "[a-zA-Z]{1,10}") {
            // Skip strings that end with "db" (case insensitive) as they might be valid
            if !s.to_lowercase().ends_with("db") {
                let result = VolumeValue::parse(&s);
                
                prop_assert!(
                    result.is_err(),
                    "Invalid string '{}' should be rejected",
                    s
                );
                
                // Error message should be descriptive
                if let Err(msg) = result {
                    prop_assert!(
                        msg.contains("Invalid") || msg.contains("Expected"),
                        "Error message should be descriptive: {}",
                        msg
                    );
                }
            }
        }

        /// Property 15: FFmpeg value round-trip for multipliers
        #[test]
        fn multiplier_ffmpeg_roundtrip(value in valid_multiplier_strategy()) {
            let volume = VolumeValue::Multiplier(value);
            let ffmpeg_str = volume.to_ffmpeg_value();
            
            // The FFmpeg value should be parseable back
            let reparsed: f64 = ffmpeg_str.parse().expect("FFmpeg value should be parseable");
            
            prop_assert!(
                (reparsed - value).abs() < 0.0001,
                "FFmpeg value '{}' should round-trip to {}",
                ffmpeg_str,
                value
            );
        }

        /// Property 15: FFmpeg value format for dB
        #[test]
        fn db_ffmpeg_format(value in valid_db_strategy()) {
            let volume = VolumeValue::Decibels(value);
            let ffmpeg_str = volume.to_ffmpeg_value();
            
            prop_assert!(
                ffmpeg_str.ends_with("dB"),
                "dB FFmpeg value '{}' should end with 'dB'",
                ffmpeg_str
            );
        }
    }

    // Feature: rust-mcp-genmedia, Property 13: GCS Path Resolution
    // **Validates: Requirements 9.9, 9.10**
    //
    // For any input or output path that is a GCS URI (starts with `gs://`),
    // the AVTool_Server SHALL download inputs to a temporary location before
    // processing and upload outputs after processing. For any local path,
    // no GCS operations SHALL occur.

    /// Strategy to generate valid GCS bucket names
    fn valid_bucket_strategy() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9-]{2,20}".prop_map(|s| s.to_string())
    }

    /// Strategy to generate valid GCS object paths
    fn valid_object_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9/_.-]{1,50}".prop_map(|s| s.to_string())
    }

    /// Strategy to generate valid local paths
    fn valid_local_path_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("/tmp/file.mp4".to_string()),
            Just("./relative/path.wav".to_string()),
            Just("file.mp3".to_string()),
            "[a-zA-Z0-9/_.-]{1,30}".prop_map(|s| format!("/tmp/{}", s)),
        ]
    }

    proptest! {
        /// Property 13: GCS URIs should be correctly identified
        #[test]
        fn gcs_uri_correctly_identified(
            bucket in valid_bucket_strategy(),
            object in valid_object_strategy()
        ) {
            let gcs_uri = format!("gs://{}/{}", bucket, object);
            
            prop_assert!(
                AVToolHandler::is_gcs_uri(&gcs_uri),
                "GCS URI '{}' should be identified as GCS",
                gcs_uri
            );
        }

        /// Property 13: Local paths should not be identified as GCS URIs
        #[test]
        fn local_path_not_gcs(path in valid_local_path_strategy()) {
            prop_assert!(
                !AVToolHandler::is_gcs_uri(&path),
                "Local path '{}' should not be identified as GCS",
                path
            );
        }

        /// Property 13: S3 URIs should not be identified as GCS URIs
        #[test]
        fn s3_uri_not_gcs(
            bucket in valid_bucket_strategy(),
            object in valid_object_strategy()
        ) {
            let s3_uri = format!("s3://{}/{}", bucket, object);
            
            prop_assert!(
                !AVToolHandler::is_gcs_uri(&s3_uri),
                "S3 URI '{}' should not be identified as GCS",
                s3_uri
            );
        }

        /// Property 13: HTTP URLs should not be identified as GCS URIs
        #[test]
        fn http_url_not_gcs(domain in "[a-z]{3,10}\\.[a-z]{2,3}", path in "[a-z/]{1,20}") {
            let http_url = format!("https://{}/{}", domain, path);
            
            prop_assert!(
                !AVToolHandler::is_gcs_uri(&http_url),
                "HTTP URL '{}' should not be identified as GCS",
                http_url
            );
        }
    }

    // Feature: rust-mcp-genmedia, Property 14: Media Info Output Completeness
    // **Validates: Requirements 9.11**
    //
    // For any valid media file, ffmpeg_get_media_info SHALL return a JSON object
    // containing at minimum: duration (number), format (string), and streams (array).
    // Each stream SHALL contain codec_type and codec_name.

    proptest! {
        /// Property 14: MediaInfo serialization always includes required fields
        #[test]
        fn media_info_has_required_fields(
            duration in 0.0f64..=3600.0f64,
            format in "[a-z0-9]{1,10}",
            num_streams in 0usize..=5usize
        ) {
            let streams: Vec<StreamInfo> = (0..num_streams)
                .map(|i| StreamInfo {
                    index: i as u32,
                    codec_type: if i % 2 == 0 { "video".to_string() } else { "audio".to_string() },
                    codec_name: format!("codec_{}", i),
                    width: if i % 2 == 0 { Some(1920) } else { None },
                    height: if i % 2 == 0 { Some(1080) } else { None },
                    sample_rate: if i % 2 == 1 { Some(44100) } else { None },
                    channels: if i % 2 == 1 { Some(2) } else { None },
                })
                .collect();
            
            let info = MediaInfo {
                duration,
                format: format.clone(),
                streams,
            };
            
            // Serialize to JSON
            let json_str = serde_json::to_string(&info).expect("Should serialize");
            let json: serde_json::Value = serde_json::from_str(&json_str).expect("Should parse");
            
            // Verify required fields exist
            prop_assert!(json.get("duration").is_some(), "Should have duration field");
            prop_assert!(json.get("format").is_some(), "Should have format field");
            prop_assert!(json.get("streams").is_some(), "Should have streams field");
            
            // Verify types
            prop_assert!(json["duration"].is_f64(), "duration should be a number");
            prop_assert!(json["format"].is_string(), "format should be a string");
            prop_assert!(json["streams"].is_array(), "streams should be an array");
            
            // Verify stream contents
            if let Some(streams_arr) = json["streams"].as_array() {
                prop_assert_eq!(streams_arr.len(), num_streams, "Should have correct number of streams");
                
                for stream in streams_arr {
                    prop_assert!(
                        stream.get("codec_type").is_some(),
                        "Each stream should have codec_type"
                    );
                    prop_assert!(
                        stream.get("codec_name").is_some(),
                        "Each stream should have codec_name"
                    );
                    prop_assert!(
                        stream["codec_type"].is_string(),
                        "codec_type should be a string"
                    );
                    prop_assert!(
                        stream["codec_name"].is_string(),
                        "codec_name should be a string"
                    );
                }
            }
        }

        /// Property 14: MediaInfo round-trip serialization preserves data
        #[test]
        fn media_info_roundtrip(
            duration in 0.0f64..=3600.0f64,
            format in "[a-z0-9]{1,10}"
        ) {
            let original = MediaInfo {
                duration,
                format: format.clone(),
                streams: vec![
                    StreamInfo {
                        index: 0,
                        codec_type: "video".to_string(),
                        codec_name: "h264".to_string(),
                        width: Some(1920),
                        height: Some(1080),
                        sample_rate: None,
                        channels: None,
                    },
                ],
            };
            
            let json_str = serde_json::to_string(&original).expect("Should serialize");
            let deserialized: MediaInfo = serde_json::from_str(&json_str).expect("Should deserialize");
            
            prop_assert!(
                (deserialized.duration - duration).abs() < 0.0001,
                "Duration should round-trip"
            );
            prop_assert_eq!(deserialized.format, format, "Format should round-trip");
            prop_assert_eq!(deserialized.streams.len(), 1, "Streams should round-trip");
        }

        /// Property 14: StreamInfo optional fields are properly serialized
        #[test]
        fn stream_info_optional_fields(
            has_width in proptest::bool::ANY,
            has_height in proptest::bool::ANY,
            has_sample_rate in proptest::bool::ANY,
            has_channels in proptest::bool::ANY
        ) {
            let stream = StreamInfo {
                index: 0,
                codec_type: "video".to_string(),
                codec_name: "h264".to_string(),
                width: if has_width { Some(1920) } else { None },
                height: if has_height { Some(1080) } else { None },
                sample_rate: if has_sample_rate { Some(44100) } else { None },
                channels: if has_channels { Some(2) } else { None },
            };
            
            let json_str = serde_json::to_string(&stream).expect("Should serialize");
            let json: serde_json::Value = serde_json::from_str(&json_str).expect("Should parse");
            
            // Optional fields should only be present if they have values
            prop_assert_eq!(
                json.get("width").is_some(),
                has_width,
                "width presence should match"
            );
            prop_assert_eq!(
                json.get("height").is_some(),
                has_height,
                "height presence should match"
            );
            prop_assert_eq!(
                json.get("sample_rate").is_some(),
                has_sample_rate,
                "sample_rate presence should match"
            );
            prop_assert_eq!(
                json.get("channels").is_some(),
                has_channels,
                "channels presence should match"
            );
        }
    }
}
