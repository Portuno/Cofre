// EmbeddingService — wraps the Google Gemini embeddings API
// Handles single and batch embedding generation, and content text extraction.
//
// Gemini embedding endpoint:
//   POST https://generativelanguage.googleapis.com/v1beta/models/{model}:embedContent?key={api_key}
//
// Default model: text-embedding-004  (768 dimensions)

use std::time::Duration;

use crate::{
    error::{Error, Result},
    models::{AiConfig, ContentItem},
};

const GEMINI_EMBED_BASE: &str =
    "https://generativelanguage.googleapis.com/v1beta/models";

// ── Internal serde types ──────────────────────────────────────────────────────

#[derive(serde::Serialize)]
struct EmbedContentRequest<'a> {
    model: &'a str,
    content: GeminiContent<'a>,
}

#[derive(serde::Serialize)]
struct GeminiContent<'a> {
    parts: Vec<GeminiPart<'a>>,
}

#[derive(serde::Serialize)]
struct GeminiPart<'a> {
    text: &'a str,
}

#[derive(serde::Deserialize)]
struct EmbedContentResponse {
    embedding: GeminiEmbedding,
}

#[derive(serde::Deserialize)]
struct GeminiEmbedding {
    values: Vec<f32>,
}

// ── Service ───────────────────────────────────────────────────────────────────

/// Calls the Gemini embeddings endpoint to produce `Vec<f32>` vectors (768-dim).
pub struct EmbeddingService {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl EmbeddingService {
    /// Construct from environment variables via [`AiConfig::from_env`].
    pub fn from_env() -> Result<Self> {
        let config = AiConfig::from_env()?;
        Ok(Self {
            client: reqwest::Client::new(),
            api_key: config.gemini_api_key,
            model: config.embedding_model,
        })
    }

    /// Select the richest available text from a [`ContentItem`] for embedding.
    ///
    /// Priority:
    /// 1. `transcript` (if `Some` and non-empty)
    /// 2. `metadata["scraped_text"]` as a string (if present)
    /// 3. `title` (if `Some` and non-empty)
    /// 4. Empty string — logs a warning
    pub fn extract_content_text(item: &ContentItem) -> String {
        if let Some(t) = &item.transcript {
            if !t.is_empty() {
                return t.clone();
            }
        }

        if let Some(meta) = &item.metadata {
            if let Some(scraped) = meta.get("scraped_text") {
                if let Some(s) = scraped.as_str() {
                    if !s.is_empty() {
                        return s.to_string();
                    }
                }
            }
        }

        if let Some(title) = &item.title {
            if !title.is_empty() {
                return title.clone();
            }
        }

        tracing::warn!(
            item_id = %item.id,
            "No embeddable text found for content item; embedding empty string"
        );
        String::new()
    }

    /// Generate a single embedding vector for `text`.
    ///
    /// - Applies a 5-second timeout.
    /// - HTTP 429 → [`Error::RateLimitExceeded`]
    /// - Other non-2xx → [`Error::EmbeddingGenerationFailed`]
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!(
            "{}/{}:embedContent?key={}",
            GEMINI_EMBED_BASE, self.model, self.api_key
        );

        let body = EmbedContentRequest {
            model: &format!("models/{}", self.model),
            content: GeminiContent {
                parts: vec![GeminiPart { text }],
            },
        };

        let response = self
            .client
            .post(&url)
            .json(&body)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| Error::EmbeddingGenerationFailed(e.to_string()))?;

        let status = response.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(Error::RateLimitExceeded);
        }

        if !status.is_success() {
            let msg = response
                .text()
                .await
                .unwrap_or_else(|_| status.to_string());
            return Err(Error::EmbeddingGenerationFailed(msg));
        }

        let parsed: EmbedContentResponse = response
            .json()
            .await
            .map_err(|e| Error::EmbeddingGenerationFailed(e.to_string()))?;

        Ok(parsed.embedding.values)
    }

    /// Generate embeddings for multiple texts by calling the API once per text.
    ///
    /// Gemini's embedContent endpoint is single-text only, so this fans out
    /// requests sequentially. Results are returned in the same order as `texts`.
    pub async fn generate_embeddings_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.generate_embedding(text).await?);
        }
        Ok(results)
    }
}

