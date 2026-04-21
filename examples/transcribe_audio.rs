// Example: Transcribe an audio file using ElevenLabs
// Usage: cargo run --example transcribe_audio <audio_file_path>
//
// Example:
//   cargo run --example transcribe_audio test.mp3
//   cargo run --example transcribe_audio recording.wav

use cofre_vault::{Database, services::AudioService};
use std::env;
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() {
    // Load environment variables from .env
    dotenv::dotenv().ok();

    // Get audio file path from command line
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --example transcribe_audio <audio_file_path>");
        eprintln!("\nExample:");
        eprintln!("  cargo run --example transcribe_audio test.mp3");
        std::process::exit(1);
    }

    let file_path = &args[1];
    let path = Path::new(file_path);

    // Check if file exists
    if !path.exists() {
        eprintln!("❌ Error: File not found: {}", file_path);
        std::process::exit(1);
    }

    println!("📂 Reading audio file: {}", file_path);

    // Read audio file
    let audio_data = match fs::read(path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("❌ Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    println!("📊 File size: {} bytes", audio_data.len());

    // Detect MIME type from file extension
    let mime_type = match path.extension().and_then(|s| s.to_str()) {
        Some("mp3") => "audio/mp3",
        Some("wav") => "audio/wav",
        Some("webm") => "audio/webm",
        Some("ogg") => "audio/ogg",
        Some("aac") => "audio/aac",
        Some("flac") => "audio/flac",
        Some("m4a") => "audio/mp4",
        _ => {
            eprintln!("❌ Unsupported file format. Supported: mp3, wav, webm, ogg, aac, flac, m4a");
            std::process::exit(1);
        }
    };

    println!("🎵 MIME type: {}", mime_type);

    // Create database (mock is fine for this example)
    let db = Database::new(cofre_vault::db::DatabaseConfig {
        supabase_url: "https://example.supabase.co".to_string(),
        supabase_key: "test-key".to_string(),
        database_url: "postgresql://user:pass@localhost/db".to_string(),
        max_connections: 10,
    });

    // Create AudioService with ElevenLabs from environment
    println!("🔧 Initializing ElevenLabs client...");
    let service = AudioService::from_env(db);

    // Create audio blob
    let blob = cofre_vault::services::audio::AudioBlob {
        data: audio_data,
        mime_type: mime_type.to_string(),
        duration_ms: 0, // Duration not needed for transcription
    };

    // Transcribe
    println!("🎙️  Sending to ElevenLabs for transcription...");
    println!("⏳ This may take a few seconds...\n");

    match service.transcribe_audio(&blob).await {
        Ok(transcript) => {
            println!("✅ Transcription successful!\n");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("📝 TRANSCRIPT:");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("{}", transcript);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
            println!("✨ Done!");
        }
        Err(e) => {
            eprintln!("\n❌ Transcription failed: {:?}", e);
            eprintln!("\n💡 Troubleshooting:");
            eprintln!("   1. Check that ELEVENLABS_API_KEY is set in .env");
            eprintln!("   2. Verify your API key is valid");
            eprintln!("   3. Ensure the audio file is in a supported format");
            eprintln!("   4. Check your internet connection");
            std::process::exit(1);
        }
    }
}
