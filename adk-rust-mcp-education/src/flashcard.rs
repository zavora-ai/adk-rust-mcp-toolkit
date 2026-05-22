//! Flashcard generation: Gemini text for Q&A + image gen for card fronts.

use adk_rust_mcp_common::Config;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FlashcardParams {
    /// Subject to create flashcards for
    pub topic: String,
    /// Number of cards (1-20)
    #[serde(default = "default_count")]
    pub count: u8,
    /// Difficulty: easy, medium, hard
    #[serde(default = "default_difficulty")]
    pub difficulty: String,
    /// Target age range
    #[serde(default = "default_age")]
    pub age_group: String,
    /// Generate images for card fronts
    #[serde(default = "default_true")]
    pub include_images: bool,
    /// Directory to save cards
    #[serde(default)]
    pub output_dir: Option<String>,
}

fn default_count() -> u8 { 5 }
fn default_difficulty() -> String { "easy".into() }
fn default_age() -> String { "8-10".into() }
fn default_true() -> bool { true }

pub async fn generate(config: &Config, params: FlashcardParams) -> Result<String, String> {
    let api_key = config.gemini_api_key.as_deref().ok_or("GEMINI_API_KEY required")?;
    let client = reqwest::Client::new();
    let base = config.gemini_base_url();
    let out_dir = PathBuf::from(params.output_dir.unwrap_or_else(|| "flashcards".into()));
    tokio::fs::create_dir_all(&out_dir).await.map_err(|e| e.to_string())?;

    info!(topic = %params.topic, count = params.count, "Generating flashcards");

    // Step 1: Generate Q&A pairs with Gemini text
    let text_url = format!("{}/models/gemini-2.5-flash:generateContent", base);
    let text_body = serde_json::json!({
        "contents": [{"parts": [{"text": format!(
            "Generate exactly {} flashcard question-answer pairs about \"{}\" for children aged {}. \
             Difficulty: {}. Return ONLY a JSON array like: \
             [{{\"question\": \"...\", \"answer\": \"...\"}}]. No markdown, no explanation.",
            params.count, params.topic, params.age_group, params.difficulty
        )}]}],
        "generationConfig": {"responseMimeType": "application/json"}
    });

    let resp = client.post(&text_url).header("x-goog-api-key", api_key)
        .json(&text_body).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("Gemini text API error: {}", resp.text().await.unwrap_or_default()));
    }
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let text = json.pointer("/candidates/0/content/parts/0/text")
        .and_then(|t| t.as_str()).ok_or("No text in response")?;
    let cards: Vec<serde_json::Value> = serde_json::from_str(text).map_err(|e| e.to_string())?;

    // Step 2: Generate images for each card
    let img_url = format!("{}/models/gemini-2.5-flash-image:generateContent", base);
    for (i, card) in cards.iter().enumerate() {
        let question = card["question"].as_str().unwrap_or("?");
        let answer = card["answer"].as_str().unwrap_or("?");

        if params.include_images {
            let prompt = format!(
                "A colorful educational flashcard for children aged {}. \
                 Front side showing the question: \"{}\". \
                 Style: bright colors, large readable text, kid-friendly, rounded corners.",
                params.age_group, question
            );
            let body = serde_json::json!({
                "contents": [{"parts": [{"text": prompt}]}],
                "generationConfig": {"responseModalities": ["IMAGE", "TEXT"]}
            });
            let resp = client.post(&img_url).header("x-goog-api-key", api_key)
                .json(&body).send().await.map_err(|e| e.to_string())?;
            let img_json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            if let Some(data) = img_json.pointer("/candidates/0/content/parts")
                .and_then(|p| p.as_array())
                .and_then(|parts| parts.iter().find_map(|p| p.pointer("/inlineData/data").and_then(|d| d.as_str())))
            {
                let bytes = BASE64.decode(data).map_err(|e| e.to_string())?;
                tokio::fs::write(out_dir.join(format!("card_{:02}_front.png", i + 1)), &bytes)
                    .await.map_err(|e| e.to_string())?;
            }
        }

        // Generate back image (answer)
        let back_prompt = format!(
            "A colorful educational flashcard back for children. \
             Shows the answer in large clear text: \"{}\". \
             Style: bright green background, large white text, kid-friendly.",
            answer
        );
        let back_body = serde_json::json!({
            "contents": [{"parts": [{"text": back_prompt}]}],
            "generationConfig": {"responseModalities": ["IMAGE", "TEXT"]}
        });
        let back_resp = client.post(&img_url).header("x-goog-api-key", api_key)
            .json(&back_body).send().await.map_err(|e| e.to_string())?;
        let back_json: serde_json::Value = back_resp.json().await.map_err(|e| e.to_string())?;
        if let Some(data) = back_json.pointer("/candidates/0/content/parts")
            .and_then(|p| p.as_array())
            .and_then(|parts| parts.iter().find_map(|p| p.pointer("/inlineData/data").and_then(|d| d.as_str())))
        {
            let bytes = BASE64.decode(data).map_err(|e| e.to_string())?;
            tokio::fs::write(out_dir.join(format!("card_{:02}_back.png", i + 1)), &bytes)
                .await.map_err(|e| e.to_string())?;
        }

        info!(card = i + 1, "Generated flashcard");
    }

    // Save Q&A data as JSON
    tokio::fs::write(out_dir.join("cards.json"), serde_json::to_string_pretty(&cards).unwrap_or_default())
        .await.map_err(|e| e.to_string())?;

    let count = cards.len();
    info!(count, dir = %out_dir.display(), "Flashcards generated");
    Ok(format!("Generated {} flashcards in: {}", count, out_dir.display()))
}
