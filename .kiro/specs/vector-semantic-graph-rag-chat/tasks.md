# Implementation Plan: Vector Semantic Graph RAG Chat

## Overview

Upgrade the Cofre vault from a tag-only semantic graph to a hybrid vector + tag graph, and add a RAG chat endpoint. Tasks proceed in dependency order: schema → models/errors → embedding service → database layer → graph engine → RAG chat service → axum route → wiring.

## Tasks

- [x] 1. SQL migration for pgvector
  - Create `migrations/001_pgvector.sql`
  - Enable `pgvector` extension with `CREATE EXTENSION IF NOT EXISTS vector`
  - Add `content_embedding vector(1536)` column (nullable) to `content_items` with `ADD COLUMN IF NOT EXISTS`
  - Create HNSW index with `vector_cosine_ops`, `m = 16`, `ef_construction = 64`, using `CREATE INDEX IF NOT EXISTS`
  - Migration must be idempotent (safe to run multiple times)
  - _Requirements: 1.1, 2.1, 2.2, 3.1, 3.2, 25.1, 25.2, 25.3, 25.4_

- [x] 2. Add pgvector support to Cargo.toml and error variants
  - [x] 2.1 Add `pgvector` crate with `sqlx` feature to `[dependencies]` in `Cargo.toml`
    - Add `pgvector = { version = "0.4", features = ["sqlx"] }` (or latest compatible version)
    - _Requirements: 24.1_

  - [x] 2.2 Add new error variants to `src/error.rs`
    - Add `EmbeddingGenerationFailed(String)` — OpenAI embedding API error
    - Add `ChatGenerationFailed(String)` — OpenAI chat completion API error
    - Add `RateLimitExceeded` — HTTP 429 from OpenAI
    - Add `InvalidSimilarityThreshold(f32)` — threshold outside [0.0, 1.0]
    - Add `InvalidEmbeddingModel(String)` — unrecognized model name
    - Add `InvalidLlmModel(String)` — unrecognized LLM model name
    - Add `ItemNotFound(Uuid)` — item_id not found in vault
    - _Requirements: 21.1, 21.2, 21.3, 20.4, 18.4, 19.4_

- [x] 3. Add AI model types to `src/models.rs`
  - Add `AiConfig` struct with fields: `openai_api_key`, `embedding_model`, `llm_model`, `similarity_threshold: f32`
  - Implement `AiConfig::from_env()` reading `OPENAI_API_KEY`, `EMBEDDING_MODEL` (default `text-embedding-3-small`), `LLM_MODEL` (default `gpt-3.5-turbo`), `SIMILARITY_THRESHOLD` (default `0.8`)
  - Validate `SIMILARITY_THRESHOLD` is in [0.0, 1.0]; return `InvalidSimilarityThreshold` if not
  - Validate `EMBEDDING_MODEL` is one of `text-embedding-3-small`, `text-embedding-3-large`, `text-embedding-ada-002`; return `InvalidEmbeddingModel` if not
  - Validate `LLM_MODEL` is one of `gpt-4`, `gpt-4-turbo`, `gpt-3.5-turbo`; return `InvalidLlmModel` if not
  - Add `ChatRequest` struct (`message: String`) with `Deserialize`
  - Add `ChatResponse` struct (`chat_reply_text: String`, `referenced_node_ids: Vec<Uuid>`) with `Serialize`/`Deserialize`
  - _Requirements: 18.1, 18.2, 18.3, 18.4, 19.1, 19.2, 19.3, 19.4, 20.1, 20.2, 20.3, 20.4_

  - [ ]* 3.1 Write unit tests for `AiConfig::from_env`
    - Test default values when env vars are absent
    - Test valid overrides for each field
    - Test `InvalidSimilarityThreshold` for values < 0.0 and > 1.0
    - Test `InvalidEmbeddingModel` for an unrecognized model name
    - Test `InvalidLlmModel` for an unrecognized model name
    - _Requirements: 18.3, 18.4, 19.3, 19.4, 20.3, 20.4_

  - [ ]* 3.2 Write property test for `AiConfig` similarity threshold validation
    - **Property 10: Similarity threshold is a valid float in [0.0, 1.0]**
    - For any float value, if outside [0.0, 1.0] `from_env` returns error; if inside, loaded value equals configured value
    - **Validates: Requirements 20.2, 20.4**

