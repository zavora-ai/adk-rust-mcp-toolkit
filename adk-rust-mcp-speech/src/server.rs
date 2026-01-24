//! MCP Server implementation for the Speech server.
//!
//! This module provides the MCP server handler that exposes:
//! - `speech_synthesize` tool for text-to-speech synthesis
//! - `speech_list_voices` tool for listing available voices

use crate::handler::{
    Pronunciation, SpeechHandler, SpeechSynthesizeParams, SpeechSynthesizeResult,
};
use adk_rust_mcp_common::config::Config;
use adk_rust_mcp_common::error::Error;
use rmcp::{
    model::{
        CallToolResult, Content, ListResourcesResult, ReadResourceResult, ServerCapabilities,
        ServerInfo,
    },
    ErrorData as McpError, ServerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;
use std::borrow::Cow;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// MCP Server for speech synthesis.
#[derive(Clone)]
pub struct SpeechServer {
    /// Handler for speech synthesis operations
    handler: Arc<RwLock<Option<SpeechHandler>>>,
    /// Server configuration
    config: Config,
}

/// Tool parameters wrapper for speech_synthesize.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SpeechSynthesizeToolParams {
    /// Text to synthesize into speech
    pub text: String,
    /// Voice name to use (Chirp3-HD voice)
    #[serde(default)]
    pub voice: Option<String>,
    /// Language code (e.g., "en-US")
    #[serde(default)]
    pub language_code: Option<String>,
    /// Speaking rate (0.25-4.0, default 1.0)
    #[serde(default)]
    pub speaking_rate: Option<f32>,
    /// Pitch adjustment in semitones (-20.0 to 20.0, default 0.0)
    #[serde(default)]
    pub pitch: Option<f32>,
    /// Custom pronunciations for specific words
    #[serde(default)]
    pub pronunciations: Option<Vec<PronunciationToolParam>>,
    /// Output file path for saving locally
    #[serde(default)]
    pub output_file: Option<String>,
}

/// Pronunciation parameter for tool input.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct PronunciationToolParam {
    /// The word to apply custom pronunciation to
    pub word: String,
    /// The phonetic representation of the word
    pub phonetic: String,
    /// The phonetic alphabet: "ipa" or "x-sampa"
    pub alphabet: String,
}

impl From<PronunciationToolParam> for Pronunciation {
    fn from(p: PronunciationToolParam) -> Self {
        Self {
            word: p.word,
            phonetic: p.phonetic,
            alphabet: p.alphabet,
        }
    }
}

impl From<SpeechSynthesizeToolParams> for SpeechSynthesizeParams {
    fn from(params: SpeechSynthesizeToolParams) -> Self {
        Self {
            text: params.text,
            voice: params.voice,
            language_code: params
                .language_code
                .unwrap_or_else(|| "en-US".to_string()),
            speaking_rate: params.speaking_rate.unwrap_or(1.0),
            pitch: params.pitch.unwrap_or(0.0),
            pronunciations: params
                .pronunciations
                .map(|p| p.into_iter().map(Into::into).collect()),
            output_file: params.output_file,
        }
    }
}


impl SpeechServer {
    /// Create a new SpeechServer with the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            handler: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// Initialize the handler (called lazily on first use).
    async fn ensure_handler(&self) -> Result<(), Error> {
        let mut handler = self.handler.write().await;
        if handler.is_none() {
            *handler = Some(SpeechHandler::new(self.config.clone()).await?);
        }
        Ok(())
    }

    /// Synthesize speech from text.
    pub async fn synthesize(
        &self,
        params: SpeechSynthesizeToolParams,
    ) -> Result<CallToolResult, McpError> {
        info!(text_len = params.text.len(), "Synthesizing speech");

        // Ensure handler is initialized
        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard
            .as_ref()
            .ok_or_else(|| McpError::internal_error("Handler not initialized", None))?;

        let synth_params: SpeechSynthesizeParams = params.into();
        let result = handler.synthesize(synth_params).await.map_err(|e| {
            McpError::internal_error(format!("Speech synthesis failed: {}", e), None)
        })?;

        // Convert result to MCP content
        let content = match result {
            SpeechSynthesizeResult::Base64(audio) => {
                vec![Content::text(format!(
                    "data:{};base64,{}",
                    audio.mime_type, audio.data
                ))]
            }
            SpeechSynthesizeResult::LocalFile(path) => {
                vec![Content::text(format!("Audio saved to: {}", path))]
            }
        };

        Ok(CallToolResult::success(content))
    }

    /// List available voices.
    pub async fn list_voices(&self) -> Result<CallToolResult, McpError> {
        info!("Listing available voices");

        // Ensure handler is initialized
        self.ensure_handler().await.map_err(|e| {
            McpError::internal_error(format!("Failed to initialize handler: {}", e), None)
        })?;

        let handler_guard = self.handler.read().await;
        let handler = handler_guard
            .as_ref()
            .ok_or_else(|| McpError::internal_error("Handler not initialized", None))?;

        let voices = handler.list_voices().await.map_err(|e| {
            McpError::internal_error(format!("Failed to list voices: {}", e), None)
        })?;

        // Format voices as JSON
        let voices_json = serde_json::to_string_pretty(&voices).map_err(|e| {
            McpError::internal_error(format!("Failed to serialize voices: {}", e), None)
        })?;

        Ok(CallToolResult::success(vec![Content::text(voices_json)]))
    }
}


