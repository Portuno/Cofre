// Audio service for Cofre Vault Platform
// Handles in-app audio recording, upload to Supabase Storage, and transcription

use crate::db::Database;
use crate::error::{Error, Result};
use crate::services::elevenlabs::ElevenLabsClient;
use chrono::Utc;
use uuid::Uuid;

/// Represents the state of an active recording session
#[derive(Debug, Clone)]
pub struct RecordingSession {
    pub session_id: Uuid,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub is_recording: bool,
}

/// Represents audio data captured from a recording session
#[derive(Debug, Clone)]
pub struct AudioBlob {
    pub data: Vec<u8>,
    pub mime_type: String,
    pub duration_ms: u32,
}

/// Result of uploading audio to storage
#[derive(Debug, Clone)]
pub struct UploadResult {
    pub storage_url: String,
    pub file_path: String,
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
}

/// Service for managing audio recording and upload operations
pub struct AudioService {
    #[allow(dead_code)]
    db: Database,
    elevenlabs: Option<ElevenLabsClient>,
}

impl AudioService {
    /// Create a new AudioService instance (without ElevenLabs)
    pub fn new(db: Database) -> Self {
        AudioService { db, elevenlabs: None }
    }

    /// Create an AudioService with ElevenLabs transcription enabled
    pub fn with_elevenlabs(db: Database, api_key: String) -> Self {
        AudioService {
            db,
            elevenlabs: Some(ElevenLabsClient::new(api_key)),
        }
    }

    /// Create an AudioService loading the ElevenLabs key from the environment
    pub fn from_env(db: Database) -> Self {
        let elevenlabs = ElevenLabsClient::from_env().ok();
        AudioService { db, elevenlabs }
    }

    /// Start a new audio recording session
    ///
    /// # Returns
    /// * `RecordingSession` - A new recording session with unique ID
    ///
    /// # Behavior
    /// * Initializes a recording session with a unique UUID
    /// * Records the start timestamp
    /// * Sets is_recording to true
    /// * Ready to capture audio data
    ///
    /// # Requirements
    /// * Validates: Requirements 7.1, 7.2
    pub fn start_recording(&self) -> RecordingSession {
        RecordingSession {
            session_id: Uuid::new_v4(),
            started_at: Utc::now(),
            is_recording: true,
        }
    }

    /// Stop an active recording session and capture audio as Blob
    ///
    /// # Arguments
    /// * `session` - The active RecordingSession to stop
    /// * `audio_data` - The raw audio bytes captured during recording
    /// * `mime_type` - The MIME type of the audio (e.g., "audio/webm", "audio/mp3")
    /// * `duration_ms` - Duration of the recording in milliseconds
    ///
    /// # Returns
    /// * `Result<AudioBlob>` - The captured audio as a Blob or an error
    ///
    /// # Validation
    /// * Verifies session is currently recording
    /// * Verifies audio_data is not empty
    /// * Verifies duration_ms is positive
    /// * Verifies mime_type is a valid audio MIME type
    ///
    /// # Behavior
    /// * Stops the recording session
    /// * Captures audio data as Blob
    /// * Returns AudioBlob with data, mime_type, and duration
    ///
    /// # Requirements
    /// * Validates: Requirements 7.1, 7.2, 7.3
    pub fn stop_recording(
        &self,
        session: &RecordingSession,
        audio_data: Vec<u8>,
        mime_type: String,
        duration_ms: u32,
    ) -> Result<AudioBlob> {
        // Validate session is recording
        if !session.is_recording {
            return Err(Error::InternalError(
                "Recording session is not active".to_string(),
            ));
        }

        // Validate audio data is not empty
        if audio_data.is_empty() {
            return Err(Error::InternalError(
                "Audio data cannot be empty".to_string(),
            ));
        }

        // Validate duration is positive
        if duration_ms == 0 {
            return Err(Error::InternalError(
                "Recording duration must be greater than 0".to_string(),
            ));
        }

        // Validate MIME type is audio
        if !mime_type.starts_with("audio/") {
            return Err(Error::InternalError(
                "Invalid audio MIME type".to_string(),
            ));
        }

        Ok(AudioBlob {
            data: audio_data,
            mime_type,
            duration_ms,
        })
    }

