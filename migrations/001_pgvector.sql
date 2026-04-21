-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Add embedding column (nullable for backward compatibility)
-- Using 768 dimensions for Gemini text-embedding-004
ALTER TABLE content_items
    ADD COLUMN IF NOT EXISTS content_embedding vector(768);

-- HNSW index for cosine distance (fast approximate nearest-neighbor)
CREATE INDEX IF NOT EXISTS content_items_embedding_hnsw_idx
    ON content_items
    USING hnsw (content_embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);
