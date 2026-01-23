//! Speech synthesis handler for the MCP Speech server.
//!
//! This module provides the `SpeechHandler` struct and parameter types for
//! text-to-speech synthesis using Google's Cloud TTS Chirp3-HD API.

use adk_rust_mcp_common::auth::AuthProvider;
use adk_rust_mcp_common::config::Config;
use adk_rust_mcp_common::error::Error;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info, instrument};

/// Default voice for speech synthesis.
pub const DEFAULT_VOICE: &str = "en-US-Chirp3-HD-Achernar";

/// Default language code.
pub const DEFAULT_LANGUAGE_CODE: &str = "en-US";

/// Default speaking rate.
pub const DEFAULT_SPEAKING_RATE: f32 = 1.0;

/// Minimum speaking rate.
pub const MIN_SPEAKING_RATE: f32 = 0.25;

/// Maximum speaking rate.
pub const MAX_SPEAKING_RATE: f32 = 4.0;

/// Default pitch.
pub const DEFAULT_PITCH: f32 = 0.0;

/// Minimum pitch (semitones).
pub const MIN_PITCH: f32 = -20.0;

/// Maximum pitch (semitones).
pub const MAX_PITCH: f32 = 20.0;

/// Valid pronunciation alphabets.
pub const VALID_ALPHABETS: &[&str] = &["ipa", "x-sampa"];


/// Custom pronunciation for a word.
///
/// Allows specifying phonetic pronunciation using IPA or X-SAMPA alphabets.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Pronunciation {
    /// The word to apply custom pronunciation to.
    pub word: String,

    /// The phonetic representation of the word.
    pub phonetic: String,

    /// The phonetic alphabet used: "ipa" or "x-sampa".
    pub alphabet: String,
}

impl Pronunciation {
    /// Validate the pronunciation entry.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.word.trim().is_empty() {
            return Err(ValidationError {
                field: "word".to_string(),
                message: "Word cannot be empty".to_string(),
            });
        }

        if self.phonetic.trim().is_empty() {
            return Err(ValidationError {
                field: "phonetic".to_string(),
                message: "Phonetic representation cannot be empty".to_string(),
            });
        }

        let alphabet_lower = self.alphabet.to_lowercase();
        if !VALID_ALPHABETS.contains(&alphabet_lower.as_str()) {
            return Err(ValidationError {
                field: "alphabet".to_string(),
                message: format!(
                    "Invalid alphabet '{}'. Must be one of: {}",
                    self.alphabet,
                    VALID_ALPHABETS.join(", ")
                ),
            });
        }

        Ok(())
    }

    /// Convert to SSML phoneme element.
    pub fn to_ssml(&self) -> String {
        let alphabet = self.alphabet.to_lowercase();
        format!(
            r#"<phoneme alphabet="{}" ph="{}">{}</phoneme>"#,
            alphabet, self.phonetic, self.word
        )
    }
}

/// Speech synthesis parameters.
///
/// These parameters control the text-to-speech synthesis via the Cloud TTS API.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct SpeechSynthesizeParams {
    /// Text to synthesize into speech.
    pub text: String,

    /// Voice name to use (Chirp3-HD voice).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,

    /// Language code (e.g., "en-US", "es-ES").
    #[serde(default = "default_language_code")]
    pub language_code: String,

    /// Speaking rate (0.25-4.0, default 1.0).
    #[serde(default = "default_speaking_rate")]
    pub speaking_rate: f32,

    /// Pitch adjustment in semitones (-20.0 to 20.0, default 0.0).
    #[serde(default)]
    pub pitch: f32,

    /// Custom pronunciations for specific words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pronunciations: Option<Vec<Pronunciation>>,

    /// Output file path for saving the WAV locally.
    /// If not specified, returns base64-encoded data.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_file: Option<String>,
}

fn default_language_code() -> String {
    DEFAULT_LANGUAGE_CODE.to_string()
}

fn default_speaking_rate() -> f32 {
    DEFAULT_SPEAKING_RATE
}


/// Validation error details for speech synthesis parameters.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// The field that failed validation.
    pub field: String,
    /// Description of the validation failure.
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

impl SpeechSynthesizeParams {
    /// Validate the parameters.
    ///
    /// # Returns
    /// - `Ok(())` if all parameters are valid
    /// - `Err(Vec<ValidationError>)` with all validation errors
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate text is not empty
        if self.text.trim().is_empty() {
            errors.push(ValidationError {
                field: "text".to_string(),
                message: "Text cannot be empty".to_string(),
            });
        }