- [x] 4. Implement `EmbeddingService` in `src/services/embedding.rs`
  - [x] 4.1 Create `src/services/embedding.rs` with `EmbeddingService` struct
    - Fields: `client: reqwest::Client`, `api_key: String`, `model: String`
    - Implement `EmbeddingService::from_env()` using `AiConfig::from_env()`
    - Implement `extract_content_text(item: &ContentItem) -> String` with priority: `transcript` → `metadata["scraped_text"]` → `title` → empty string (log warning on empty)
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 22.1, 22.4_

  - [x] 4.2 Implement `generate_embedding` method
    - POST to `https://api.openai.com/v1/embeddings` with `model` and single text input
    - Parse response and return `Vec<f32>` of length 1536
    - Apply 2-second timeout on the reqwest call
    - Map HTTP 429 → `RateLimitExceeded`; other API errors → `EmbeddingGenerationFailed`
    - _Requirements: 4.5, 12.1, 12.2, 12.3, 21.1, 21.3, 22.2_

  - [x] 4.3 Implement `generate_embeddings_batch` method
    - POST to `https://api.openai.com/v1/embeddings` with array input
    - Return `Vec<Vec<f32>>`; preserve input order
    - Apply same error mapping as `generate_embedding`
    - _Requirements: 22.3, 27.2_

  - [ ]* 4.4 Write unit tests for `extract_content_text`
    - Test transcript takes priority over scraped_text and title
    - Test scraped_text used when transcript is absent
    - Test title used as final fallback
    - Test empty string returned when all fields are absent
    - _Requirements: 4.1, 4.2, 4.3, 4.4_

  - [ ]* 4.5 Write property test for embedding dimensionality
    - **Property 9: Embedding round-trip preserves dimensionality**
    - For any non-empty text, `generate_embedding` returns a vector of exactly 1536 elements
    - (Use a mock/stub HTTP response returning a fixed 1536-dim vector)
    - **Validates: Requirements 4.5, 22.2**

- [x] 5. Add vector database functions to `src/db.rs`
  - [x] 5.1 Implement `upsert_embedding`
    - `pub async fn upsert_embedding(pool: &PgPool, item_id: Uuid, embedding: &[f32]) -> Result<()>`
    - Use `pgvector::Vector` to bind the `Vec<f32>` as a `vector` parameter
    - SQL: `UPDATE content_items SET content_embedding = $1 WHERE id = $2`
    - Use parameterized query to prevent SQL injection
    - _Requirements: 5.1, 5.3, 24.1_

  - [x] 5.2 Implement `find_similar_items`
    - `pub async fn find_similar_items(pool: &PgPool, vault_id: Uuid, query_vector: &[f32], limit: i64) -> Result<Vec<SimilarResult>>`
    - SQL: cosine similarity `1 - (content_embedding <=> $1)` ordered descending, filtered by `vault_id`, excluding NULL embeddings
    - Return `Vec<SimilarResult>` where `SimilarResult { item: ContentItem, similarity: f32 }`
    - Define `SimilarResult` struct in `src/db.rs`
    - _Requirements: 6.1, 6.2, 6.3, 6.5, 13.1, 13.2, 13.3, 17.1, 24.2_

  - [x] 5.3 Implement `find_similar_to_item`
    - `pub async fn find_similar_to_item(pool: &PgPool, vault_id: Uuid, item_id: Uuid, threshold: f32, limit: i64) -> Result<Vec<SimilarResult>>`
    - SQL: join on stored embedding of `item_id`, compute cosine similarity, filter by `vault_id` and `similarity >= threshold`, exclude `item_id` itself
    - _Requirements: 6.1, 6.4, 6.5, 17.1, 17.2, 24.3_

  - [x] 5.4 Implement `find_items_without_embeddings`
    - `pub async fn find_items_without_embeddings(pool: &PgPool, vault_id: Uuid) -> Result<Vec<Uuid>>`
    - SQL: `SELECT id FROM content_items WHERE vault_id = $1 AND content_embedding IS NULL`
    - _Requirements: 27.1_

  - [ ]* 5.5 Write unit tests for `SimilarResult` ordering
    - Test that results from `find_similar_items` are ordered by descending similarity
    - Test that the query item is excluded from `find_similar_to_item` results
    - Test that items with NULL embeddings are excluded
    - _Requirements: 6.3, 6.4, 17.1_

- [x] 6. Checkpoint — compile and verify DB layer
  - Ensure all tests pass, ask the user if questions arise.

