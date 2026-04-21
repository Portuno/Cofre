// Database connection pool and migration framework for Cofre Vault Platform
// Handles PostgreSQL connections via Supabase

use crate::error::{Error, Result};
use crate::models::{ContentItem, ContentType};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

/// Database connection pool configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub supabase_url: String,
    pub supabase_key: String,
    pub database_url: String,
    pub max_connections: u32,
}

impl DatabaseConfig {
    /// Create a new database configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let supabase_url = std::env::var("SUPABASE_URL")
            .map_err(|_| Error::InternalError("SUPABASE_URL not set".to_string()))?;
        let supabase_key = std::env::var("SUPABASE_KEY")
            .map_err(|_| Error::InternalError("SUPABASE_KEY not set".to_string()))?;
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| Error::InternalError("DATABASE_URL not set".to_string()))?;

        Ok(DatabaseConfig {
            supabase_url,
            supabase_key,
            database_url,
            max_connections: 10,
        })
    }
}

/// Database connection pool and client
pub struct Database {
    config: Arc<DatabaseConfig>,
}

impl Database {
    /// Create a new database instance with the given configuration
    pub fn new(config: DatabaseConfig) -> Self {
        Database {
            config: Arc::new(config),
        }
    }

    /// Create a database instance from environment variables
    pub fn from_env() -> Result<Self> {
        let config = DatabaseConfig::from_env()?;
        Ok(Database::new(config))
    }

    /// Get the Supabase URL
    pub fn supabase_url(&self) -> &str {
        &self.config.supabase_url
    }

    /// Get the Supabase API key
    pub fn supabase_key(&self) -> &str {
        &self.config.supabase_key
    }

    /// Get the database URL
    pub fn database_url(&self) -> &str {
        &self.config.database_url
    }

    /// Initialize the database connection pool
    /// This would typically be called during application startup
    pub async fn initialize(&self) -> Result<()> {
        // In a real implementation, this would:
        // 1. Create a connection pool using sqlx or similar
        // 2. Run migrations
        // 3. Verify connectivity
        // For now, we just verify the configuration is valid
        if self.config.supabase_url.is_empty() {
            return Err(Error::DatabaseError("Supabase URL is empty".to_string()));
        }
        if self.config.database_url.is_empty() {
            return Err(Error::DatabaseError("Database URL is empty".to_string()));
        }
        Ok(())
    }

    /// Run database migrations
    /// This would apply all pending migrations to the database
    pub async fn migrate(&self) -> Result<()> {
        // In a real implementation, this would:
        // 1. Connect to the database
        // 2. Check the migrations table
        // 3. Apply any pending migrations
        // For now, this is a placeholder
        Ok(())
    }

    /// Health check - verify database connectivity
    pub async fn health_check(&self) -> Result<()> {
        // In a real implementation, this would execute a simple query
        // to verify the database is accessible
        Ok(())
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            config: Arc::clone(&self.config),
        }
    }
}

/// A content item paired with its cosine similarity score
#[derive(Debug, Clone)]
pub struct SimilarResult {
    pub item: ContentItem,
    pub similarity: f32,
}

/// Upsert the embedding vector for a content item.
pub async fn upsert_embedding(pool: &PgPool, item_id: Uuid, embedding: &[f32]) -> Result<()> {
    let vector = pgvector::Vector::from(embedding.to_vec());
    sqlx::query(
        "UPDATE content_items SET content_embedding = $1 WHERE id = $2",
    )
    .bind(vector)
    .bind(item_id)
    .execute(pool)
    .await
    .map_err(|e| Error::DatabaseError(e.to_string()))?;
    Ok(())
}

/// Find the top-k most similar items to a query vector within a vault.
pub async fn find_similar_items(
    pool: &PgPool,
    vault_id: Uuid,
    query_vector: &[f32],
    limit: i64,
) -> Result<Vec<SimilarResult>> {
    let vector = pgvector::Vector::from(query_vector.to_vec());
    let rows = sqlx::query(
        r#"
        SELECT id, vault_id, created_by, content_type, title, url, transcript, metadata, created_at,
               1.0 - (content_embedding <=> $1) AS similarity
        FROM content_items
        WHERE vault_id = $2
          AND content_embedding IS NOT NULL
        ORDER BY content_embedding <=> $1
        LIMIT $3
        "#,
    )
    .bind(vector)
    .bind(vault_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| Error::DatabaseError(e.to_string()))?;

    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        results.push(row_to_similar_result(&row)?);
    }
    Ok(results)
}

