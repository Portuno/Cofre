-- Enable pgvector extension for vector similarity search
CREATE EXTENSION IF NOT EXISTS vector;

-- Verify pgvector is available
SELECT extname FROM pg_extension WHERE extname = 'vector';
