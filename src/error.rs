// Error handling types and Result wrapper for Cofre Vault Platform

use thiserror::Error;
use uuid::Uuid;

/// Custom error type for Cofre Vault Platform operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Authorization failed: user is not a member of this vault")]
    Unauthorized,

    #[error("Vault not found")]
    VaultNotFound,

    #[error("User not found")]
    UserNotFound,

    #[error("Content item not found")]
    ContentItemNotFound,

    #[error("Tag not found")]
    TagNotFound,

    #[error("Invite not found")]
    InviteNotFound,

    #[error("Invite already used")]
    InviteAlreadyUsed,

    #[error("Invite expired")]
    InviteExpired,

    #[error("Invalid invite token")]
    InvalidInviteToken,

    #[error("Tag name already exists in this vault")]
    DuplicateTagName,

    #[error("Duplicate tag attachment")]
    DuplicateTagAttachment,

    #[error("Vault name is required and must not exceed 100 characters")]
    InvalidVaultName,

    #[error("Tag name is required")]
    InvalidTagName,

    #[error("Invalid URL format")]
    InvalidUrl,

    #[error("Storage upload failed: {0}")]
    StorageUploadFailed(String),

    #[error("Transcription failed: {0}")]
    TranscriptionFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Embedding generation failed: {0}")]
    EmbeddingGenerationFailed(String),

    #[error("Chat generation failed: {0}")]
    ChatGenerationFailed(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid similarity threshold: {0} (must be in [0.0, 1.0])")]
    InvalidSimilarityThreshold(f32),

    #[error("Invalid embedding model: {0}")]
    InvalidEmbeddingModel(String),

    #[error("Invalid LLM model: {0}")]
    InvalidLlmModel(String),

    #[error("Item not found: {0}")]
    ItemNotFound(Uuid),
}

/// Result type alias for Cofre Vault Platform operations
pub type Result<T> = std::result::Result<T, Error>;
