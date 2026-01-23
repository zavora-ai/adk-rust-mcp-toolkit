//! Debug script to see actual API responses

use adk_rust_mcp_common::auth::AuthProvider;
use adk_rust_mcp_common::config::Config;
use std::env;
use std::sync::Once;

static INIT: Once = Once::new();

fn init_env() {
    INIT.call_once(|| {
        let _ = dotenvy::dotenv();
    });
}

fn get_test_config() -> Option<Config> {
    init_env();
    let project_id = env::var("PROJECT_ID").ok()?;
    Some(Config {
        project_id,
        location: env::var("LOCATION").unwrap_or_else(|_| "us-central1".to_string()),
        gcs_bucket: env::var("GCS_BUCKET").ok(),
        port: 8080,
    })
}

#[tokio::test]
async fn debug_image_api_response() {
    let config = match get_test_config() {
        Some(c) => c,
        None => {
            eprintln!("No config available");
            return;
        }
    };

    let auth = AuthProvider::new().await.expect("Failed to create auth");
    let http = reqwest::Client::new();

    let endpoint = format!(
        "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/gemini-2.0-flash-preview-image-generation:generateContent",
        config.location,
        config.project_id,
        config.location,
    );

    let request_body = serde_json::json!({
        "contents": [{
            "role": "user",
            "parts": [{"text": "A simple red circle"}]
        }],
        "generationConfig": {
            "responseModalities": ["TEXT", "IMAGE"]
        }
    });

    let token = auth
        .get_token(&["https://www.googleapis.com/auth/cloud-platform"])
        .await
        .expect("Failed to get token");

    let response = http
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .expect("Request failed");

    let status = response.status();
    let body = response.text().await.expect("Failed to read body");

    println!("Status: {}", status);
    println!("Response body:\n{}", body);
}

#[tokio::test]
async fn debug_tts_api_response() {
    let config = match get_test_config() {
        Some(c) => c,
        None => {
            eprintln!("No config available");
            return;
        }
    };

    let auth = AuthProvider::new().await.expect("Failed to create auth");
    let http = reqwest::Client::new();

    let endpoint = format!(
        "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/gemini-2.5-flash-preview-tts:generateContent",
        config.location,
        config.project_id,
        config.location,
    );

    let request_body = serde_json::json!({
        "contents": [{
            "role": "user",
            "parts": [{"text": "Hello world"}]
        }],
        "generationConfig": {
            "responseModalities": ["AUDIO"],
            "speechConfig": {
                "voiceConfig": {
                    "prebuiltVoiceConfig": {
                        "voiceName": "Kore"
                    }
                }
            }
        }
    });

    let token = auth
        .get_token(&["https://www.googleapis.com/auth/cloud-platform"])
        .await
        .expect("Failed to get token");

    let response = http
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .expect("Request failed");

    let status = response.status();
    let body = response.text().await.expect("Failed to read body");

    println!("Status: {}", status);
    println!("Response body:\n{}", body);
}