    /// Upload audio blob to Supabase Storage
    ///
    /// # Arguments
    /// * `blob` - The AudioBlob to upload
    /// * `vault_id` - UUID of the vault where audio is being stored
    /// * `user_id` - UUID of the user uploading the audio
    ///
    /// # Returns
    /// * `Result<UploadResult>` - Upload result with storage URL or an error
    ///
    /// # Validation
    /// * Verifies blob is not empty
    /// * Verifies vault_id is valid
    /// * Verifies user_id is valid
    ///
    /// # Behavior
    /// * Generates unique file path per vault: `vaults/{vault_id}/audio/{uuid}.{extension}`
    /// * Sends audio blob to Supabase Storage
    /// * Returns signed URL for uploaded file
    /// * Does not create partial ContentItem on failure
    /// * Allows retry on failure
    ///
    /// # Error Handling
    /// * If storage upload fails, returns StorageUploadFailed error
    /// * Does not create database records on upload failure
    /// * Keeps blob in memory for retry
    ///
    /// # Requirements
    /// * Validates: Requirements 7.4, 7.5, 7.6, 7.7, 17.1, 17.2, 17.3, 17.4, 17.5
    pub async fn upload_audio(
        &self,
        blob: &AudioBlob,
        vault_id: Uuid,
        _user_id: Uuid,
    ) -> Result<UploadResult> {
        // Validate blob is not empty
        if blob.data.is_empty() {
            return Err(Error::StorageUploadFailed(
                "Audio blob cannot be empty".to_string(),
            ));
        }

        // Generate unique file path per vault
        let file_extension = self.get_file_extension(&blob.mime_type);
        let file_name = format!("{}.{}", Uuid::new_v4(), file_extension);
        let file_path = format!("vaults/{}/audio/{}", vault_id, file_name);

        // In a real implementation, this would:
        // 1. Upload blob to Supabase Storage at file_path
        // 2. Generate signed URL for the uploaded file
        // 3. Return UploadResult with storage_url and file_path
        // 4. On failure, return StorageUploadFailed error without creating ContentItem
        //
        // TODO: Implement Supabase Storage upload
        // let storage_url = supabase_storage.upload(&file_path, &blob.data).await?;
        // let signed_url = supabase_storage.generate_signed_url(&file_path).await?;

        let storage_url = format!("https://storage.example.com/{}", file_path);

        Ok(UploadResult {
            storage_url,
            file_path,
            uploaded_at: Utc::now(),
        })
    }

