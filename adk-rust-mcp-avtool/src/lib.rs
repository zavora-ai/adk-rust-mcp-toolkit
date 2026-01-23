//! ADK Rust MCP AVTool Library
//!
//! MCP server for audio/video processing using FFmpeg.
//!
//! This crate provides FFmpeg-based media processing tools exposed via MCP:
//! - `ffmpeg_get_media_info` - Get media file information
//! - `ffmpeg_convert_audio_wav_to_mp3` - Convert WAV to MP3
//! - `ffmpeg_video_to_gif` - Convert video to GIF
//! - `ffmpeg_combine_audio_and_video` - Combine audio and video tracks
//! - `ffmpeg_overlay_image_on_video` - Overlay image on video
//! - `ffmpeg_concatenate_media_files` - Concatenate media files
//! - `ffmpeg_adjust_volume` - Adjust audio volume
//! - `ffmpeg_layer_audio_files` - Layer/mix multiple audio files

pub mod handler;
pub mod server;

pub use handler::{
    AVToolHandler,
    AdjustVolumeParams,
    AudioLayer,
    CombineAvParams,
    ConcatenateParams,
    ConvertAudioParams,
    GetMediaInfoParams,
    LayerAudioParams,
    MediaInfo,
    OverlayImageParams,
    StreamInfo,
    VideoToGifParams,
    VolumeValue,
};
pub use server::AVToolServer;
