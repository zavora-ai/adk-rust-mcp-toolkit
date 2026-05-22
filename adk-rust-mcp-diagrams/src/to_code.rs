//! diagram_to_code tool — convert description to diagram source (no render).

use adk_rust_mcp_common::Config;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DiagramToCodeParams {
    /// Natural language description of the diagram
    pub description: String,
    /// Diagram type hint: flowchart, sequence, class, state, er, gantt, mindmap, pie, auto
    #[serde(default = "default_type")]
    pub r#type: String,
    /// Output syntax: mermaid, plantuml
    #[serde(default = "default_syntax")]
    pub syntax: String,
}

fn default_type() -> String { "auto".into() }
fn default_syntax() -> String { "mermaid".into() }

pub async fn to_code(config: &Config, params: DiagramToCodeParams) -> Result<String, String> {
    let syntax_name = if params.syntax == "plantuml" { "PlantUML" } else { "Mermaid" };
    let prompt = format!(
        "Convert this to {syntax_name} diagram code. Output ONLY the {syntax_name} code, no markdown fences. Type hint: {}. Description: {}",
        params.r#type, params.description
    );
    crate::generate::call_gemini_text(config, &prompt).await
}