impl ServerHandler for SpeechServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Text-to-speech server using Google Cloud TTS Chirp3-HD API. \
                 Use the speech_synthesize tool to convert text to speech, \
                 and speech_list_voices to see available voices."
                    .to_string(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }

    fn list_tools(
        &self,
        _params: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::ListToolsResult, McpError>> + Send + '_
    {
        async move {
            use rmcp::model::{ListToolsResult, Tool};
            use schemars::schema_for;

            // speech_synthesize tool
            let synth_schema = schema_for!(SpeechSynthesizeToolParams);
            let synth_schema_value = serde_json::to_value(&synth_schema).unwrap_or_default();
            let synth_input_schema = match synth_schema_value {
                serde_json::Value::Object(map) => Arc::new(map),
                _ => Arc::new(serde_json::Map::new()),
            };

            // speech_list_voices tool (no parameters)
            let empty_schema = Arc::new(serde_json::Map::new());

            Ok(ListToolsResult {
                tools: vec![
                    Tool {
                        name: Cow::Borrowed("speech_synthesize"),
                        description: Some(Cow::Borrowed(
                            "Convert text to speech using Google Cloud TTS Chirp3-HD voices. \
                             Returns base64-encoded WAV audio or saves to a local file. \
                             Supports custom pronunciations using IPA or X-SAMPA phonetic alphabets.",
                        )),
                        input_schema: synth_input_schema,
                        annotations: None,
                        icons: None,
                        meta: None,
                        output_schema: None,
                        title: None,
                    },
                    Tool {
                        name: Cow::Borrowed("speech_list_voices"),
                        description: Some(Cow::Borrowed(
                            "List available Chirp3-HD voices with their supported languages.",
                        )),
                        input_schema: empty_schema,
                        annotations: None,
                        icons: None,
                        meta: None,
                        output_schema: None,
                        title: None,
                    },
                ],
                next_cursor: None,
                meta: None,
            })
        }
    }

    fn call_tool(
        &self,
        params: rmcp::model::CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        async move {
            match params.name.as_ref() {
                "speech_synthesize" => {
                    let tool_params: SpeechSynthesizeToolParams = params
                        .arguments
                        .map(|args| serde_json::from_value(serde_json::Value::Object(args)))
                        .transpose()
                        .map_err(|e| {
                            McpError::invalid_params(format!("Invalid parameters: {}", e), None)
                        })?
                        .ok_or_else(|| McpError::invalid_params("Missing parameters", None))?;

                    self.synthesize(tool_params).await
                }
                "speech_list_voices" => self.list_voices().await,
                _ => Err(McpError::invalid_params(
                    format!("Unknown tool: {}", params.name),
                    None,
                )),
            }
        }
    }

    fn list_resources(
        &self,
        _params: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListResourcesResult, McpError>> + Send + '_ {
        async move {
            debug!("Listing resources (none available for speech server)");

            Ok(ListResourcesResult {
                resources: vec![],
                next_cursor: None,
                meta: None,
            })
        }
    }

    fn read_resource(
        &self,
        params: rmcp::model::ReadResourceRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ReadResourceResult, McpError>> + Send + '_ {
        async move {
            let uri = &params.uri;
            debug!(uri = %uri, "Reading resource");

            Err(McpError::resource_not_found(
                format!("Unknown resource: {}", uri),
                None,
            ))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config {
            project_id: "test-project".to_string(),
            location: "us-central1".to_string(),
            gcs_bucket: None,
            port: 8080,
        }
    }

    #[test]
    fn test_server_info() {
        let server = SpeechServer::new(test_config());
        let info = server.get_info();
        assert!(info.instructions.is_some());
    }

    #[test]
    fn test_tool_params_conversion() {
        let tool_params = SpeechSynthesizeToolParams {
            text: "Hello world".to_string(),
            voice: Some("en-US-Chirp3-HD-Achernar".to_string()),
            language_code: Some("en-US".to_string()),
            speaking_rate: Some(1.5),
            pitch: Some(2.0),
            pronunciations: Some(vec![PronunciationToolParam {
                word: "hello".to_string(),
                phonetic: "həˈloʊ".to_string(),
                alphabet: "ipa".to_string(),
            }]),
            output_file: None,
        };

        let synth_params: SpeechSynthesizeParams = tool_params.into();
        assert_eq!(synth_params.text, "Hello world");
        assert_eq!(synth_params.voice, Some("en-US-Chirp3-HD-Achernar".to_string()));
        assert_eq!(synth_params.language_code, "en-US");
        assert_eq!(synth_params.speaking_rate, 1.5);
        assert_eq!(synth_params.pitch, 2.0);
        assert!(synth_params.pronunciations.is_some());
    }

    #[test]
    fn test_tool_params_defaults() {
        let tool_params = SpeechSynthesizeToolParams {
            text: "Hello".to_string(),
            voice: None,
            language_code: None,
            speaking_rate: None,
            pitch: None,
            pronunciations: None,
            output_file: None,
        };

        let synth_params: SpeechSynthesizeParams = tool_params.into();
        assert_eq!(synth_params.language_code, "en-US");
        assert_eq!(synth_params.speaking_rate, 1.0);
        assert_eq!(synth_params.pitch, 0.0);
    }

    #[test]
    fn test_pronunciation_conversion() {
        let tool_pron = PronunciationToolParam {
            word: "tomato".to_string(),
            phonetic: "təˈmeɪtoʊ".to_string(),
            alphabet: "ipa".to_string(),
        };

        let pron: Pronunciation = tool_pron.into();
        assert_eq!(pron.word, "tomato");
        assert_eq!(pron.phonetic, "təˈmeɪtoʊ");
        assert_eq!(pron.alphabet, "ipa");
    }
}