- [x] 7. Update `SemanticGraphEngine` in `src/services/graph.rs`
  - [x] 7.1 Add `GraphFilter` struct and `SimilarityPair` struct
    - `GraphFilter { start_date: Option<DateTime<Utc>>, end_date: Option<DateTime<Utc>>, content_types: Vec<ContentType>, user_id: Option<Uuid>, similarity_threshold: f32 }`
    - Implement `Default` for `GraphFilter` (all `None`/empty, `similarity_threshold = 0.8`)
    - `SimilarityPair { item_a: Uuid, item_b: Uuid, similarity: f32 }`
    - _Requirements: 7.2, 8.1, 9.1, 10.1, 20.2_

  - [x] 7.2 Implement `build_graph_with_similarity`
    - `pub fn build_graph_with_similarity(items, tags, item_tags, similarity_pairs: Vec<SimilarityPair>, filter: &GraphFilter) -> Graph`
    - Apply `GraphFilter` to items before building nodes: filter by `start_date`/`end_date` on `created_at`, by `content_types` (skip if empty), by `user_id` (skip if None)
    - Build tag-based edges using existing logic (call or inline `build_graph` logic)
    - Add similarity edges for pairs where both items pass the filter and `similarity >= filter.similarity_threshold`; edge weight = similarity score; bidirectional
    - Preserve all tag-based edges alongside similarity edges
    - _Requirements: 7.1, 7.3, 7.4, 7.5, 8.2, 8.3, 8.4, 8.5, 9.2, 9.3, 9.4, 10.2, 10.3, 10.4, 28.1, 28.2, 28.3_

  - [ ]* 7.3 Write unit tests for `build_graph_with_similarity`
    - Test date range filter excludes items outside range
    - Test content type filter excludes non-matching types
    - Test user filter excludes items from other users
    - Test similarity edges are bidirectional with correct weight
    - Test tag-based edges are preserved when similarity edges are added
    - Test null filter fields include all items
    - _Requirements: 7.3, 7.4, 7.5, 8.2, 8.3, 9.2, 9.3, 10.2, 10.3_

  - [ ]* 7.4 Write property test for graph filter correctness
    - **Property 3: Graph filter preserves only matching items and their edges**
    - For any graph built with a `GraphFilter`, every node satisfies all active filter predicates, and every edge connects two nodes in the filtered set
    - **Validates: Requirements 8.2, 8.3, 9.2, 9.3, 10.2, 10.3**

  - [ ]* 7.5 Write property test for null filter includes all items
    - **Property 4: Null filter fields include all items**
    - For any `GraphFilter` with all optional fields `None` and `content_types` empty, the resulting graph contains the same nodes as an unfiltered graph
    - **Validates: Requirements 8.4, 8.5, 9.4, 10.4**

  - [ ]* 7.6 Write property test for similarity edge bidirectionality
    - **Property 5: Vector similarity edges are bidirectional**
    - For any pair (A, B) where similarity ≥ threshold, if edge A→B exists then edge B→A exists with the same weight
    - **Validates: Requirements 7.3, 7.4**

  - [ ]* 7.7 Write property test for tag edge preservation
    - **Property 6: Tag-based edges are preserved alongside similarity edges**
    - For any graph built with `build_graph_with_similarity`, every edge that `build_graph` would have created is still present
    - **Validates: Requirements 7.5, 28.1, 28.2, 28.3**

- [x] 8. Implement `RagChatService` in `src/services/rag_chat.rs`
  - [x] 8.1 Create `src/services/rag_chat.rs` with `RagChatService` struct
    - Fields: `embedding_service: Arc<EmbeddingService>`, `db: Arc<sqlx::PgPool>`, `llm_model: String`, `openai_api_key: String`, `client: reqwest::Client`, `top_k: usize` (default 5)
    - Implement `RagChatService::from_env(embedding_service, db)` reading `LLM_MODEL` and `OPENAI_API_KEY`
    - _Requirements: 23.1, 23.4_

  - [x] 8.2 Implement `process_message` method
    - `pub async fn process_message(&self, vault_id: Uuid, user_id: Uuid, message: &str) -> Result<ChatResponse>`
    - Step 1: call `embedding_service.generate_embedding(message)` → query vector
    - Step 2: call `Database::find_similar_items(pool, vault_id, &query_vector, top_k)` → `Vec<SimilarResult>`
    - Step 3: build RAG prompt using the template from the design (system message with context items, user message)
    - Step 4: POST to `https://api.openai.com/v1/chat/completions` with `llm_model`, system + user messages, `max_tokens: 1024`
    - Apply 10-second timeout on the chat completion call
    - Map HTTP 429 → `RateLimitExceeded`; other errors → `ChatGenerationFailed`
    - Return `ChatResponse { chat_reply_text, referenced_node_ids }` where `referenced_node_ids` is ordered by similarity (highest first)
    - _Requirements: 11.4, 13.1, 13.2, 13.3, 13.4, 13.5, 14.1, 14.2, 14.3, 14.4, 14.5, 15.1, 15.2, 15.3, 15.4, 21.2, 21.3, 23.2, 23.3_

  - [ ]* 8.3 Write unit tests for RAG prompt construction
    - Test that context items appear in the prompt string with their IDs and content
    - Test that the user message appears in the prompt
    - Test that empty context produces a valid (empty context section) prompt
    - _Requirements: 14.1, 14.3_

  - [ ]* 8.4 Write property test for referenced_node_ids vault membership
    - **Property 7: Chat response referenced_node_ids are a subset of vault items**
    - For any chat message, every UUID in `referenced_node_ids` corresponds to a `ContentItem` belonging to that vault
    - (Test via `find_similar_items` always scoping to `vault_id`)
    - **Validates: Requirements 13.3, 15.2**