        // Validate speaking_rate range
        if self.speaking_rate < MIN_SPEAKING_RATE || self.speaking_rate > MAX_SPEAKING_RATE {
            errors.push(ValidationError {
                field: "speaking_rate".to_string(),
                message: format!(
                    "speaking_rate must be between {} and {}, got {}",
                    MIN_SPEAKING_RATE, MAX_SPEAKING_RATE, self.speaking_rate
                ),
            });
        }

        // Validate pitch range
        if self.pitch < MIN_PITCH || self.pitch > MAX_PITCH {
            errors.push(ValidationError {
                field: "pitch".to_string(),
                message: format!(
                    "pitch must be between {} and {} semitones, got {}",
                    MIN_PITCH, MAX_PITCH, self.pitch
                ),
            });
        }

        // Validate pronunciations if provided
        if let Some(ref pronunciations) = self.pronunciations {
            for (i, pron) in pronunciations.iter().enumerate() {
                if let Err(e) = pron.validate() {
                    errors.push(ValidationError {
                        field: format!("pronunciations[{}].{}", i, e.field),
                        message: e.message,
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get the voice name to use, defaulting if not specified.
    pub fn get_voice(&self) -> &str {
        self.voice.as_deref().unwrap_or(DEFAULT_VOICE)
    }

    /// Build SSML text with pronunciations applied.
    pub fn build_ssml(&self) -> String {
        let mut text = self.text.clone();

        // Apply pronunciations if provided
        if let Some(ref pronunciations) = self.pronunciations {
            for pron in pronunciations {
                // Replace word with SSML phoneme
                text = text.replace(&pron.word, &pron.to_ssml());
            }
        }

        // Wrap in SSML speak element
        format!(r#"<speak>{}</speak>"#, text)
    }
}


/// Speech synthesis handler.
///
/// Handles text-to-speech requests using the Cloud TTS Chirp3-HD API.
pub struct SpeechHandler {
    /// Application configuration.
    pub config: Config,
    /// HTTP client for API requests.
    pub http: reqwest::Client,
    /// Authentication provider.
    pub auth: AuthProvider,
}

impl SpeechHandler {
    /// Create a new SpeechHandler with the given configuration.
    ///
    /// # Errors
    /// Returns an error if auth provider initialization fails.
    #[instrument(level = "debug", name = "speech_handler_new", skip_all)]
    pub async fn new(config: Config) -> Result<Self, Error> {
        debug!("Initializing SpeechHandler");

        let auth = AuthProvider::new().await?;
        let http = reqwest::Client::new();

        Ok(Self { config, http, auth })
    }

    /// Create a new SpeechHandler with provided dependencies (for testing).
    #[cfg(test)]
    pub fn with_deps(config: Config, http: reqwest::Client, auth: AuthProvider) -> Self {
        Self { config, http, auth }
    }

    /// Get the Cloud TTS API endpoint.
    pub fn get_endpoint(&self) -> String {
        format!(
            "https://texttospeech.googleapis.com/v1/text:synthesize"
        )
    }

    /// Get the Cloud TTS voices list endpoint.
    pub fn get_voices_endpoint(&self) -> String {
        format!("https://texttospeech.googleapis.com/v1/voices")
    }

    /// Synthesize speech from text.
    ///
    /// # Arguments
    /// * `params` - Speech synthesis parameters
    ///
    /// # Returns
    /// * `Ok(SpeechSynthesizeResult)` - Generated audio with data or path
    /// * `Err(Error)` - If validation fails, API call fails, or output handling fails
    #[instrument(level = "info", name = "synthesize_speech", skip(self, params))]
    pub async fn synthesize(&self, params: SpeechSynthesizeParams) -> Result<SpeechSynthesizeResult, Error> {
        // Validate parameters
        params.validate().map_err(|errors| {
            let messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
            Error::validation(messages.join("; "))
        })?;

        info!(voice = %params.get_voice(), "Synthesizing speech with Cloud TTS API");

        // Determine if we need SSML (for pronunciations)
        let (input, use_ssml) = if params.pronunciations.is_some() {
            (params.build_ssml(), true)
        } else {
            (params.text.clone(), false)
        };

        // Build the API request
        let request = TtsRequest {
            input: TtsInput {
                text: if use_ssml { None } else { Some(input.clone()) },
                ssml: if use_ssml { Some(input) } else { None },
            },
            voice: TtsVoice {
                language_code: params.language_code.clone(),
                name: params.get_voice().to_string(),
            },
            audio_config: TtsAudioConfig {
                audio_encoding: "LINEAR16".to_string(),
                speaking_rate: Some(params.speaking_rate),
                pitch: Some(params.pitch),
                sample_rate_hertz: Some(24000),
            },
        };

        // Get auth token
        let token = self
            .auth
            .get_token(&["https://www.googleapis.com/auth/cloud-platform"])
            .await?;

        // Make API request
        let endpoint = self.get_endpoint();
        debug!(endpoint = %endpoint, "Calling Cloud TTS API");

        let response = self
            .http
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::api(&endpoint, 0, format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::api(&endpoint, status.as_u16(), body));
        }

        // Parse response
        let api_response: TtsResponse = response.json().await.map_err(|e| {
            Error::api(
                &endpoint,
                status.as_u16(),
                format!("Failed to parse response: {}", e),
            )
        })?;

        let audio_data = api_response.audio_content;
        if audio_data.is_empty() {
            return Err(Error::api(&endpoint, 200, "No audio content returned from API"));
        }

        info!("Received audio data from Cloud TTS API");

        let audio = GeneratedAudio {
            data: audio_data,
            mime_type: "audio/wav".to_string(),
        };

        // Handle output based on params
        self.handle_output(audio, &params).await
    }


    /// List available voices.
    ///
    /// # Returns
    /// * `Ok(Vec<VoiceInfo>)` - List of available voices
    /// * `Err(Error)` - If API call fails
    #[instrument(level = "info", name = "list_voices", skip(self))]
    pub async fn list_voices(&self) -> Result<Vec<VoiceInfo>, Error> {
        info!("Listing available voices from Cloud TTS API");

        // Get auth token
        let token = self
            .auth
            .get_token(&["https://www.googleapis.com/auth/cloud-platform"])
            .await?;

        // Make API request
        let endpoint = self.get_voices_endpoint();
        debug!(endpoint = %endpoint, "Calling Cloud TTS voices API");

        let response = self
            .http
            .get(&endpoint)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| Error::api(&endpoint, 0, format!("Request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::api(&endpoint, status.as_u16(), body));
        }

        // Parse response
        let api_response: VoicesResponse = response.json().await.map_err(|e| {
            Error::api(
                &endpoint,
                status.as_u16(),
                format!("Failed to parse response: {}", e),
            )
        })?;

        // Filter for Chirp3-HD voices
        let chirp3_voices: Vec<VoiceInfo> = api_response
            .voices
            .into_iter()
            .filter(|v| v.name.contains("Chirp3-HD"))
            .map(|v| VoiceInfo {
                name: v.name,
                language_codes: v.language_codes,
                ssml_gender: v.ssml_gender,
                natural_sample_rate_hertz: v.natural_sample_rate_hertz,
            })
            .collect();

        info!(count = chirp3_voices.len(), "Found Chirp3-HD voices");
        Ok(chirp3_voices)
    }

    /// Handle output of generated audio based on params.
    async fn handle_output(
        &self,
        audio: GeneratedAudio,
        params: &SpeechSynthesizeParams,
    ) -> Result<SpeechSynthesizeResult, Error> {
        // If output_file is specified, save to local file
        if let Some(output_file) = &params.output_file {
            return self.save_to_file(audio, output_file).await;
        }

        // Otherwise, return base64-encoded data
        Ok(SpeechSynthesizeResult::Base64(audio))
    }

    /// Save audio to local file.
    async fn save_to_file(
        &self,
        audio: GeneratedAudio,
        output_file: &str,
    ) -> Result<SpeechSynthesizeResult, Error> {
        // Decode base64 data
        let data = BASE64.decode(&audio.data).map_err(|e| {
            Error::validation(format!("Invalid base64 data: {}", e))
        })?;

        // Ensure parent directory exists
        if let Some(parent) = Path::new(output_file).parent() {
            if !parent.as_os_str().is_empty() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }

        // Write to file
        tokio::fs::write(output_file, &data).await?;

        info!(path = %output_file, "Saved audio to local file");
        Ok(SpeechSynthesizeResult::LocalFile(output_file.to_string()))
    }
}


// =============================================================================
// API Request/Response Types
// =============================================================================

/// Cloud TTS API request.
#[derive(Debug, Serialize)]
pub struct TtsRequest {
    /// Input text or SSML
    pub input: TtsInput,
    /// Voice configuration
    pub voice: TtsVoice,
    /// Audio configuration
    #[serde(rename = "audioConfig")]
    pub audio_config: TtsAudioConfig,
}

/// TTS input (text or SSML).
#[derive(Debug, Serialize)]
pub struct TtsInput {
    /// Plain text input
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// SSML input
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssml: Option<String>,
}

/// TTS voice configuration.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsVoice {
    /// Language code (e.g., "en-US")
    pub language_code: String,
    /// Voice name
    pub name: String,
}

/// TTS audio configuration.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsAudioConfig {
    /// Audio encoding format
    pub audio_encoding: String,
    /// Speaking rate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speaking_rate: Option<f32>,
    /// Pitch adjustment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitch: Option<f32>,
    /// Sample rate in Hz
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_rate_hertz: Option<u32>,
}

/// Cloud TTS API response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsResponse {
    /// Base64-encoded audio content
    pub audio_content: String,
}

/// Cloud TTS voices list response.
#[derive(Debug, Deserialize)]
pub struct VoicesResponse {
    /// List of available voices
    pub voices: Vec<ApiVoiceInfo>,
}

/// Voice information from API.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiVoiceInfo {
    /// Voice name
    pub name: String,
    /// Supported language codes
    pub language_codes: Vec<String>,
    /// SSML gender
    pub ssml_gender: Option<String>,
    /// Natural sample rate
    pub natural_sample_rate_hertz: Option<u32>,
}

// =============================================================================
// Result Types
// =============================================================================

/// Generated audio data.
#[derive(Debug, Clone)]
pub struct GeneratedAudio {
    /// Base64-encoded audio data
    pub data: String,
    /// MIME type of the audio
    pub mime_type: String,
}

/// Voice information.
#[derive(Debug, Clone, Serialize)]
pub struct VoiceInfo {
    /// Voice name
    pub name: String,
    /// Supported language codes
    pub language_codes: Vec<String>,
    /// SSML gender
    pub ssml_gender: Option<String>,
    /// Natural sample rate
    pub natural_sample_rate_hertz: Option<u32>,
}

/// Result of speech synthesis.
#[derive(Debug)]
pub enum SpeechSynthesizeResult {
    /// Base64-encoded audio data (when no output specified)
    Base64(GeneratedAudio),
    /// Local file path (when output_file specified)
    LocalFile(String),
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let params: SpeechSynthesizeParams =
            serde_json::from_str(r#"{"text": "Hello world"}"#).unwrap();
        assert_eq!(params.language_code, DEFAULT_LANGUAGE_CODE);
        assert_eq!(params.speaking_rate, DEFAULT_SPEAKING_RATE);
        assert_eq!(params.pitch, DEFAULT_PITCH);
        assert!(params.voice.is_none());
        assert!(params.pronunciations.is_none());
        assert!(params.output_file.is_none());
    }

    #[test]
    fn test_valid_params() {
        let params = SpeechSynthesizeParams {
            text: "Hello world".to_string(),
            voice: Some("en-US-Chirp3-HD-Achernar".to_string()),
            language_code: "en-US".to_string(),
            speaking_rate: 1.5,
            pitch: 2.0,
            pronunciations: None,
            output_file: None,
        };

        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_empty_text() {
        let params = SpeechSynthesizeParams {
            text: "   ".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: 1.0,
            pitch: 0.0,
            pronunciations: None,
            output_file: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "text"));
    }

    #[test]
    fn test_speaking_rate_too_low() {
        let params = SpeechSynthesizeParams {
            text: "Hello".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: 0.1,
            pitch: 0.0,
            pronunciations: None,
            output_file: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "speaking_rate"));
    }

    #[test]
    fn test_speaking_rate_too_high() {
        let params = SpeechSynthesizeParams {
            text: "Hello".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: 5.0,
            pitch: 0.0,
            pronunciations: None,
            output_file: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "speaking_rate"));
    }

