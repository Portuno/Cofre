// Core data models for Cofre Vault Platform
// Represents all domain entities: Vault, VaultMember, VaultInvite, ContentItem, Tag, ItemTag

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Represents a shared collaborative vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Represents a user's membership in a vault with role assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultMember {
    pub vault_id: Uuid,
    pub user_id: Uuid,
    pub role: MemberRole,
    pub joined_at: DateTime<Utc>,
}

/// Role types for vault members
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemberRole {
    Owner,
    Member,
}

impl std::fmt::Display for MemberRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemberRole::Owner => write!(f, "owner"),
            MemberRole::Member => write!(f, "member"),
        }
    }
}

/// Represents an invitation to join a vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInvite {
    pub id: Uuid,
    pub vault_id: Uuid,
    pub invited_email: String,
    pub token: String,
    pub accepted: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Represents a discrete piece of content stored in a vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentItem {
    pub id: Uuid,
    pub vault_id: Uuid,
    pub created_by: Uuid,
    pub content_type: ContentType,
    pub title: Option<String>,
    pub url: String,
    pub transcript: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Types of content that can be stored in a vault
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Audio,
    Image,
    Link,
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::Audio => write!(f, "audio"),
            ContentType::Image => write!(f, "image"),
            ContentType::Link => write!(f, "link"),
        }
    }
}

/// Represents a label applied to content items for semantic organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub vault_id: Uuid,
    pub name: String,
    pub is_special: bool,
    pub color: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Join table linking content items to tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTag {
    pub item_id: Uuid,
    pub tag_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// In-memory representation of a node in the semantic graph
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub item: ContentItem,
    pub edges: Vec<GraphEdge>,
}

/// Represents a connection between two items in the semantic graph
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub target_item_id: Uuid,
    pub shared_tag: Tag,
    pub weight: f32,
}

/// In-memory semantic graph representation
#[derive(Debug, Clone)]
pub struct Graph {
    pub nodes: std::collections::HashMap<Uuid, GraphNode>,
}

impl Graph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Graph {
            nodes: std::collections::HashMap::new(),
        }
    }

    /// Get a node by item ID
    pub fn get_node(&self, item_id: &Uuid) -> Option<&GraphNode> {
        self.nodes.get(item_id)
    }

    /// Get all nodes in the graph
    pub fn all_nodes(&self) -> impl Iterator<Item = &GraphNode> {
        self.nodes.values()
    }

    /// Get the number of nodes in the graph
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the total number of edges in the graph
    pub fn edge_count(&self) -> usize {
        self.nodes.values().map(|n| n.edges.len()).sum::<usize>() / 2
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

/// Input structure for creating a new vault
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVaultInput {
    pub name: String,
    pub description: Option<String>,
}

/// Input structure for creating a new tag
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTagInput {
    pub name: String,
    pub is_special: bool,
    pub color: Option<String>,
}

/// Input structure for creating a new content item
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateContentItemInput {
    pub content_type: ContentType,
    pub title: Option<String>,
    pub url: String,
    pub transcript: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Input structure for attaching tags to a content item
#[derive(Debug, Serialize, Deserialize)]
pub struct AttachTagsInput {
    pub tag_ids: Vec<Uuid>,
}

/// User information (minimal representation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
}

/// Authentication result containing user and session info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub user: User,
    pub session_token: String,
}

/// Configuration for AI/Gemini services
#[derive(Debug, Clone)]
pub struct AiConfig {
    pub gemini_api_key: String,
    pub embedding_model: String,
    pub llm_model: String,
    pub similarity_threshold: f32,
}

impl AiConfig {
    /// Load AI configuration from environment variables.
    ///
    /// - `GEMINI_API_KEY`: required
    /// - `EMBEDDING_MODEL`: default `text-embedding-004`
    /// - `LLM_MODEL`: default `gemini-1.5-flash`
    /// - `SIMILARITY_THRESHOLD`: default `0.8`, must be in [0.0, 1.0]
    pub fn from_env() -> crate::error::Result<Self> {
        let gemini_api_key = std::env::var("GEMINI_API_KEY")
            .map_err(|_| crate::error::Error::AuthenticationFailed(
                "GEMINI_API_KEY environment variable is required".to_string(),
            ))?;

        let embedding_model = std::env::var("EMBEDDING_MODEL")
            .unwrap_or_else(|_| "text-embedding-004".to_string());

        const VALID_EMBEDDING_MODELS: &[&str] = &[
            "text-embedding-004",
            "embedding-001",
        ];
        if !VALID_EMBEDDING_MODELS.contains(&embedding_model.as_str()) {
            return Err(crate::error::Error::InvalidEmbeddingModel(embedding_model));
        }

        let llm_model = std::env::var("LLM_MODEL")
            .unwrap_or_else(|_| "gemini-1.5-flash".to_string());

        const VALID_LLM_MODELS: &[&str] = &[
            "gemini-1.5-flash",
            "gemini-1.5-pro",
            "gemini-2.0-flash",
            "gemini-2.5-pro",
        ];
        if !VALID_LLM_MODELS.contains(&llm_model.as_str()) {
            return Err(crate::error::Error::InvalidLlmModel(llm_model));
        }

        let similarity_threshold = std::env::var("SIMILARITY_THRESHOLD")
            .unwrap_or_else(|_| "0.8".to_string())
            .parse::<f32>()
            .map_err(|_| crate::error::Error::InvalidSimilarityThreshold(f32::NAN))?;

        if !(0.0..=1.0).contains(&similarity_threshold) {
            return Err(crate::error::Error::InvalidSimilarityThreshold(similarity_threshold));
        }

        Ok(Self {
            gemini_api_key,
            embedding_model,
            llm_model,
            similarity_threshold,
        })
    }
}

/// Incoming chat request payload
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
}

/// Outgoing chat response payload
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub chat_reply_text: String,
    pub referenced_node_ids: Vec<Uuid>,
}
