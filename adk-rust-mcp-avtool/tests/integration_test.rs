//! Integration tests for adk-rust-mcp-avtool server.
//!
//! These tests require FFmpeg and FFprobe to be installed on the system.
//!
//! Run with: `cargo test --package adk-rust-mcp-avtool --test integration_test`
//! Skip in CI: `cargo test --package adk-rust-mcp-avtool --lib`
//!
//! Generated media files are saved to `./test_output/` directory for inspection.

use adk_rust_mcp_common::config::Config;
use adk_rust_mcp_avtool::{
    AVToolHandler, GetMediaInfoParams, ConvertAudioParams, VideoToGifParams,
    CombineAvParams, OverlayImageParams, ConcatenateParams, AdjustVolumeParams,
    LayerAudioParams, AudioLayer,
};
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Once;

static INIT: Once = Once::new();

/// Output directory for test-generated media
const TEST_OUTPUT_DIR: &str = "test_output";

/// Initialize environment from .env file once
fn init_env() {
    INIT.call_once(|| {
        let _ = dotenvy::dotenv();
    });
}

/// Check if FFmpeg is available on the system.
fn ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if FFprobe is available on the system.
fn ffprobe_available() -> bool {
    Command::new("ffprobe")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if integration tests should run.
fn should_run_integration_tests() -> bool {
    if env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        return false;
    }
    ffmpeg_available() && ffprobe_available()
}

/// Macro to skip test if integration tests are disabled.
macro_rules! skip_if_no_integration {
    () => {
        if !should_run_integration_tests() {
            eprintln!("Skipping integration test: FFmpeg/FFprobe not available");
            return;
        }
    };
}

/// Get the test output directory (absolute path).
fn get_test_output_dir() -> PathBuf {
    let dir = std::env::current_dir()
        .expect("Failed to get current directory")
        .join(TEST_OUTPUT_DIR);
    if !dir.exists() {
        std::fs::create_dir_all(&dir).expect("Failed to create test output directory");
    }
    dir
}

/// Get test configuration.
fn get_test_config() -> Config {
    init_env();
    Config {
        project_id: env::var("PROJECT_ID").unwrap_or_else(|_| "test-project".to_string()),
        location: env::var("LOCATION").unwrap_or_else(|_| "us-central1".to_string()),
        gcs_bucket: env::var("GCS_BUCKET").ok(),
        port: 8080,
    }
}

/// Generate a simple UUID for test uniqueness.
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", timestamp)
}

