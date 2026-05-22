//! diagram_from_code tool — render Mermaid/PlantUML source to SVG/PNG.

use adk_rust_mcp_common::Config;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::generate::render_mermaid;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DiagramFromCodeParams {
    /// Mermaid or PlantUML source code
    pub code: String,
    /// Source syntax: mermaid, plantuml
    #[serde(default = "default_syntax")]
    pub syntax: String,
    /// Output format: svg, png
    #[serde(default = "default_format")]
    pub format: String,
    /// Theme: default, dark, forest, neutral
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Output file path
    #[serde(default)]
    pub output_file: Option<String>,
}

fn default_syntax() -> String { "mermaid".into() }
fn default_format() -> String { "svg".into() }
fn default_theme() -> String { "default".into() }

pub async fn from_code(config: &Config, params: DiagramFromCodeParams) -> Result<String, String> {
    let rendered = if params.syntax == "mermaid" {
        render_mermaid(config, &params.code, &params.format, &params.theme).await?
    } else {
        // PlantUML: use Gemini to convert to SVG as fallback
        let prompt = format!(
            "Convert this PlantUML diagram to SVG. Output ONLY the SVG code, no markdown fences:\n\n{}",
            params.code
        );
        crate::generate::call_gemini_text(config, &prompt).await?
    };

    if let Some(ref path) = params.output_file {
        std::fs::write(path, &rendered).map_err(|e| e.to_string())?;
        Ok(format!("Diagram saved to {path}"))
    } else {
        Ok(rendered)
    }
}
