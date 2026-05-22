//! diagram_generate tool — natural language to rendered diagram.

use adk_rust_mcp_common::Config;
use schemars::JsonSchema;
use serde::Deserialize;
use std::io::Write;
use tokio::process::Command;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DiagramGenerateParams {
    /// Natural language description of the diagram
    pub description: String,
    /// Diagram type hint: flowchart, sequence, class, state, er, gantt, mindmap, pie, auto
    #[serde(default = "default_type")]
    pub r#type: String,
    /// Output format: svg, mermaid, plantuml, png
    #[serde(default = "default_format")]
    pub format: String,
    /// Theme: default, dark, forest, neutral
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Output file path
    #[serde(default)]
    pub output_file: Option<String>,
}

fn default_type() -> String { "auto".into() }
fn default_format() -> String { "svg".into() }
fn default_theme() -> String { "default".into() }

pub async fn generate(config: &Config, params: DiagramGenerateParams) -> Result<String, String> {
    // Step 1: Convert description to Mermaid code via Gemini
    let mermaid_code = call_gemini_for_mermaid(config, &params.description, &params.r#type).await?;

    // Step 2: If format is mermaid, return code directly
    if params.format == "mermaid" {
        if let Some(ref path) = params.output_file {
            std::fs::write(path, &mermaid_code).map_err(|e| e.to_string())?;
            return Ok(format!("Mermaid code saved to {path}"));
        }
        return Ok(mermaid_code);
    }

    // Step 3: Render to SVG/PNG
    let rendered = render_mermaid(config, &mermaid_code, &params.format, &params.theme).await?;

    if let Some(ref path) = params.output_file {
        std::fs::write(path, &rendered).map_err(|e| e.to_string())?;
        Ok(format!("Diagram saved to {path}"))
    } else {
        Ok(rendered)
    }
}

pub(crate) async fn call_gemini_for_mermaid(config: &Config, description: &str, diagram_type: &str) -> Result<String, String> {
    let prompt = format!(
        "Convert this to Mermaid diagram code. Output ONLY the mermaid code, no markdown fences. Type hint: {diagram_type}. Description: {description}"
    );
    call_gemini_text(config, &prompt).await
}

pub(crate) async fn call_gemini_text(config: &Config, prompt: &str) -> Result<String, String> {
    let api_key = config.gemini_api_key.as_deref()
        .ok_or_else(|| "GEMINI_API_KEY not set".to_string())?;

    let url = format!("{}/models/gemini-2.5-flash:generateContent", config.gemini_base_url());

    let body = serde_json::json!({
        "contents": [{"parts": [{"text": prompt}]}]
    });

    let resp = reqwest::Client::new()
        .post(&url)
        .header("x-goog-api-key", api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let text = json.pointer("/candidates/0/content/parts/0/text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("No text in Gemini response: {json}"))?;

    Ok(strip_markdown_fences(text))
}

fn strip_markdown_fences(text: &str) -> String {
    let trimmed = text.trim();
    // Strip ```mermaid ... ``` or ```plantuml ... ``` or ``` ... ```
    if let Some(rest) = trimmed.strip_prefix("```") {
        let rest = rest.strip_prefix("mermaid").or_else(|| rest.strip_prefix("plantuml")).unwrap_or(rest);
        let rest = rest.strip_prefix('\n').unwrap_or(rest);
        if let Some(code) = rest.strip_suffix("```") {
            return code.trim().to_string();
        }
    }
    trimmed.to_string()
}

pub(crate) async fn render_mermaid(config: &Config, code: &str, format: &str, theme: &str) -> Result<String, String> {
    // Try mmdc first
    if has_mmdc().await {
        render_with_mmdc(code, format, theme).await
    } else {
        // Fallback: ask Gemini to generate SVG directly
        let prompt = format!(
            "Generate a valid SVG image for this Mermaid diagram. Output ONLY the SVG code, no markdown fences:\n\n{code}"
        );
        call_gemini_text(config, &prompt).await
    }
}

async fn has_mmdc() -> bool {
    Command::new("which").arg("mmdc").output().await.map(|o| o.status.success()).unwrap_or(false)
}

async fn render_with_mmdc(code: &str, format: &str, theme: &str) -> Result<String, String> {
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let input_path = dir.path().join("input.mmd");
    let ext = if format == "png" { "png" } else { "svg" };
    let output_path = dir.path().join(format!("output.{ext}"));

    std::fs::File::create(&input_path)
        .and_then(|mut f| f.write_all(code.as_bytes()))
        .map_err(|e| e.to_string())?;

    let status = Command::new("mmdc")
        .arg("-i").arg(&input_path)
        .arg("-o").arg(&output_path)
        .arg("-t").arg(theme)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !status.status.success() {
        return Err(format!("mmdc failed: {}", String::from_utf8_lossy(&status.stderr)));
    }

    std::fs::read_to_string(&output_path).map_err(|e| e.to_string())
}
