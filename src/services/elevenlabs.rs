// ElevenLabs client for Cofre Vault Platform
// Handles Speech-to-Text transcription via ElevenLabs API

use crate::error::{Error, Result};
use reqwest::multipart;

const ELEVENLABS_STT_URL: &str = "https://api.elevenlabs.io/v1/speech-to-text";

/// Response from ElevenLabs Speech-to-Text API
#[derive(Debug, serde::Deserialize)]
struct SttResponse {
    text: String,
}

/// Client for ElevenLabs API operations
pub struct ElevenLabsClient {
    api_key: String,
    http: reqwest::Client,
}

impl ElevenLabsClient {
    /// Create a new ElevenLabsClient with the given API key
    pub fn new(api_key: String) -> Self {
        ElevenLabsClient {
            api_key,
            http: reqwest::Client::new(),
        }
    }

    /// Create a client from the ELEVENLABS_API_KEY environment variable
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("ELEVENLABS_API_KEY")
            .map_err(|_| Error::InternalError("ELEVENLABS_API_KEY not set".to_string()))?;
        Ok(Self::new(api_key))
    }

    /// Transcribe audio bytes to text using ElevenLabs Speech-to-Text
    ///
    /// # Arguments
    /// * `audio_data` - Raw audio bytes
    /// * `mime_type`  - MIME type of the audio (e.g. "audio/webm", "audio/mp3")
    ///
    /// # Returns
    /// * `Result<String>` - The transcribed text or an error
    pub async fn transcribe(&self, audio_data: Vec<u8>, mime_type: &str) -> Result<String> {
        if audio_data.is_empty() {
            return Err(Error::TranscriptionFailed("Audio data is empty".to_string()));
        }

        let filename = format!("audio.{}", mime_to_ext(mime_type));

        let part = multipart::Part::bytes(audio_data)
            .file_name(filename)
            .mime_str(mime_type)
            .map_err(|e| Error::TranscriptionFailed(e.to_string()))?;

        let form = multipart::Form::new()
            .text("model_id", "scribe_v1")
            .part("file", part);

        let response = self
            .http
            .post(ELEVENLABS_STT_URL)
            .header("xi-api-key", &self.api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| Error::TranscriptionFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::TranscriptionFailed(format!(
                "ElevenLabs API error {}: {}",
                status, body
            )));
        }

        let stt: SttResponse = response
            .json()
            .await
            .map_err(|e| Error::TranscriptionFailed(e.to_string()))?;

        Ok(stt.text)
    }
}

fn mime_to_ext(mime_type: &str) -> &str {
    match mime_type {
        "audio/webm" => "webm",
        "audio/mp3" | "audio/mpeg" => "mp3",
        "audio/wav" => "wav",
        "audio/ogg" => "ogg",
        "audio/aac" => "aac",
        "audio/flac" => "flac",
        _ => "webm",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = ElevenLabsClient::new("test-key".to_string());
        assert_eq!(client.api_key, "test-key");
    }

    #[test]
    fn test_from_env_missing_key() {
        // Ensure the env var is not set for this test
        std::env::remove_var("ELEVENLABS_API_KEY");
        let result = ElevenLabsClient::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_from_env_with_key() {
        std::env::set_var("ELEVENLABS_API_KEY", "test-key-123");
        let result = ElevenLabsClient::from_env();
        assert!(result.is_ok());
        std::env::remove_var("ELEVENLABS_API_KEY");
    }

    #[tokio::test]
    async fn test_transcribe_empty_audio_returns_error() {
        let client = ElevenLabsClient::new("test-key".to_string());
        let result = client.transcribe(vec![], "audio/webm").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::TranscriptionFailed(msg) => assert!(msg.contains("empty")),
            _ => panic!("Expected TranscriptionFailed"),
        }
    }

    #[test]
    fn test_mime_to_ext() {
        assert_eq!(mime_to_ext("audio/webm"), "webm");
        assert_eq!(mime_to_ext("audio/mp3"), "mp3");
        assert_eq!(mime_to_ext("audio/mpeg"), "mp3");
        assert_eq!(mime_to_ext("audio/wav"), "wav");
        assert_eq!(mime_to_ext("audio/ogg"), "ogg");
        assert_eq!(mime_to_ext("audio/aac"), "aac");
        assert_eq!(mime_to_ext("audio/flac"), "flac");
        assert_eq!(mime_to_ext("audio/unknown"), "webm");
    }
}