    #[test]
    fn test_pitch_too_low() {
        let params = SpeechSynthesizeParams {
            text: "Hello".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: 1.0,
            pitch: -25.0,
            pronunciations: None,
            output_file: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "pitch"));
    }

    #[test]
    fn test_pitch_too_high() {
        let params = SpeechSynthesizeParams {
            text: "Hello".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: 1.0,
            pitch: 25.0,
            pronunciations: None,
            output_file: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "pitch"));
    }

    #[test]
    fn test_valid_speaking_rate_boundaries() {
        // Test minimum valid speaking rate
        let params = SpeechSynthesizeParams {
            text: "Hello".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: MIN_SPEAKING_RATE,
            pitch: 0.0,
            pronunciations: None,
            output_file: None,
        };
        assert!(params.validate().is_ok());

        // Test maximum valid speaking rate
        let params = SpeechSynthesizeParams {
            text: "Hello".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: MAX_SPEAKING_RATE,
            pitch: 0.0,
            pronunciations: None,
            output_file: None,
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_valid_pitch_boundaries() {
        // Test minimum valid pitch
        let params = SpeechSynthesizeParams {
            text: "Hello".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: 1.0,
            pitch: MIN_PITCH,
            pronunciations: None,
            output_file: None,
        };
        assert!(params.validate().is_ok());

        // Test maximum valid pitch
        let params = SpeechSynthesizeParams {
            text: "Hello".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: 1.0,
            pitch: MAX_PITCH,
            pronunciations: None,
            output_file: None,
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_pronunciation_valid_ipa() {
        let pron = Pronunciation {
            word: "tomato".to_string(),
            phonetic: "təˈmeɪtoʊ".to_string(),
            alphabet: "ipa".to_string(),
        };
        assert!(pron.validate().is_ok());
    }

    #[test]
    fn test_pronunciation_valid_xsampa() {
        let pron = Pronunciation {
            word: "tomato".to_string(),
            phonetic: "t@\"meItoU".to_string(),
            alphabet: "x-sampa".to_string(),
        };
        assert!(pron.validate().is_ok());
    }

    #[test]
    fn test_pronunciation_invalid_alphabet() {
        let pron = Pronunciation {
            word: "tomato".to_string(),
            phonetic: "tomato".to_string(),
            alphabet: "invalid".to_string(),
        };
        let result = pron.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().field == "alphabet");
    }

    #[test]
    fn test_pronunciation_empty_word() {
        let pron = Pronunciation {
            word: "".to_string(),
            phonetic: "test".to_string(),
            alphabet: "ipa".to_string(),
        };
        let result = pron.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().field == "word");
    }

    #[test]
    fn test_pronunciation_empty_phonetic() {
        let pron = Pronunciation {
            word: "test".to_string(),
            phonetic: "".to_string(),
            alphabet: "ipa".to_string(),
        };
        let result = pron.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().field == "phonetic");
    }

    #[test]
    fn test_pronunciation_to_ssml() {
        let pron = Pronunciation {
            word: "tomato".to_string(),
            phonetic: "təˈmeɪtoʊ".to_string(),
            alphabet: "ipa".to_string(),
        };
        let ssml = pron.to_ssml();
        assert!(ssml.contains("phoneme"));
        assert!(ssml.contains("ipa"));
        assert!(ssml.contains("təˈmeɪtoʊ"));
        assert!(ssml.contains("tomato"));
    }

    #[test]
    fn test_build_ssml_with_pronunciations() {
        let params = SpeechSynthesizeParams {
            text: "I like tomato".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: 1.0,
            pitch: 0.0,
            pronunciations: Some(vec![Pronunciation {
                word: "tomato".to_string(),
                phonetic: "təˈmeɪtoʊ".to_string(),
                alphabet: "ipa".to_string(),
            }]),
            output_file: None,
        };

        let ssml = params.build_ssml();
        assert!(ssml.starts_with("<speak>"));
        assert!(ssml.ends_with("</speak>"));
        assert!(ssml.contains("phoneme"));
        assert!(!ssml.contains("tomato</speak>")); // tomato should be wrapped in phoneme
    }

    #[test]
    fn test_build_ssml_without_pronunciations() {
        let params = SpeechSynthesizeParams {
            text: "Hello world".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: 1.0,
            pitch: 0.0,
            pronunciations: None,
            output_file: None,
        };

        let ssml = params.build_ssml();
        assert_eq!(ssml, "<speak>Hello world</speak>");
    }

    #[test]
    fn test_get_voice_default() {
        let params = SpeechSynthesizeParams {
            text: "Hello".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: 1.0,
            pitch: 0.0,
            pronunciations: None,
            output_file: None,
        };

        assert_eq!(params.get_voice(), DEFAULT_VOICE);
    }

    #[test]
    fn test_get_voice_custom() {
        let params = SpeechSynthesizeParams {
            text: "Hello".to_string(),
            voice: Some("custom-voice".to_string()),
            language_code: "en-US".to_string(),
            speaking_rate: 1.0,
            pitch: 0.0,
            pronunciations: None,
            output_file: None,
        };

        assert_eq!(params.get_voice(), "custom-voice");
    }

    #[test]
    fn test_params_with_invalid_pronunciation() {
        let params = SpeechSynthesizeParams {
            text: "Hello".to_string(),
            voice: None,
            language_code: "en-US".to_string(),
            speaking_rate: 1.0,
            pitch: 0.0,
            pronunciations: Some(vec![Pronunciation {
                word: "test".to_string(),
                phonetic: "test".to_string(),
                alphabet: "invalid".to_string(),
            }]),
            output_file: None,
        };

        let result = params.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field.contains("pronunciations")));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let params = SpeechSynthesizeParams {
            text: "Hello world".to_string(),
            voice: Some("en-US-Chirp3-HD-Achernar".to_string()),
            language_code: "en-US".to_string(),
            speaking_rate: 1.5,
            pitch: 2.0,
            pronunciations: Some(vec![Pronunciation {
                word: "hello".to_string(),
                phonetic: "həˈloʊ".to_string(),
                alphabet: "ipa".to_string(),
            }]),
            output_file: Some("/tmp/output.wav".to_string()),
        };

        let json = serde_json::to_string(&params).unwrap();
        let deserialized: SpeechSynthesizeParams = serde_json::from_str(&json).unwrap();

        assert_eq!(params.text, deserialized.text);
        assert_eq!(params.voice, deserialized.voice);
        assert_eq!(params.language_code, deserialized.language_code);
        assert_eq!(params.speaking_rate, deserialized.speaking_rate);
        assert_eq!(params.pitch, deserialized.pitch);
        assert_eq!(params.output_file, deserialized.output_file);
    }
}


