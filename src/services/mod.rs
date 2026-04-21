// Services module for Cofre Vault Platform
// Provides high-level business logic for authentication, vault management, content, and graph operations

pub mod auth;
pub mod vault;
pub mod tag;
pub mod content;
pub mod audio;
pub mod graph;
pub mod elevenlabs;
pub mod embedding;
pub mod rag_chat;

pub use auth::AuthService;
pub use vault::VaultService;
pub use tag::TagService;
pub use content::ContentService;
pub use audio::AudioService;
pub use graph::SemanticGraphEngine;
pub use elevenlabs::ElevenLabsClient;
pub use embedding::EmbeddingService;
pub use rag_chat::RagChatService;