/// Find similar items to a known item by its stored embedding.
pub async fn find_similar_to_item(
    pool: &PgPool,
    vault_id: Uuid,
    item_id: Uuid,
    threshold: f32,
    limit: i64,
) -> Result<Vec<SimilarResult>> {
    let rows = sqlx::query(
        r#"
        SELECT b.id, b.vault_id, b.created_by, b.content_type, b.title, b.url, b.transcript, b.metadata, b.created_at,
               1.0 - (b.content_embedding <=> a.content_embedding) AS similarity
        FROM content_items a
        JOIN content_items b ON b.vault_id = a.vault_id
        WHERE a.id = $1
          AND b.id != $1
          AND b.vault_id = $2
          AND b.content_embedding IS NOT NULL
          AND a.content_embedding IS NOT NULL
          AND 1.0 - (b.content_embedding <=> a.content_embedding) >= $3
        ORDER BY b.content_embedding <=> a.content_embedding
        LIMIT $4
        "#,
    )
    .bind(item_id)
    .bind(vault_id)
    .bind(threshold)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| Error::DatabaseError(e.to_string()))?;

    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        results.push(row_to_similar_result(&row)?);
    }
    Ok(results)
}

/// Return all item IDs in a vault that have NULL content_embedding.
pub async fn find_items_without_embeddings(pool: &PgPool, vault_id: Uuid) -> Result<Vec<Uuid>> {
    let rows = sqlx::query(
        "SELECT id FROM content_items WHERE vault_id = $1 AND content_embedding IS NULL",
    )
    .bind(vault_id)
    .fetch_all(pool)
    .await
    .map_err(|e| Error::DatabaseError(e.to_string()))?;

    let ids = rows
        .iter()
        .map(|row| {
            use sqlx::Row;
            row.try_get::<Uuid, _>("id")
                .map_err(|e| Error::DatabaseError(e.to_string()))
        })
        .collect::<Result<Vec<Uuid>>>()?;
    Ok(ids)
}

/// Helper: map a sqlx Row to a SimilarResult.
fn row_to_similar_result(row: &sqlx::postgres::PgRow) -> Result<SimilarResult> {
    use sqlx::Row;

    let id: Uuid = row.try_get("id").map_err(|e| Error::DatabaseError(e.to_string()))?;
    let vault_id: Uuid = row.try_get("vault_id").map_err(|e| Error::DatabaseError(e.to_string()))?;
    let created_by: Uuid = row.try_get("created_by").map_err(|e| Error::DatabaseError(e.to_string()))?;
    let content_type_str: String = row.try_get("content_type").map_err(|e| Error::DatabaseError(e.to_string()))?;
    let title: Option<String> = row.try_get("title").map_err(|e| Error::DatabaseError(e.to_string()))?;
    let url: String = row.try_get("url").map_err(|e| Error::DatabaseError(e.to_string()))?;
    let transcript: Option<String> = row.try_get("transcript").map_err(|e| Error::DatabaseError(e.to_string()))?;
    let metadata: Option<serde_json::Value> = row.try_get("metadata").map_err(|e| Error::DatabaseError(e.to_string()))?;
    let created_at: DateTime<Utc> = row.try_get("created_at").map_err(|e| Error::DatabaseError(e.to_string()))?;
    let similarity: f32 = row.try_get("similarity").map_err(|e| Error::DatabaseError(e.to_string()))?;

    let content_type = match content_type_str.as_str() {
        "audio" => ContentType::Audio,
        "image" => ContentType::Image,
        "link" => ContentType::Link,
        other => return Err(Error::DatabaseError(format!("Unknown content_type: {other}"))),
    };

    Ok(SimilarResult {
        item: ContentItem {
            id,
            vault_id,
            created_by,
            content_type,
            title,
            url,
            transcript,
            metadata,
            created_at,
        },
        similarity,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_creation() {
        let config = DatabaseConfig {
            supabase_url: "https://example.supabase.co".to_string(),
            supabase_key: "test-key".to_string(),
            database_url: "postgresql://user:pass@localhost/db".to_string(),
            max_connections: 10,
        };

        assert_eq!(config.supabase_url, "https://example.supabase.co");
        assert_eq!(config.supabase_key, "test-key");
        assert_eq!(config.max_connections, 10);
    }

    #[test]
    fn test_database_creation() {
        let config = DatabaseConfig {
            supabase_url: "https://example.supabase.co".to_string(),
            supabase_key: "test-key".to_string(),
            database_url: "postgresql://user:pass@localhost/db".to_string(),
            max_connections: 10,
        };

        let db = Database::new(config);
        assert_eq!(db.supabase_url(), "https://example.supabase.co");
        assert_eq!(db.supabase_key(), "test-key");
    }

    #[tokio::test]
    async fn test_database_initialization() {
        let config = DatabaseConfig {
            supabase_url: "https://example.supabase.co".to_string(),
            supabase_key: "test-key".to_string(),
            database_url: "postgresql://user:pass@localhost/db".to_string(),
            max_connections: 10,
        };

        let db = Database::new(config);
        let result = db.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_database_clone() {
        let config = DatabaseConfig {
            supabase_url: "https://example.supabase.co".to_string(),
            supabase_key: "test-key".to_string(),
            database_url: "postgresql://user:pass@localhost/db".to_string(),
            max_connections: 10,
        };

        let db = Database::new(config);
        let db_clone = db.clone();
        assert_eq!(db.supabase_url(), db_clone.supabase_url());
    }
}
