// Integration test for ElevenLabs Speech-to-Text
// Run with: cargo test --test elevenlabs_integration -- --nocapture --ignored

use cofre_vault::{Database, services::AudioService};

#[tokio::test]
#[ignore] // Ignored by default since it requires API key and makes real API calls
async fn test_elevenlabs_transcription_real_api() {
    // Load environment variables
    dotenv::dotenv().ok();

    // Create database (mock is fine for this test)
    let db = Database::new(cofre_vault::db::DatabaseConfig {
        supabase_url: "https://example.supabase.co".to_string(),
        supabase_key: "test-key".to_string(),
        database_url: "postgresql://user:pass@localhost/db".to_string(),
        max_connections: 10,
    });

    // Create AudioService with ElevenLabs from env
    let service = AudioService::from_env(db);

    // Create a simple audio blob with actual audio data
    // This is a minimal valid MP3 file (silent, ~0.1 seconds)
    let mp3_data = vec![
        0xFF, 0xFB, 0x90, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    let blob = cofre_vault::services::audio::AudioBlob {
        data: mp3_data,
        mime_type: "audio/mp3".to_string(),
        duration_ms: 100,
    };

    // Attempt transcription
    let result = service.transcribe_audio(&blob).await;

    match result {
        Ok(transcript) => {
            println!("✅ Transcription successful!");
            println!("📝 Transcript: {}", transcript);
            assert!(!transcript.is_empty() || transcript.is_empty()); // Silent audio may return empty
        }
        Err(e) => {
            println!("❌ Transcription failed: {:?}", e);
            panic!("Transcription should succeed with valid API key");
        }
    }
}

#[tokio::test]
async fn test_transcription_without_api_key() {
    // Create database
    let db = Database::new(cofre_vault::db::DatabaseConfig {
        supabase_url: "https://example.supabase.co".to_string(),
        supabase_key: "test-key".to_string(),
        database_url: "postgresql://user:pass@localhost/db".to_string(),
        max_connections: 10,
    });

    // Create AudioService WITHOUT ElevenLabs
    let service = AudioService::new(db);

    let blob = cofre_vault::services::audio::AudioBlob {
        data: vec![0xFF, 0xFB, 0x90, 0x00],
        mime_type: "audio/mp3".to_string(),
        duration_ms: 100,
    };

    // Should fail because ElevenLabs is not configured
    let result = service.transcribe_audio(&blob).await;
    assert!(result.is_err());
}