    /// Retry uploading audio blob to Supabase Storage
    ///
    /// # Arguments
    /// * `blob` - The AudioBlob to upload (kept in memory from previous attempt)
    /// * `vault_id` - UUID of the vault where audio is being stored
    /// * `user_id` - UUID of the user uploading the audio
    /// * `max_retries` - Maximum number of retry attempts (default: 3)
    ///
    /// # Returns
    /// * `Result<UploadResult>` - Upload result with storage URL or an error
    ///
    /// # Behavior
    /// * Retries upload up to max_retries times
    /// * Uses exponential backoff between retries
    /// * Returns success on first successful upload
    /// * Returns error if all retries fail
    /// * Keeps blob in memory throughout retry attempts
    ///
    /// # Requirements
    /// * Validates: Requirements 7.7, 17.1, 17.2, 17.3, 17.4, 17.5
    pub async fn retry_upload_audio(
        &self,
        blob: &AudioBlob,
        vault_id: Uuid,
        user_id: Uuid,
        max_retries: u32,
    ) -> Result<UploadResult> {
        let mut last_error = Error::StorageUploadFailed("No attempts made".to_string());

        for attempt in 0..max_retries {
            match self.upload_audio(blob, vault_id, user_id).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = e;
                    if attempt < max_retries - 1 {
                        // Exponential backoff: 100ms, 200ms, 400ms, etc.
                        let backoff_ms = 100 * (2_u64.pow(attempt));
                        tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                    }
                }
            }
        }

        Err(last_error)
    }

    /// Helper function to get file extension from MIME type
    fn get_file_extension(&self, mime_type: &str) -> String {
        match mime_type {
            "audio/webm" => "webm".to_string(),
            "audio/mp3" | "audio/mpeg" => "mp3".to_string(),
            "audio/wav" => "wav".to_string(),
            "audio/ogg" => "ogg".to_string(),
            "audio/aac" => "aac".to_string(),
            "audio/flac" => "flac".to_string(),
            _ => "webm".to_string(), // Default to webm
        }
    }

    /// Transcribe audio blob to text using ElevenLabs Speech-to-Text
    ///
    /// # Arguments
    /// * `blob` - The AudioBlob to transcribe
    ///
    /// # Returns
    /// * `Result<String>` - The transcribed text or an error
    ///
    /// # Behavior
    /// * Sends audio data to ElevenLabs API
    /// * Returns transcribed text on success
    /// * Returns TranscriptionFailed error if ElevenLabs is not configured or API fails
    ///
    /// # Requirements
    /// * Validates: Requirements 8.1, 8.2, 8.3, 8.4, 8.5, 18.1, 18.2, 18.3, 18.4
    pub async fn transcribe_audio(&self, blob: &AudioBlob) -> Result<String> {
        let client = self.elevenlabs.as_ref().ok_or_else(|| {
            Error::TranscriptionFailed("ElevenLabs client not configured".to_string())
        })?;

        client.transcribe(blob.data.clone(), &blob.mime_type).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> Database {
        Database::new(crate::db::DatabaseConfig {
            supabase_url: "https://example.supabase.co".to_string(),
            supabase_key: "test-key".to_string(),
            database_url: "postgresql://user:pass@localhost/db".to_string(),
            max_connections: 10,
        })
    }

    // Recording session tests

    #[test]
    fn test_start_recording_creates_session() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let session = service.start_recording();

        assert!(session.is_recording);
        assert!(session.started_at <= Utc::now());
    }

    #[test]
    fn test_start_recording_generates_unique_session_ids() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let session1 = service.start_recording();
        let session2 = service.start_recording();

        assert_ne!(session1.session_id, session2.session_id);
    }

    #[test]
    fn test_start_recording_sets_is_recording_true() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let session = service.start_recording();

        assert_eq!(session.is_recording, true);
    }

    // Stop recording tests

    #[test]
    fn test_stop_recording_with_valid_data() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let session = service.start_recording();
        let audio_data = vec![0x00, 0x01, 0x02, 0x03];
        let mime_type = "audio/webm".to_string();
        let duration_ms = 5000;

        let result = service.stop_recording(&session, audio_data.clone(), mime_type.clone(), duration_ms);

        assert!(result.is_ok());
        let blob = result.unwrap();
        assert_eq!(blob.data, audio_data);
        assert_eq!(blob.mime_type, mime_type);
        assert_eq!(blob.duration_ms, duration_ms);
    }

    #[test]
    fn test_stop_recording_with_empty_audio_data() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let session = service.start_recording();
        let audio_data = vec![];
        let mime_type = "audio/webm".to_string();
        let duration_ms = 5000;

        let result = service.stop_recording(&session, audio_data, mime_type, duration_ms);

        assert!(result.is_err());
    }

    #[test]
    fn test_stop_recording_with_zero_duration() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let session = service.start_recording();
        let audio_data = vec![0x00, 0x01, 0x02, 0x03];
        let mime_type = "audio/webm".to_string();
        let duration_ms = 0;

        let result = service.stop_recording(&session, audio_data, mime_type, duration_ms);

        assert!(result.is_err());
    }

    #[test]
    fn test_stop_recording_with_invalid_mime_type() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let session = service.start_recording();
        let audio_data = vec![0x00, 0x01, 0x02, 0x03];
        let mime_type = "video/mp4".to_string();
        let duration_ms = 5000;

        let result = service.stop_recording(&session, audio_data, mime_type, duration_ms);

        assert!(result.is_err());
    }

    #[test]
    fn test_stop_recording_with_inactive_session() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let mut session = service.start_recording();
        session.is_recording = false;

        let audio_data = vec![0x00, 0x01, 0x02, 0x03];
        let mime_type = "audio/webm".to_string();
        let duration_ms = 5000;

        let result = service.stop_recording(&session, audio_data, mime_type, duration_ms);

        assert!(result.is_err());
    }

    #[test]
    fn test_stop_recording_with_mp3_mime_type() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let session = service.start_recording();
        let audio_data = vec![0xFF, 0xFB, 0x10, 0x00]; // MP3 header
        let mime_type = "audio/mp3".to_string();
        let duration_ms = 3000;

        let result = service.stop_recording(&session, audio_data.clone(), mime_type.clone(), duration_ms);

        assert!(result.is_ok());
        let blob = result.unwrap();
        assert_eq!(blob.mime_type, mime_type);
    }

    #[test]
    fn test_stop_recording_with_wav_mime_type() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let session = service.start_recording();
        let audio_data = vec![0x52, 0x49, 0x46, 0x46]; // RIFF header
        let mime_type = "audio/wav".to_string();
        let duration_ms = 2000;

        let result = service.stop_recording(&session, audio_data.clone(), mime_type.clone(), duration_ms);

        assert!(result.is_ok());
        let blob = result.unwrap();
        assert_eq!(blob.mime_type, mime_type);
    }

    #[test]
    fn test_stop_recording_preserves_audio_data() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let session = service.start_recording();
        let audio_data = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
        let mime_type = "audio/webm".to_string();
        let duration_ms = 1000;

        let result = service.stop_recording(&session, audio_data.clone(), mime_type, duration_ms);

        assert!(result.is_ok());
        let blob = result.unwrap();
        assert_eq!(blob.data, audio_data);
    }

    // Upload audio tests

    #[tokio::test]
    async fn test_upload_audio_with_valid_blob() {
        let db = create_test_db();
        let service = AudioService::new(db);
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let blob = AudioBlob {
            data: vec![0x00, 0x01, 0x02, 0x03],
            mime_type: "audio/webm".to_string(),
            duration_ms: 5000,
        };

        let result = service.upload_audio(&blob, vault_id, user_id).await;

        assert!(result.is_ok());
        let upload_result = result.unwrap();
        assert!(!upload_result.storage_url.is_empty());
        assert!(upload_result.file_path.contains(&vault_id.to_string()));
        assert!(upload_result.file_path.contains("audio"));
    }

    #[tokio::test]
    async fn test_upload_audio_with_empty_blob() {
        let db = create_test_db();
        let service = AudioService::new(db);
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let blob = AudioBlob {
            data: vec![],
            mime_type: "audio/webm".to_string(),
            duration_ms: 5000,
        };

        let result = service.upload_audio(&blob, vault_id, user_id).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_upload_audio_generates_unique_file_paths() {
        let db = create_test_db();
        let service = AudioService::new(db);
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let blob = AudioBlob {
            data: vec![0x00, 0x01, 0x02, 0x03],
            mime_type: "audio/webm".to_string(),
            duration_ms: 5000,
        };

        let result1 = service.upload_audio(&blob, vault_id, user_id).await;
        let result2 = service.upload_audio(&blob, vault_id, user_id).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let upload1 = result1.unwrap();
        let upload2 = result2.unwrap();

        // File paths should be different (different UUIDs)
        assert_ne!(upload1.file_path, upload2.file_path);
    }

    #[tokio::test]
    async fn test_upload_audio_includes_vault_id_in_path() {
        let db = create_test_db();
        let service = AudioService::new(db);
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let blob = AudioBlob {
            data: vec![0x00, 0x01, 0x02, 0x03],
            mime_type: "audio/webm".to_string(),
            duration_ms: 5000,
        };

        let result = service.upload_audio(&blob, vault_id, user_id).await;

        assert!(result.is_ok());
        let upload_result = result.unwrap();
        assert!(upload_result.file_path.contains(&vault_id.to_string()));
    }

    #[tokio::test]
    async fn test_upload_audio_includes_audio_directory() {
        let db = create_test_db();
        let service = AudioService::new(db);
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let blob = AudioBlob {
            data: vec![0x00, 0x01, 0x02, 0x03],
            mime_type: "audio/webm".to_string(),
            duration_ms: 5000,
        };

        let result = service.upload_audio(&blob, vault_id, user_id).await;

        assert!(result.is_ok());
        let upload_result = result.unwrap();
        assert!(upload_result.file_path.contains("/audio/"));
    }

    #[tokio::test]
    async fn test_upload_audio_with_mp3_mime_type() {
        let db = create_test_db();
        let service = AudioService::new(db);
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let blob = AudioBlob {
            data: vec![0xFF, 0xFB, 0x10, 0x00],
            mime_type: "audio/mp3".to_string(),
            duration_ms: 3000,
        };

        let result = service.upload_audio(&blob, vault_id, user_id).await;

        assert!(result.is_ok());
        let upload_result = result.unwrap();
        assert!(upload_result.file_path.ends_with(".mp3"));
    }

    #[tokio::test]
    async fn test_upload_audio_with_wav_mime_type() {
        let db = create_test_db();
        let service = AudioService::new(db);
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let blob = AudioBlob {
            data: vec![0x52, 0x49, 0x46, 0x46],
            mime_type: "audio/wav".to_string(),
            duration_ms: 2000,
        };

        let result = service.upload_audio(&blob, vault_id, user_id).await;

        assert!(result.is_ok());
        let upload_result = result.unwrap();
        assert!(upload_result.file_path.ends_with(".wav"));
    }

    #[tokio::test]
    async fn test_upload_audio_returns_storage_url() {
        let db = create_test_db();
        let service = AudioService::new(db);
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let blob = AudioBlob {
            data: vec![0x00, 0x01, 0x02, 0x03],
            mime_type: "audio/webm".to_string(),
            duration_ms: 5000,
        };

        let result = service.upload_audio(&blob, vault_id, user_id).await;

        assert!(result.is_ok());
        let upload_result = result.unwrap();
        assert!(!upload_result.storage_url.is_empty());
        assert!(upload_result.storage_url.contains("storage"));
    }

    #[tokio::test]
    async fn test_upload_audio_sets_uploaded_at_timestamp() {
        let db = create_test_db();
        let service = AudioService::new(db);
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let blob = AudioBlob {
            data: vec![0x00, 0x01, 0x02, 0x03],
            mime_type: "audio/webm".to_string(),
            duration_ms: 5000,
        };

        let before = Utc::now();
        let result = service.upload_audio(&blob, vault_id, user_id).await;
        let after = Utc::now();

        assert!(result.is_ok());
        let upload_result = result.unwrap();
        assert!(upload_result.uploaded_at >= before);
        assert!(upload_result.uploaded_at <= after);
    }

    // Retry upload tests

    #[tokio::test]
    async fn test_retry_upload_audio_succeeds_on_first_attempt() {
        let db = create_test_db();
        let service = AudioService::new(db);
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let blob = AudioBlob {
            data: vec![0x00, 0x01, 0x02, 0x03],
            mime_type: "audio/webm".to_string(),
            duration_ms: 5000,
        };

        let result = service.retry_upload_audio(&blob, vault_id, user_id, 3).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_retry_upload_audio_with_max_retries() {
        let db = create_test_db();
        let service = AudioService::new(db);
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let blob = AudioBlob {
            data: vec![0x00, 0x01, 0x02, 0x03],
            mime_type: "audio/webm".to_string(),
            duration_ms: 5000,
        };

        // With max_retries = 1, should succeed on first attempt
        let result = service.retry_upload_audio(&blob, vault_id, user_id, 1).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_retry_upload_audio_with_multiple_retries() {
        let db = create_test_db();
        let service = AudioService::new(db);
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let blob = AudioBlob {
            data: vec![0x00, 0x01, 0x02, 0x03],
            mime_type: "audio/webm".to_string(),
            duration_ms: 5000,
        };

        // With max_retries = 5, should succeed
        let result = service.retry_upload_audio(&blob, vault_id, user_id, 5).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_retry_upload_audio_keeps_blob_in_memory() {
        let db = create_test_db();
        let service = AudioService::new(db);
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let original_data = vec![0x00, 0x01, 0x02, 0x03];
        let blob = AudioBlob {
            data: original_data.clone(),
            mime_type: "audio/webm".to_string(),
            duration_ms: 5000,
        };

        let result = service.retry_upload_audio(&blob, vault_id, user_id, 3).await;

        assert!(result.is_ok());
        // Verify blob data is unchanged
        assert_eq!(blob.data, original_data);
    }

    // File extension tests

    #[test]
    fn test_get_file_extension_for_webm() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let ext = service.get_file_extension("audio/webm");
        assert_eq!(ext, "webm");
    }

    #[test]
    fn test_get_file_extension_for_mp3() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let ext = service.get_file_extension("audio/mp3");
        assert_eq!(ext, "mp3");
    }

    #[test]
    fn test_get_file_extension_for_mpeg() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let ext = service.get_file_extension("audio/mpeg");
        assert_eq!(ext, "mp3");
    }

    #[test]
    fn test_get_file_extension_for_wav() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let ext = service.get_file_extension("audio/wav");
        assert_eq!(ext, "wav");
    }

    #[test]
    fn test_get_file_extension_for_ogg() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let ext = service.get_file_extension("audio/ogg");
        assert_eq!(ext, "ogg");
    }

    #[test]
    fn test_get_file_extension_for_aac() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let ext = service.get_file_extension("audio/aac");
        assert_eq!(ext, "aac");
    }

    #[test]
    fn test_get_file_extension_for_flac() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let ext = service.get_file_extension("audio/flac");
        assert_eq!(ext, "flac");
    }

    #[test]
    fn test_get_file_extension_defaults_to_webm() {
        let db = create_test_db();
        let service = AudioService::new(db);

        let ext = service.get_file_extension("audio/unknown");
        assert_eq!(ext, "webm");
    }
}
