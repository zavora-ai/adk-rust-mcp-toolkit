//! ADK Rust MCP Artist — Artistic image creation and style transfer.

pub mod create;
pub mod server;
pub mod sketch_to_art;
pub mod style_transfer;
pub mod variations;

pub use server::ArtistServer;

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Load an image from a file path and return (base64_data, mime_type).
/// If the input is already base64 (no file extension), returns it as-is with image/png.
pub async fn load_image_as_base64(path: &str) -> Result<(String, String), String> {
    if std::path::Path::new(path).exists() {
        let data = tokio::fs::read(path).await.map_err(|e| e.to_string())?;
        let mime = if path.ends_with(".png") { "image/png" } else { "image/jpeg" };
        Ok((BASE64.encode(&data), mime.to_string()))
    } else {
        // Assume it's already base64
        Ok((path.to_string(), "image/png".to_string()))
    }
}

/// Call Gemini generateContent and extract the first image from the response.
pub async fn call_gemini_image(
    config: &adk_rust_mcp_common::Config,
    model: &str,
    parts: Vec<serde_json::Value>,
    aspect_ratio: Option<&str>,
    resolution: Option<&str>,
) -> Result<Vec<u8>, String> {
    let api_key = config.gemini_api_key.as_deref().ok_or("GEMINI_API_KEY required")?;
    let url = format!("{}/models/{}:generateContent", config.gemini_base_url(), model);

    let mut gen_config = serde_json::json!({
        "responseModalities": ["IMAGE", "TEXT"],
    });
    if aspect_ratio.is_some() || resolution.is_some() {
        let mut img_cfg = serde_json::Map::new();
        if let Some(ar) = aspect_ratio {
            img_cfg.insert("aspectRatio".into(), serde_json::json!(ar));
        }
        if let Some(res) = resolution {
            img_cfg.insert("imageSize".into(), serde_json::json!(res));
        }
        gen_config.as_object_mut().unwrap().insert("imageConfig".into(), serde_json::Value::Object(img_cfg));
    }

    let body = serde_json::json!({
        "contents": [{"parts": parts}],
        "generationConfig": gen_config,
    });

    let resp = reqwest::Client::new()
        .post(&url)
        .header("x-goog-api-key", api_key)
        .json(&body)
        .send().await.map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("Gemini API error: {}", resp.text().await.unwrap_or_default()));
    }

    let response: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let image_data = response.pointer("/candidates/0/content/parts")
        .and_then(|parts| parts.as_array())
        .and_then(|parts| parts.iter().find_map(|p| p.pointer("/inlineData/data").and_then(|d| d.as_str())))
        .ok_or("No image in response")?;

    BASE64.decode(image_data).map_err(|e| e.to_string())
}

/// Save image bytes to a file, creating parent dirs as needed.
pub async fn save_image(data: &[u8], path: &str) -> Result<String, String> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
    }
    tokio::fs::write(path, data).await.map_err(|e| e.to_string())?;
    Ok(format!("Saved to: {}", path))
}
