//! Quiz generation: multiple-choice questions with images and answer key.

use adk_rust_mcp_common::Config;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use schemars::JsonSchema;
use serde::Deserialize;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QuizParams {
    /// Quiz subject
    pub topic: String,
    /// Number of questions (1-20)
    #[serde(default = "default_questions")]
    pub questions: u8,
    /// Difficulty: easy, medium, hard
    #[serde(default = "default_difficulty")]
    pub difficulty: String,
    /// Target age range
    #[serde(default = "default_age")]
    pub age_group: String,
    /// Question type: multiple_choice, true_false, fill_blank
    #[serde(default = "default_type")]
    pub question_type: String,
    /// Generate images for questions
    #[serde(default = "default_true")]
    pub include_images: bool,
    /// Directory to save quiz
    #[serde(default)]
    pub output_dir: Option<String>,
}

fn default_questions() -> u8 { 5 }
fn default_difficulty() -> String { "easy".into() }
fn default_age() -> String { "8-10".into() }
fn default_type() -> String { "multiple_choice".into() }
fn default_true() -> bool { true }

pub async fn generate(config: &Config, params: QuizParams) -> Result<String, String> {
    let api_key = config.gemini_api_key.as_deref().ok_or("GEMINI_API_KEY required")?;
    let client = reqwest::Client::new();
    let base = config.gemini_base_url();
    let out_dir = PathBuf::from(params.output_dir.unwrap_or_else(|| "quiz".into()));
    tokio::fs::create_dir_all(&out_dir).await.map_err(|e| e.to_string())?;

    info!(topic = %params.topic, questions = params.questions, "Generating quiz");

    // Step 1: Generate questions with Gemini text
    let text_url = format!("{}/models/gemini-2.5-flash:generateContent", base);
    let format_hint = match params.question_type.as_str() {
        "true_false" => "options should be [\"True\", \"False\"]",
        "fill_blank" => "options should be 4 possible answers for the blank",
        _ => "options should be 4 choices labeled A-D",
    };
    let text_body = serde_json::json!({
        "contents": [{"parts": [{"text": format!(
            "Generate exactly {} {} questions about \"{}\" for children aged {}. Difficulty: {}. \
             {format_hint}. Return ONLY a JSON array: \
             [{{\"question\": \"...\", \"options\": [\"A\", \"B\", \"C\", \"D\"], \"correct\": \"B\", \"explanation\": \"...\"}}]. \
             No markdown.",
            params.questions, params.question_type, params.topic, params.age_group, params.difficulty,
            format_hint = format_hint
        )}]}],
        "generationConfig": {"responseMimeType": "application/json"}
    });

    let resp = client.post(&text_url).header("x-goog-api-key", api_key)
        .json(&text_body).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("Quiz gen error: {}", resp.text().await.unwrap_or_default()));
    }
    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let text = json.pointer("/candidates/0/content/parts/0/text")
        .and_then(|t| t.as_str()).ok_or("No quiz text")?;
    let questions: Vec<serde_json::Value> = serde_json::from_str(text).map_err(|e| e.to_string())?;

    // Step 2: Generate question images
    if params.include_images {
        let img_url = format!("{}/models/gemini-2.5-flash-image:generateContent", base);
        for (i, q) in questions.iter().enumerate() {
            let question = q["question"].as_str().unwrap_or("?");
            let options = q["options"].as_array()
                .map(|opts| opts.iter().filter_map(|o| o.as_str()).collect::<Vec<_>>().join(", "))
                .unwrap_or_default();

            let prompt = format!(
                "Educational quiz card for children aged {}. Question {}: \"{}\" Options: {}. \
                 Style: clean layout, large text, colorful option boxes, kid-friendly.",
                params.age_group, i + 1, question, options
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
                tokio::fs::write(out_dir.join(format!("question_{:02}.png", i + 1)), &bytes)
                    .await.map_err(|e| e.to_string())?;
            }
            info!(question = i + 1, "Generated quiz image");
        }
    }

    // Step 3: Write answer key
    let answer_key = serde_json::json!({
        "questions": questions.iter().enumerate().map(|(i, q)| {
            serde_json::json!({
                "number": i + 1,
                "question": q["question"],
                "options": q["options"],
                "correct": q["correct"],
                "explanation": q["explanation"]
            })
        }).collect::<Vec<_>>()
    });
    tokio::fs::write(
        out_dir.join("answer_key.json"),
        serde_json::to_string_pretty(&answer_key).unwrap_or_default()
    ).await.map_err(|e| e.to_string())?;

    let count = questions.len();
    info!(count, dir = %out_dir.display(), "Quiz generated");
    Ok(format!("Generated quiz with {} questions in: {}", count, out_dir.display()))
}
