// RagChatService — retrieval-augmented generation chat over vault content.
// Embeds the user message, finds similar vault items via pgvector, builds a
// RAG system prompt, and calls the Gemini generateContent API.
//
// Gemini chat endpoint:
//   POST https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}

use std::sync::Arc;
use std::time::Duration;

use uuid::Uuid;

use crate::{
    db::{find_similar_items, SimilarResult},
    error::{Error, Result},
    models::ChatResponse,
    services::EmbeddingService,
};

// ── Internal serde types ──────────────────────────────────────────────────────

// Gemini generateContent request/response shapes
#[derive(serde::Serialize)]
struct GenerateContentRequest {
    system_instruction: GeminiSystemInstruction,
    contents: Vec<GeminiContent>,
    generation_config: GenerationConfig,
}

#[derive(serde::Serialize)]
struct GeminiSystemInstruction {
    parts: Vec<GeminiPart>,
}

#[derive(serde::Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(serde::Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(serde::Serialize)]
struct GenerationConfig {
    max_output_tokens: u32,
}

#[derive(serde::Deserialize)]
struct GenerateContentResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(serde::Deserialize)]
struct GeminiCandidate {
    content: GeminiCandidateContent,
}

#[derive(serde::Deserialize)]
struct GeminiCandidateContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(serde::Deserialize)]
struct GeminiResponsePart {
    text: String,
}

// ── Service ───────────────────────────────────────────────────────────────────

/// Orchestrates RAG-based chat: embed → similarity search → prompt → Gemini LLM.
pub struct RagChatService {
    embedding_service: Arc<EmbeddingService>,
    db: Arc<sqlx::PgPool>,
    llm_model: String,
    gemini_api_key: String,
    client: reqwest::Client,
    top_k: usize,
}

impl RagChatService {
    /// Construct from environment variables.
    ///
    /// - `LLM_MODEL`: default `"gemini-1.5-flash"`
    /// - `GEMINI_API_KEY`: required
    pub fn from_env(
        embedding_service: Arc<EmbeddingService>,
        db: Arc<sqlx::PgPool>,
    ) -> Result<Self> {
        let llm_model =
            std::env::var("LLM_MODEL").unwrap_or_else(|_| "gemini-1.5-flash".to_string());

        let gemini_api_key = std::env::var("GEMINI_API_KEY").map_err(|_| {
            Error::AuthenticationFailed(
                "GEMINI_API_KEY environment variable is required".to_string(),
            )
        })?;

        Ok(Self {
            embedding_service,
            db,
            llm_model,
            gemini_api_key,
            client: reqwest::Client::new(),
            top_k: 5,
        })
    }

    /// Main entry point: embed message → similarity search → LLM → [`ChatResponse`].
    pub async fn process_message(
        &self,
        vault_id: Uuid,
        _user_id: Uuid,
        message: &str,
    ) -> Result<ChatResponse> {
        // Step 1: embed the user message
        let query_vector = self.embedding_service.generate_embedding(message).await?;

        // Step 2: find top-k similar vault items
        let similar: Vec<SimilarResult> =
            find_similar_items(&self.db, vault_id, &query_vector, self.top_k as i64).await?;

        // Step 3: build RAG system prompt
        let system_prompt = build_system_prompt(&similar);

        // Step 4: call OpenAI Chat Completions
        let chat_reply_text = self.call_chat_api(&system_prompt, message).await?;

        // Step 5: collect referenced node IDs (ordered by similarity, highest first)
        let referenced_node_ids = similar.iter().map(|r| r.item.id).collect();

        Ok(ChatResponse {
            chat_reply_text,
            referenced_node_ids,
        })
    }

    async fn call_chat_api(&self, system_prompt: &str, user_message: &str) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.llm_model, self.gemini_api_key
        );

        let request_body = GenerateContentRequest {
            system_instruction: GeminiSystemInstruction {
                parts: vec![GeminiPart { text: system_prompt.to_string() }],
            },
            contents: vec![GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart { text: user_message.to_string() }],
            }],
            generation_config: GenerationConfig { max_output_tokens: 1024 },
        };

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| Error::ChatGenerationFailed(e.to_string()))?;

        let status = response.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(Error::RateLimitExceeded);
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_else(|_| status.to_string());
            return Err(Error::ChatGenerationFailed(body));
        }

        let parsed: GenerateContentResponse = response
            .json()
            .await
            .map_err(|e| Error::ChatGenerationFailed(e.to_string()))?;

        parsed
            .candidates
            .into_iter()
            .next()
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text)
            .ok_or_else(|| Error::ChatGenerationFailed("empty candidates in Gemini response".to_string()))
    }
}

// ── Prompt builder (pure function — easy to unit-test) ────────────────────────