#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: rust-mcp-genmedia, Property 8: Numeric Parameter Range Validation (speaking_rate, pitch)
    // **Validates: Requirements 7.6, 7.7**
    //
    // For any numeric parameter with defined bounds (speaking_rate 0.25-4.0, pitch -20.0 to 20.0),
    // values outside the valid range SHALL be rejected with a validation error.

    /// Strategy to generate valid speaking_rate values (0.25-4.0)
    fn valid_speaking_rate_strategy() -> impl Strategy<Value = f32> {
        (MIN_SPEAKING_RATE..=MAX_SPEAKING_RATE).prop_map(|x| (x * 100.0).round() / 100.0)
    }

    /// Strategy to generate invalid speaking_rate values (< 0.25 or > 4.0)
    fn invalid_speaking_rate_strategy() -> impl Strategy<Value = f32> {
        prop_oneof![
            // Values below minimum (exclusive of MIN_SPEAKING_RATE)
            (0.0f32..0.24f32).prop_map(|x| (x * 100.0).round() / 100.0),
            // Values above maximum (exclusive of MAX_SPEAKING_RATE)
            (4.01f32..10.0f32).prop_map(|x| (x * 100.0).round() / 100.0),
        ]
    }

    /// Strategy to generate valid pitch values (-20.0 to 20.0)
    fn valid_pitch_strategy() -> impl Strategy<Value = f32> {
        (MIN_PITCH..=MAX_PITCH).prop_map(|x| (x * 10.0).round() / 10.0)
    }

    /// Strategy to generate invalid pitch values (< -20.0 or > 20.0)
    fn invalid_pitch_strategy() -> impl Strategy<Value = f32> {
        prop_oneof![
            (-50.0f32..MIN_PITCH).prop_map(|x| (x * 10.0).round() / 10.0),
            (MAX_PITCH + 0.1..50.0f32).prop_map(|x| (x * 10.0).round() / 10.0),
        ]
    }

    /// Strategy to generate valid text (non-empty)
    fn valid_text_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 ]{1,100}"
            .prop_map(|s| s.trim().to_string())
            .prop_filter("Must not be empty", |s| !s.trim().is_empty())
    }

    proptest! {
        /// Property 8: Valid speaking_rate values (0.25-4.0) should pass validation
        #[test]
        fn valid_speaking_rate_passes_validation(
            rate in valid_speaking_rate_strategy(),
            text in valid_text_strategy(),
        ) {
            let params = SpeechSynthesizeParams {
                text,
                voice: None,
                language_code: "en-US".to_string(),
                speaking_rate: rate,
                pitch: 0.0,
                pronunciations: None,
                output_file: None,
            };

            let result = params.validate();
            prop_assert!(
                result.is_ok(),
                "speaking_rate {} should be valid, but got errors: {:?}",
                rate,
                result.err()
            );
        }

        /// Property 8: Invalid speaking_rate values (< 0.25 or > 4.0) should fail validation
        #[test]
        fn invalid_speaking_rate_fails_validation(
            rate in invalid_speaking_rate_strategy(),
            text in valid_text_strategy(),
        ) {
            let params = SpeechSynthesizeParams {
                text,
                voice: None,
                language_code: "en-US".to_string(),
                speaking_rate: rate,
                pitch: 0.0,
                pronunciations: None,
                output_file: None,
            };

            let result = params.validate();
            prop_assert!(
                result.is_err(),
                "speaking_rate {} should be invalid",
                rate
            );

            let errors = result.unwrap_err();
            prop_assert!(
                errors.iter().any(|e| e.field == "speaking_rate"),
                "Should have a speaking_rate validation error for value {}",
                rate
            );
        }

        /// Property 8: Valid pitch values (-20.0 to 20.0) should pass validation
        #[test]
        fn valid_pitch_passes_validation(
            pitch in valid_pitch_strategy(),
            text in valid_text_strategy(),
        ) {
            let params = SpeechSynthesizeParams {
                text,
                voice: None,
                language_code: "en-US".to_string(),
                speaking_rate: 1.0,
                pitch,
                pronunciations: None,
                output_file: None,
            };

            let result = params.validate();
            prop_assert!(
                result.is_ok(),
                "pitch {} should be valid, but got errors: {:?}",
                pitch,
                result.err()
            );
        }

        /// Property 8: Invalid pitch values (< -20.0 or > 20.0) should fail validation
        #[test]
        fn invalid_pitch_fails_validation(
            pitch in invalid_pitch_strategy(),
            text in valid_text_strategy(),
        ) {
            let params = SpeechSynthesizeParams {
                text,
                voice: None,
                language_code: "en-US".to_string(),
                speaking_rate: 1.0,
                pitch,
                pronunciations: None,
                output_file: None,
            };

            let result = params.validate();
            prop_assert!(
                result.is_err(),
                "pitch {} should be invalid",
                pitch
            );

            let errors = result.unwrap_err();
            prop_assert!(
                errors.iter().any(|e| e.field == "pitch"),
                "Should have a pitch validation error for value {}",
                pitch
            );
        }

        /// Property: Combined valid speaking_rate and pitch should pass validation
        #[test]
        fn valid_speaking_rate_and_pitch_passes_validation(
            rate in valid_speaking_rate_strategy(),
            pitch in valid_pitch_strategy(),
            text in valid_text_strategy(),
        ) {
            let params = SpeechSynthesizeParams {
                text,
                voice: None,
                language_code: "en-US".to_string(),
                speaking_rate: rate,
                pitch,
                pronunciations: None,
                output_file: None,
            };

            let result = params.validate();
            prop_assert!(
                result.is_ok(),
                "speaking_rate {} and pitch {} should be valid, but got errors: {:?}",
                rate,
                pitch,
                result.err()
            );
        }
    }

    // Feature: rust-mcp-genmedia, Property 12: Pronunciation Alphabet Validation
    // **Validates: Requirements 7.9**
    //
    // For any pronunciation entry in speech_synthesize, the alphabet field SHALL be
    // either "ipa" or "x-sampa". Other values SHALL be rejected with a validation error.

    /// Strategy to generate valid alphabet values
    fn valid_alphabet_strategy() -> impl Strategy<Value = String> {
        prop_oneof![Just("ipa".to_string()), Just("x-sampa".to_string()),]
    }

    /// Strategy to generate invalid alphabet values
    fn invalid_alphabet_strategy() -> impl Strategy<Value = String> {
        "[a-z]{1,10}"
            .prop_filter("Must not be valid alphabet", |s| {
                let lower = s.to_lowercase();
                lower != "ipa" && lower != "x-sampa"
            })
    }

    /// Strategy to generate valid word (non-empty)
    fn valid_word_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z]{1,20}".prop_filter("Must not be empty", |s| !s.trim().is_empty())
    }

    /// Strategy to generate valid phonetic (non-empty)
    fn valid_phonetic_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Zəˈɪʊæɑɔɛʌ]{1,30}".prop_filter("Must not be empty", |s| !s.trim().is_empty())
    }

    proptest! {
        /// Property 12: Valid alphabet values ("ipa", "x-sampa") should pass validation
        #[test]
        fn valid_alphabet_passes_validation(
            alphabet in valid_alphabet_strategy(),
            word in valid_word_strategy(),
            phonetic in valid_phonetic_strategy(),
        ) {
            let pron = Pronunciation {
                word,
                phonetic,
                alphabet: alphabet.clone(),
            };

            let result = pron.validate();
            prop_assert!(
                result.is_ok(),
                "alphabet '{}' should be valid, but got error: {:?}",
                alphabet,
                result.err()
            );
        }

        /// Property 12: Invalid alphabet values should fail validation
        #[test]
        fn invalid_alphabet_fails_validation(
            alphabet in invalid_alphabet_strategy(),
            word in valid_word_strategy(),
            phonetic in valid_phonetic_strategy(),
        ) {
            let pron = Pronunciation {
                word,
                phonetic,
                alphabet: alphabet.clone(),
            };

            let result = pron.validate();
            prop_assert!(
                result.is_err(),
                "alphabet '{}' should be invalid",
                alphabet
            );

            let error = result.unwrap_err();
            prop_assert!(
                error.field == "alphabet",
                "Should have an alphabet validation error for value '{}'",
                alphabet
            );
        }

        /// Property 12: Pronunciation with valid alphabet in params should pass validation
        #[test]
        fn params_with_valid_pronunciation_passes_validation(
            alphabet in valid_alphabet_strategy(),
            word in valid_word_strategy(),
            phonetic in valid_phonetic_strategy(),
            text in valid_text_strategy(),
        ) {
            let params = SpeechSynthesizeParams {
                text,
                voice: None,
                language_code: "en-US".to_string(),
                speaking_rate: 1.0,
                pitch: 0.0,
                pronunciations: Some(vec![Pronunciation {
                    word,
                    phonetic,
                    alphabet: alphabet.clone(),
                }]),
                output_file: None,
            };

            let result = params.validate();
            prop_assert!(
                result.is_ok(),
                "params with alphabet '{}' should be valid, but got errors: {:?}",
                alphabet,
                result.err()
            );
        }

        /// Property 12: Pronunciation with invalid alphabet in params should fail validation
        #[test]
        fn params_with_invalid_pronunciation_fails_validation(
            alphabet in invalid_alphabet_strategy(),
            word in valid_word_strategy(),
            phonetic in valid_phonetic_strategy(),
            text in valid_text_strategy(),
        ) {
            let params = SpeechSynthesizeParams {
                text,
                voice: None,
                language_code: "en-US".to_string(),
                speaking_rate: 1.0,
                pitch: 0.0,
                pronunciations: Some(vec![Pronunciation {
                    word,
                    phonetic,
                    alphabet: alphabet.clone(),
                }]),
                output_file: None,
            };

            let result = params.validate();
            prop_assert!(
                result.is_err(),
                "params with alphabet '{}' should be invalid",
                alphabet
            );

            let errors = result.unwrap_err();
            prop_assert!(
                errors.iter().any(|e| e.field.contains("pronunciations") && e.field.contains("alphabet")),
                "Should have a pronunciations.alphabet validation error for value '{}'",
                alphabet
            );
        }

        /// Property: Empty text should always fail validation regardless of other params
        #[test]
        fn empty_text_fails_validation(
            rate in valid_speaking_rate_strategy(),
            pitch in valid_pitch_strategy(),
        ) {
            let params = SpeechSynthesizeParams {
                text: "   ".to_string(),
                voice: None,
                language_code: "en-US".to_string(),
                speaking_rate: rate,
                pitch,
                pronunciations: None,
                output_file: None,
            };

            let result = params.validate();
            prop_assert!(result.is_err());

            let errors = result.unwrap_err();
            prop_assert!(
                errors.iter().any(|e| e.field == "text"),
                "Should have a text validation error"
            );
        }
    }
}