- [x] 9. Add axum route handler for `POST /api/vaults/:vault_id/chat`
  - Add `chat_handler` async function in the appropriate handler module (or inline in `src/bin/cofre.rs` if that's where routes are defined)
  - Signature: `async fn chat_handler(State(app_state), Path(vault_id), Extension(user), Json(payload: ChatRequest)) -> Result<Json<ChatResponse>, AppError>`
  - Validate vault membership via existing `AuthService` / vault membership check; return `Unauthorized` if not a member
  - Delegate to `RagChatService::process_message(vault_id, user.id, &payload.message)`
  - Map `Error::RateLimitExceeded` → HTTP 429, `Error::ChatGenerationFailed` → HTTP 502, `Error::Unauthorized` → HTTP 401
  - Register route in the axum router
  - _Requirements: 11.1, 11.2, 11.3, 11.4, 11.5_

  - [ ]* 9.1 Write unit tests for `chat_handler` authorization
    - Test that a non-member user receives HTTP 401
    - Test that a valid member receives a `ChatResponse` shaped response
    - _Requirements: 11.3, 11.5_

- [x] 10. Wire everything up in `src/services/mod.rs` and the main binary
  - [x] 10.1 Export new modules from `src/services/mod.rs`
    - Add `pub mod embedding;` and `pub mod rag_chat;`
    - Add `pub use embedding::EmbeddingService;` and `pub use rag_chat::RagChatService;`
    - _Requirements: 22.1, 23.1_

  - [x] 10.2 Integrate `EmbeddingService` into content creation flow
    - In `ContentService::create_item` (or equivalent), after creating the `ContentItem`, call `EmbeddingService::generate_embedding` with `extract_content_text(&item)`
    - Call `Database::upsert_embedding` to store the vector
    - On embedding failure: log warning, continue (item is created with NULL embedding)
    - _Requirements: 4.1, 4.2, 4.3, 5.1, 5.2, 17.3, 17.4_

  - [x] 10.3 Add `EmbeddingService` and `RagChatService` to `AppState` in the main binary
    - Construct `EmbeddingService::from_env()` and `RagChatService::from_env(...)` at startup
    - Add both to the axum `AppState` struct so handlers can access them via `State`
    - Fail fast at startup if `AiConfig::from_env()` returns an error
    - _Requirements: 1.3, 18.4, 19.4, 20.4_

- [x] 11. Implement incremental embedding backfill utility
  - Add a function (e.g., `backfill_embeddings` in `src/services/embedding.rs` or a dedicated module) that:
    - Calls `Database::find_items_without_embeddings` to get item IDs
    - Fetches each item, calls `extract_content_text`, generates embedding via `generate_embeddings_batch`
    - Calls `Database::upsert_embedding` for each result
    - Logs progress (items processed, errors)
  - _Requirements: 27.1, 27.2, 27.3, 27.4_

- [x] 12. Final checkpoint — ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for a faster MVP
- Each task references specific requirements for traceability
- Property tests map directly to the Correctness Properties in the design document
- The `pgvector` crate bridges `Vec<f32>` ↔ PostgreSQL `vector` type via sqlx
- Embedding failures during content creation are non-fatal: item is saved with NULL embedding and excluded from similarity search (Requirement 17)
- The `build_graph` method is preserved unchanged; `build_graph_with_similarity` is additive