/// Build the RAG system prompt from a list of similar vault items.
pub(crate) fn build_system_prompt(similar: &[SimilarResult]) -> String {
    let mut prompt = String::from(
        "You are a helpful assistant with access to the user's vault content.\n\
         Answer the user's question using ONLY the provided context items.\n\
         Reference specific items by their ID when relevant.\n\
         \n\
         Context items:\n",
    );

    if similar.is_empty() {
        prompt.push_str("(No relevant items found in vault)");
    } else {
        for (i, result) in similar.iter().enumerate() {
            let item = &result.item;

            let title = item
                .title
                .as_deref()
                .filter(|t| !t.is_empty())
                .unwrap_or("Untitled");

            // Content: transcript > metadata["scraped_text"] > title > "(no content)"
            let content: String = item
                .transcript
                .as_deref()
                .filter(|t| !t.is_empty())
                .map(str::to_string)
                .or_else(|| {
                    item.metadata
                        .as_ref()
                        .and_then(|m| m.get("scraped_text"))
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(str::to_string)
                })
                .or_else(|| {
                    item.title
                        .as_deref()
                        .filter(|t| !t.is_empty())
                        .map(str::to_string)
                })
                .unwrap_or_else(|| "(no content)".to_string());

            prompt.push_str(&format!(
                "[Item ID: {}]\nTitle: {}\nContent: {}\n",
                item.id, title, content
            ));

            // Separator between items (not after the last one)
            if i < similar.len() - 1 {
                prompt.push_str("\n---\n");
            }
        }
    }

    prompt
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ContentItem, ContentType};
    use chrono::Utc;

    fn make_similar(
        id: Uuid,
        vault_id: Uuid,
        title: Option<&str>,
        transcript: Option<&str>,
        metadata: Option<serde_json::Value>,
    ) -> SimilarResult {
        SimilarResult {
            item: ContentItem {
                id,
                vault_id,
                created_by: Uuid::new_v4(),
                content_type: ContentType::Link,
                title: title.map(str::to_string),
                url: "https://example.com".to_string(),
                transcript: transcript.map(str::to_string),
                metadata,
                created_at: Utc::now(),
            },
            similarity: 0.9,
        }
    }

    // ── Task 8.3: RAG prompt construction tests ───────────────────────────────

    #[test]
    fn prompt_contains_item_id_and_content() {
        let vault_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let similar = vec![make_similar(
            item_id,
            vault_id,
            Some("My Article"),
            Some("This is the transcript text."),
            None,
        )];

        let prompt = build_system_prompt(&similar);

        assert!(
            prompt.contains(&item_id.to_string()),
            "prompt should contain the item UUID"
        );
        assert!(
            prompt.contains("My Article"),
            "prompt should contain the item title"
        );
        assert!(
            prompt.contains("This is the transcript text."),
            "prompt should contain the transcript content"
        );
    }

    #[test]
    fn prompt_does_not_contain_user_message() {
        let vault_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let similar = vec![make_similar(
            item_id,
            vault_id,
            Some("Title"),
            Some("Some content"),
            None,
        )];

        let user_message = "What do you know about machine learning?";
        let prompt = build_system_prompt(&similar);

        // The system prompt is built independently of the user message
        assert!(
            !prompt.contains(user_message),
            "system prompt should not contain the user message"
        );
    }

    #[test]
    fn empty_context_produces_valid_prompt() {
        let prompt = build_system_prompt(&[]);

        assert!(
            prompt.contains("(No relevant items found in vault)"),
            "empty context should include the no-items placeholder"
        );
        // Should still have the preamble
        assert!(
            prompt.contains("You are a helpful assistant"),
            "prompt should still have the assistant preamble"
        );
    }

    #[test]
    fn multiple_items_all_appear_in_prompt() {
        let vault_id = Uuid::new_v4();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let similar = vec![
            make_similar(id1, vault_id, Some("First"), Some("Content one"), None),
            make_similar(id2, vault_id, Some("Second"), Some("Content two"), None),
        ];

        let prompt = build_system_prompt(&similar);

        assert!(prompt.contains(&id1.to_string()));
        assert!(prompt.contains(&id2.to_string()));
        assert!(prompt.contains("Content one"));
        assert!(prompt.contains("Content two"));
    }

    #[test]
    fn item_without_transcript_uses_scraped_text() {
        let vault_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let similar = vec![make_similar(
            item_id,
            vault_id,
            Some("Link Title"),
            None,
            Some(serde_json::json!({ "scraped_text": "scraped page body" })),
        )];

        let prompt = build_system_prompt(&similar);
        assert!(prompt.contains("scraped page body"));
    }

    #[test]
    fn item_with_no_content_shows_placeholder() {
        let vault_id = Uuid::new_v4();
        let item_id = Uuid::new_v4();
        let similar = vec![make_similar(item_id, vault_id, None, None, None)];

        let prompt = build_system_prompt(&similar);
        assert!(prompt.contains("(no content)"));
        assert!(prompt.contains("Untitled"));
    }
}