/// Create a simple test WAV file using FFmpeg.
fn create_test_wav(path: &PathBuf, duration: f32) -> bool {
    Command::new("ffmpeg")
        .args([
            "-y",
            "-f", "lavfi",
            "-i", &format!("sine=frequency=440:duration={}", duration),
            "-ac", "2",
            path.to_str().unwrap(),
        ])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Create a test WAV file with a specific frequency.
fn create_test_wav_freq(path: &PathBuf, frequency: u32, duration: f32) -> bool {
    Command::new("ffmpeg")
        .args([
            "-y",
            "-f", "lavfi",
            "-i", &format!("sine=frequency={}:duration={}", frequency, duration),
            "-ac", "2",
            path.to_str().unwrap(),
        ])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Create a simple test video file using FFmpeg.
fn create_test_video(path: &PathBuf, duration: f32) -> bool {
    Command::new("ffmpeg")
        .args([
            "-y",
            "-f", "lavfi",
            "-i", &format!("testsrc=duration={}:size=320x240:rate=10", duration),
            "-f", "lavfi",
            "-i", &format!("sine=frequency=440:duration={}", duration),
            "-c:v", "libx264",
            "-c:a", "aac",
            "-pix_fmt", "yuv420p",
            path.to_str().unwrap(),
        ])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Create a test video without audio.
fn create_test_video_no_audio(path: &PathBuf, duration: f32) -> bool {
    Command::new("ffmpeg")
        .args([
            "-y",
            "-f", "lavfi",
            "-i", &format!("testsrc=duration={}:size=320x240:rate=10", duration),
            "-c:v", "libx264",
            "-pix_fmt", "yuv420p",
            path.to_str().unwrap(),
        ])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Create a test PNG image.
fn create_test_image(path: &PathBuf, width: u32, height: u32) -> bool {
    Command::new("ffmpeg")
        .args([
            "-y",
            "-f", "lavfi",
            "-i", &format!("color=c=red:s={}x{}:d=1", width, height),
            "-frames:v", "1",
            path.to_str().unwrap(),
        ])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// =============================================================================
// Handler Creation Tests
// =============================================================================

#[tokio::test]
async fn test_handler_creation() {
    skip_if_no_integration!();
    
    let config = get_test_config();
    let handler = AVToolHandler::new(config).await;
    
    assert!(handler.is_ok(), "Handler creation should succeed: {:?}", handler.err());
}

// =============================================================================
// Media Info Tests (Requirement 9.11)
// =============================================================================

#[tokio::test]
async fn test_get_media_info_audio() {
    skip_if_no_integration!();
    
    let output_dir = get_test_output_dir();
    let test_wav = output_dir.join(format!("test_info_audio_{}.wav", uuid_v4()));
    
    // Create test WAV file
    assert!(create_test_wav(&test_wav, 2.0), "Failed to create test WAV file");
    
    let config = get_test_config();
    let handler = AVToolHandler::new(config).await.expect("Failed to create handler");
    
    let params = GetMediaInfoParams {
        input: test_wav.to_string_lossy().to_string(),
    };
    
    let result = handler.get_media_info(params).await;
    assert!(result.is_ok(), "get_media_info should succeed: {:?}", result.err());
    
    let info = result.unwrap();
    assert!(info.duration > 1.5 && info.duration < 2.5, "Duration should be ~2 seconds: {}", info.duration);
    assert!(!info.streams.is_empty(), "Should have at least one stream");
    
    let audio_stream = info.streams.iter().find(|s| s.codec_type == "audio");
    assert!(audio_stream.is_some(), "Should have audio stream");
    
    eprintln!("Media info: duration={:.2}s, format={}, streams={}", 
              info.duration, info.format, info.streams.len());
    
    // Keep file for inspection
    eprintln!("Test file saved: {}", test_wav.display());
}

#[tokio::test]
async fn test_get_media_info_video() {
    skip_if_no_integration!();
    
    let output_dir = get_test_output_dir();
    let test_video = output_dir.join(format!("test_info_video_{}.mp4", uuid_v4()));
    
    // Create test video file
    assert!(create_test_video(&test_video, 3.0), "Failed to create test video file");
    
    let config = get_test_config();
    let handler = AVToolHandler::new(config).await.expect("Failed to create handler");
    
    let params = GetMediaInfoParams {
        input: test_video.to_string_lossy().to_string(),
    };
    
    let result = handler.get_media_info(params).await;
    assert!(result.is_ok(), "get_media_info should succeed: {:?}", result.err());
    
    let info = result.unwrap();
    assert!(info.duration > 2.5 && info.duration < 3.5, "Duration should be ~3 seconds: {}", info.duration);
    
    // Should have both video and audio streams
    let video_stream = info.streams.iter().find(|s| s.codec_type == "video");
    let audio_stream = info.streams.iter().find(|s| s.codec_type == "audio");
    
    assert!(video_stream.is_some(), "Should have video stream");
    assert!(audio_stream.is_some(), "Should have audio stream");
    
    let video = video_stream.unwrap();
    assert_eq!(video.width, Some(320), "Video width should be 320");
    assert_eq!(video.height, Some(240), "Video height should be 240");
    
    eprintln!("Video info: duration={:.2}s, format={}, {}x{}", 
              info.duration, info.format, 
              video.width.unwrap_or(0), video.height.unwrap_or(0));
    
    // Keep file for inspection
    eprintln!("Test file saved: {}", test_video.display());
}

// =============================================================================
// Audio Conversion Tests (Requirement 9.2)
// =============================================================================

#[tokio::test]
async fn test_convert_wav_to_mp3() {
    skip_if_no_integration!();
    
    let output_dir = get_test_output_dir();
    let id = uuid_v4();
    let test_wav = output_dir.join(format!("convert_input_{}.wav", id));
    let output_mp3 = output_dir.join(format!("convert_output_{}.mp3", id));
    
    // Create test WAV file
    assert!(create_test_wav(&test_wav, 2.0), "Failed to create test WAV file");
    
    let config = get_test_config();
    let handler = AVToolHandler::new(config).await.expect("Failed to create handler");
    
    let params = ConvertAudioParams {
        input: test_wav.to_string_lossy().to_string(),
        output: output_mp3.to_string_lossy().to_string(),
        bitrate: "192k".to_string(),
    };
    
    let result = handler.convert_wav_to_mp3(params).await;
    assert!(result.is_ok(), "convert_wav_to_mp3 should succeed: {:?}", result.err());
    
    // Verify output file exists and is valid
    assert!(output_mp3.exists(), "Output MP3 should exist");
    let metadata = std::fs::metadata(&output_mp3).expect("Should read metadata");
    assert!(metadata.len() > 1000, "MP3 should have reasonable size: {} bytes", metadata.len());
    
    eprintln!("Converted WAV to MP3: {} ({} bytes)", output_mp3.display(), metadata.len());
}

// =============================================================================
// Video to GIF Tests (Requirement 9.3)
// =============================================================================

#[tokio::test]
async fn test_video_to_gif() {
    skip_if_no_integration!();
    
    let output_dir = get_test_output_dir();
    let id = uuid_v4();
    let test_video = output_dir.join(format!("gif_input_{}.mp4", id));
    let output_gif = output_dir.join(format!("gif_output_{}.gif", id));
    
    // Create test video file
    assert!(create_test_video(&test_video, 2.0), "Failed to create test video file");
    
    let config = get_test_config();
    let handler = AVToolHandler::new(config).await.expect("Failed to create handler");
    
    let params = VideoToGifParams {
        input: test_video.to_string_lossy().to_string(),
        output: output_gif.to_string_lossy().to_string(),
        fps: 10,
        width: Some(160),
        start_time: None,
        duration: Some(1.0),
    };
    
    let result = handler.video_to_gif(params).await;
    assert!(result.is_ok(), "video_to_gif should succeed: {:?}", result.err());
    
    // Verify output file exists
    assert!(output_gif.exists(), "Output GIF should exist");
    let metadata = std::fs::metadata(&output_gif).expect("Should read metadata");
    assert!(metadata.len() > 1000, "GIF should have reasonable size: {} bytes", metadata.len());
    
    eprintln!("Converted video to GIF: {} ({} bytes)", output_gif.display(), metadata.len());
}

// =============================================================================
// Combine Audio and Video Tests (Requirement 9.4)
// =============================================================================

#[tokio::test]
async fn test_combine_audio_video() {
    skip_if_no_integration!();
    
    let output_dir = get_test_output_dir();
    let id = uuid_v4();
    let test_video = output_dir.join(format!("combine_video_{}.mp4", id));
    let test_audio = output_dir.join(format!("combine_audio_{}.wav", id));
    let output_combined = output_dir.join(format!("combine_output_{}.mp4", id));
    
    // Create test video (without audio) and audio files
    assert!(create_test_video_no_audio(&test_video, 3.0), "Failed to create test video");
    assert!(create_test_wav(&test_audio, 3.0), "Failed to create test audio");
    
    let config = get_test_config();
    let handler = AVToolHandler::new(config).await.expect("Failed to create handler");
    
    let params = CombineAvParams {
        video_input: test_video.to_string_lossy().to_string(),
        audio_input: test_audio.to_string_lossy().to_string(),
        output: output_combined.to_string_lossy().to_string(),
    };
    
    let result = handler.combine_audio_video(params).await;
    assert!(result.is_ok(), "combine_audio_video should succeed: {:?}", result.err());
    
    // Verify output has both video and audio
    assert!(output_combined.exists(), "Output should exist");
    
    let info_params = GetMediaInfoParams {
        input: output_combined.to_string_lossy().to_string(),
    };
    let info = handler.get_media_info(info_params).await.expect("Should get info");
    
    let has_video = info.streams.iter().any(|s| s.codec_type == "video");
    let has_audio = info.streams.iter().any(|s| s.codec_type == "audio");
    
    assert!(has_video, "Combined file should have video");
    assert!(has_audio, "Combined file should have audio");
    
    eprintln!("Combined audio and video: {}", output_combined.display());
}

// =============================================================================
// Overlay Image Tests (Requirement 9.5)
// =============================================================================

#[tokio::test]
async fn test_overlay_image_on_video() {
    skip_if_no_integration!();
    
    let output_dir = get_test_output_dir();
    let id = uuid_v4();
    let test_video = output_dir.join(format!("overlay_video_{}.mp4", id));
    let test_image = output_dir.join(format!("overlay_image_{}.png", id));
    let output_overlay = output_dir.join(format!("overlay_output_{}.mp4", id));
    
    // Create test video and image
    assert!(create_test_video(&test_video, 3.0), "Failed to create test video");
    assert!(create_test_image(&test_image, 50, 50), "Failed to create test image");
    
    let config = get_test_config();
    let handler = AVToolHandler::new(config).await.expect("Failed to create handler");
    
    let params = OverlayImageParams {
        video_input: test_video.to_string_lossy().to_string(),
        image_input: test_image.to_string_lossy().to_string(),
        output: output_overlay.to_string_lossy().to_string(),
        x: 10,
        y: 10,
        scale: Some(0.5),
        start_time: Some(0.5),
        duration: Some(2.0),
    };
    
    let result = handler.overlay_image(params).await;
    assert!(result.is_ok(), "overlay_image should succeed: {:?}", result.err());
    
    // Verify output exists
    assert!(output_overlay.exists(), "Output should exist");
    let metadata = std::fs::metadata(&output_overlay).expect("Should read metadata");
    assert!(metadata.len() > 10000, "Output should have reasonable size");
    
    eprintln!("Overlaid image on video: {} ({} bytes)", output_overlay.display(), metadata.len());
}

// =============================================================================
// Concatenate Media Tests (Requirement 9.6)
// =============================================================================

#[tokio::test]
async fn test_concatenate_videos() {
    skip_if_no_integration!();
    
    let output_dir = get_test_output_dir();
    let id = uuid_v4();
    let video1 = output_dir.join(format!("concat_video1_{}.mp4", id));
    let video2 = output_dir.join(format!("concat_video2_{}.mp4", id));
    let output_concat = output_dir.join(format!("concat_output_{}.mp4", id));
    
    // Create two test videos
    assert!(create_test_video(&video1, 2.0), "Failed to create video 1");
    assert!(create_test_video(&video2, 2.0), "Failed to create video 2");
    
    let config = get_test_config();
    let handler = AVToolHandler::new(config).await.expect("Failed to create handler");
    
    let params = ConcatenateParams {
        inputs: vec![
            video1.to_string_lossy().to_string(),
            video2.to_string_lossy().to_string(),
        ],
        output: output_concat.to_string_lossy().to_string(),
    };
    
    let result = handler.concatenate(params).await;
    assert!(result.is_ok(), "concatenate should succeed: {:?}", result.err());
    
    // Verify output duration is approximately sum of inputs
    let info_params = GetMediaInfoParams {
        input: output_concat.to_string_lossy().to_string(),
    };
    let info = handler.get_media_info(info_params).await.expect("Should get info");
    
    assert!(info.duration > 3.5 && info.duration < 4.5, 
            "Concatenated duration should be ~4 seconds: {}", info.duration);
    
    eprintln!("Concatenated videos: {} (duration: {:.2}s)", output_concat.display(), info.duration);
}

// =============================================================================
// Volume Adjustment Tests (Requirement 9.7)
// =============================================================================

#[tokio::test]
async fn test_adjust_volume_multiplier() {
    skip_if_no_integration!();
    
    let output_dir = get_test_output_dir();
    let id = uuid_v4();
    let test_wav = output_dir.join(format!("volume_input_{}.wav", id));
    let output_wav = output_dir.join(format!("volume_output_mult_{}.wav", id));
    
    // Create test WAV file
    assert!(create_test_wav(&test_wav, 2.0), "Failed to create test WAV file");
    
    let config = get_test_config();
    let handler = AVToolHandler::new(config).await.expect("Failed to create handler");
    
    let params = AdjustVolumeParams {
        input: test_wav.to_string_lossy().to_string(),
        output: output_wav.to_string_lossy().to_string(),
        volume: "0.5".to_string(),
    };
    
    let result = handler.adjust_volume(params).await;
    assert!(result.is_ok(), "adjust_volume should succeed: {:?}", result.err());
    
    assert!(output_wav.exists(), "Output should exist");
    eprintln!("Adjusted volume (0.5x): {}", output_wav.display());
}

#[tokio::test]
async fn test_adjust_volume_db() {
    skip_if_no_integration!();
    
    let output_dir = get_test_output_dir();
    let id = uuid_v4();
    let test_wav = output_dir.join(format!("volume_db_input_{}.wav", id));
    let output_wav = output_dir.join(format!("volume_db_output_{}.wav", id));
    
    // Create test WAV file
    assert!(create_test_wav(&test_wav, 2.0), "Failed to create test WAV file");
    
    let config = get_test_config();
    let handler = AVToolHandler::new(config).await.expect("Failed to create handler");
    
    let params = AdjustVolumeParams {
        input: test_wav.to_string_lossy().to_string(),
        output: output_wav.to_string_lossy().to_string(),
        volume: "-6dB".to_string(),
    };
    
    let result = handler.adjust_volume(params).await;
    assert!(result.is_ok(), "adjust_volume with dB should succeed: {:?}", result.err());
    
    assert!(output_wav.exists(), "Output should exist");
    eprintln!("Adjusted volume (-6dB): {}", output_wav.display());
}

// =============================================================================
// Layer Audio Tests (Requirement 9.8)
// =============================================================================

#[tokio::test]
async fn test_layer_audio_files() {
    skip_if_no_integration!();
    
    let output_dir = get_test_output_dir();
    let id = uuid_v4();
    let audio1 = output_dir.join(format!("layer_audio1_{}.wav", id));
    let audio2 = output_dir.join(format!("layer_audio2_{}.wav", id));
    let output_mixed = output_dir.join(format!("layer_output_{}.wav", id));
    
    // Create two test audio files with different frequencies
    assert!(create_test_wav_freq(&audio1, 440, 3.0), "Failed to create audio 1 (440Hz)");
    assert!(create_test_wav_freq(&audio2, 880, 3.0), "Failed to create audio 2 (880Hz)");
    
    let config = get_test_config();
    let handler = AVToolHandler::new(config).await.expect("Failed to create handler");
    
    let params = LayerAudioParams {
        inputs: vec![
            AudioLayer {
                path: audio1.to_string_lossy().to_string(),
                offset_seconds: 0.0,
                volume: 1.0,
            },
            AudioLayer {
                path: audio2.to_string_lossy().to_string(),
                offset_seconds: 1.0, // Start 1 second later
                volume: 0.5,         // Half volume
            },
        ],
        output: output_mixed.to_string_lossy().to_string(),
    };
    
    let result = handler.layer_audio(params).await;
    assert!(result.is_ok(), "layer_audio should succeed: {:?}", result.err());
    
    // Verify output exists and has reasonable duration
    assert!(output_mixed.exists(), "Output should exist");
    
    let info_params = GetMediaInfoParams {
        input: output_mixed.to_string_lossy().to_string(),
    };
    let info = handler.get_media_info(info_params).await.expect("Should get info");
    
    // Duration should be max of (audio1 duration, audio2 offset + duration) = max(3, 1+3) = 4
    assert!(info.duration > 3.5 && info.duration < 4.5, 
            "Mixed duration should be ~4 seconds: {}", info.duration);
    
    eprintln!("Layered audio files: {} (duration: {:.2}s)", output_mixed.display(), info.duration);
}

// =============================================================================
// Error Handling Tests (Requirements 9.19, 9.20)
// =============================================================================

#[tokio::test]
async fn test_get_media_info_nonexistent_file() {
    skip_if_no_integration!();
    
    let config = get_test_config();
    let handler = AVToolHandler::new(config).await.expect("Failed to create handler");
    
    let params = GetMediaInfoParams {
        input: "/nonexistent/path/to/file.mp4".to_string(),
    };
    
    let result = handler.get_media_info(params).await;
    assert!(result.is_err(), "Should fail for nonexistent file");
    
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("FFmpeg") || err_msg.contains("ffprobe") || err_msg.contains("No such file"),
        "Error should mention FFmpeg/ffprobe: {}", err_msg
    );
}

#[tokio::test]
async fn test_convert_invalid_input() {
    skip_if_no_integration!();
    
    let output_dir = get_test_output_dir();
    let id = uuid_v4();
    let invalid_file = output_dir.join(format!("invalid_{}.txt", id));
    let output_mp3 = output_dir.join(format!("invalid_output_{}.mp3", id));
    
    // Create an invalid "media" file
    std::fs::write(&invalid_file, "This is not a valid media file").unwrap();
    
    let config = get_test_config();
    let handler = AVToolHandler::new(config).await.expect("Failed to create handler");
    
    let params = ConvertAudioParams {
        input: invalid_file.to_string_lossy().to_string(),
        output: output_mp3.to_string_lossy().to_string(),
        bitrate: "192k".to_string(),
    };
    
    let result = handler.convert_wav_to_mp3(params).await;
    assert!(result.is_err(), "Should fail for invalid input");
    
    // Cleanup
    let _ = std::fs::remove_file(&invalid_file);
}

#[tokio::test]
async fn test_adjust_volume_invalid_volume() {
    skip_if_no_integration!();
    
    let output_dir = get_test_output_dir();
    let id = uuid_v4();
    let test_wav = output_dir.join(format!("invalid_vol_input_{}.wav", id));
    let output_wav = output_dir.join(format!("invalid_vol_output_{}.wav", id));
    
    // Create test WAV file
    assert!(create_test_wav(&test_wav, 1.0), "Failed to create test WAV file");
    
    let config = get_test_config();
    let handler = AVToolHandler::new(config).await.expect("Failed to create handler");
    
    let params = AdjustVolumeParams {
        input: test_wav.to_string_lossy().to_string(),
        output: output_wav.to_string_lossy().to_string(),
        volume: "invalid_volume".to_string(),
    };
    
    let result = handler.adjust_volume(params).await;
    assert!(result.is_err(), "Should fail for invalid volume");
    
    // Cleanup
    let _ = std::fs::remove_file(&test_wav);
}

#[tokio::test]
async fn test_concatenate_empty_inputs() {
    skip_if_no_integration!();
    
    let output_dir = get_test_output_dir();
    let output = output_dir.join("empty_concat.mp4");
    
    let config = get_test_config();
    let handler = AVToolHandler::new(config).await.expect("Failed to create handler");
    
    let params = ConcatenateParams {
        inputs: vec![], // Empty inputs
        output: output.to_string_lossy().to_string(),
    };
    
    let result = handler.concatenate(params).await;
    assert!(result.is_err(), "Should fail for empty inputs");
}