/// Backfill embeddings for all items in a vault that currently have NULL `content_embedding`.
pub async fn backfill_embeddings(
    pool: &sqlx::PgPool,
    embedding_service: &EmbeddingService,
    vault_id: uuid::Uuid,
) -> crate::error::Result<()> {
    let item_ids = crate::db::find_items_without_embeddings(pool, vault_id).await?;

    if item_ids.is_empty() {
        tracing::info!(vault_id = %vault_id, "No items need embedding backfill");
        return Ok(());
    }

    tracing::info!(vault_id = %vault_id, count = item_ids.len(), "Starting embedding backfill");

    let mut processed = 0usize;
    let mut errors = 0usize;

    for id in &item_ids {
        let row = sqlx::query_as::<_, (uuid::Uuid, Option<String>, Option<String>, Option<String>, Option<serde_json::Value>)>(
            "SELECT id, title, url, transcript, metadata FROM content_items WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| crate::error::Error::DatabaseError(e.to_string()))?;

        if let Some((item_id, title, url, transcript, metadata)) = row {
            let item = crate::models::ContentItem {
                id: item_id,
                vault_id,
                created_by: uuid::Uuid::nil(),
                content_type: crate::models::ContentType::Link,
                title,
                url: url.unwrap_or_default(),
                transcript,
                metadata,
                created_at: chrono::Utc::now(),
            };
            let text = EmbeddingService::extract_content_text(&item);

            match embedding_service.generate_embedding(&text).await {
                Ok(embedding) => {
                    match crate::db::upsert_embedding(pool, item_id, &embedding).await {
                        Ok(()) => {
                            processed += 1;
                            tracing::info!(item_id = %item_id, "Embedding upserted ({}/{})", processed, item_ids.len());
                        }
                        Err(e) => {
                            errors += 1;
                            tracing::warn!(item_id = %item_id, error = %e, "Failed to upsert embedding");
                        }
                    }
                }
                Err(e) => {
                    errors += 1;
                    tracing::warn!(item_id = %item_id, error = %e, "Failed to generate embedding");
                }
            }
        }
    }

    tracing::info!(vault_id = %vault_id, processed, errors, "Embedding backfill complete");
    Ok(())
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ContentItem, ContentType};
    use chrono::Utc;
    use uuid::Uuid;

    fn make_item(
        transcript: Option<&str>,
        metadata: Option<serde_json::Value>,
        title: Option<&str>,
    ) -> ContentItem {
        ContentItem {
            id: Uuid::new_v4(),
            vault_id: Uuid::new_v4(),
            created_by: Uuid::new_v4(),
            content_type: ContentType::Link,
            title: title.map(str::to_string),
            url: "https://example.com".to_string(),
            transcript: transcript.map(str::to_string),
            metadata,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn transcript_takes_priority() {
        let item = make_item(
            Some("audio transcript text"),
            Some(serde_json::json!({ "scraped_text": "scraped" })),
            Some("A Title"),
        );
        assert_eq!(EmbeddingService::extract_content_text(&item), "audio transcript text");
    }

    #[test]
    fn scraped_text_used_when_transcript_absent() {
        let item = make_item(
            None,
            Some(serde_json::json!({ "scraped_text": "scraped page content" })),
            Some("A Title"),
        );
        assert_eq!(EmbeddingService::extract_content_text(&item), "scraped page content");
    }

    #[test]
    fn title_as_final_fallback() {
        let item = make_item(None, None, Some("Just a Title"));
        assert_eq!(EmbeddingService::extract_content_text(&item), "Just a Title");
    }

    #[test]
    fn empty_string_when_all_absent() {
        let item = make_item(None, None, None);
        assert_eq!(EmbeddingService::extract_content_text(&item), "");
    }

    #[test]
    fn empty_transcript_falls_through_to_scraped_text() {
        let item = make_item(
            Some(""),
            Some(serde_json::json!({ "scraped_text": "scraped" })),
            Some("Title"),
        );
        assert_eq!(EmbeddingService::extract_content_text(&item), "scraped");
    }

    #[test]
    fn empty_scraped_text_falls_through_to_title() {
        let item = make_item(
            None,
            Some(serde_json::json!({ "scraped_text": "" })),
            Some("Title"),
        );
        assert_eq!(EmbeddingService::extract_content_text(&item), "Title");
    }
}
