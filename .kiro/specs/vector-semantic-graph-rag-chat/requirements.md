# Requirements Document: Vector Semantic Graph RAG Chat

## Introduction

This feature improves the Cofre vault application's semantic graph and RAG chat interface:

1. **Graph Enhancement**: Currently the graph is built from content titles only. It needs to be built from the full content (transcripts, scraped text) to create more meaningful semantic relationships.

2. **Interactive RAG Chat**: Add a chat interface integrated with the graph visualization that highlights referenced nodes when the AI responds.

3. **UI/UX Improvements**: Modernize the visual design with better typography, colors, spacing, and interactive elements for both the graph and chat interface.

## Glossary

- **System**: The Cofre vault backend application (Rust)
- **Database**: The Supabase PostgreSQL database with pgvector extension
- **ContentItem**: A discrete piece of content (audio, image, or link) stored in a vault
- **Embedding**: A high-dimensional vector representation of content text
- **Similarity_Threshold**: A configurable cosine similarity value (0.0 to 1.0) used to determine graph edges
- **RAG_Engine**: The Retrieval-Augmented Generation component that combines similarity search with LLM responses
- **Chat_Panel**: The frontend interface for semantic chat interactions
- **Graph_Visualizer**: The frontend component that displays the semantic graph
- **OpenAI_API**: External service for generating embeddings and LLM responses
- **User**: A vault member interacting with the system

## Requirements

### Requirement 1: Enable pgvector Extension

**User Story:** As a system administrator, I want the pgvector extension enabled in the database, so that I can store and query vector embeddings efficiently.

#### Acceptance Criteria

1. THE System SHALL provide a SQL migration script that enables the pgvector extension
2. WHEN the migration is executed, THE Database SHALL have the pgvector extension available
3. THE System SHALL verify pgvector extension availability during initialization

### Requirement 2: Add Embedding Column to ContentItem Table

**User Story:** As a developer, I want a content_embedding column in the ContentItem table, so that I can store vector representations of content.

#### Acceptance Criteria

1. THE System SHALL provide a SQL migration that adds a content_embedding column of type vector(1536) to the content_items table
2. THE content_embedding column SHALL allow NULL values for backward compatibility
3. WHEN a ContentItem is created without an embedding, THE content_embedding column SHALL be NULL

### Requirement 3: Create Vector Index for Similarity Search

**User Story:** As a developer, I want a vector index on the content_embedding column, so that similarity searches execute efficiently.

#### Acceptance Criteria

1. THE System SHALL provide a SQL migration that creates an HNSW or IVFFlat index on the content_embedding column
2. THE index SHALL support cosine distance operations
3. WHEN a similarity search is performed, THE Database SHALL use the vector index for query optimization

### Requirement 4: Generate Embeddings from Rich Content

**User Story:** As a user, I want embeddings generated from full content text (transcripts, scraped text), so that semantic relationships are based on actual content rather than just titles.

#### Acceptance Criteria

1. WHEN a ContentItem with type Audio is created, THE System SHALL generate an embedding from the transcript text
2. WHEN a ContentItem with type Link is created, THE System SHALL generate an embedding from the scraped page text
3. WHEN a ContentItem with type Image is created, THE System SHALL generate an embedding from the title and metadata
4. IF content text is empty or unavailable, THEN THE System SHALL generate an embedding from the title field
5. THE System SHALL use OpenAI API (text-embedding-3-small or text-embedding-ada-002) to generate embeddings

### Requirement 5: Store Embeddings in Database

**User Story:** As a developer, I want embeddings automatically stored when content is created or updated, so that they are available for similarity search.

#### Acceptance Criteria

1. WHEN a ContentItem is created, THE System SHALL generate and store the embedding in the content_embedding column
2. WHEN a ContentItem's transcript or content is updated, THE System SHALL regenerate and update the embedding
3. THE System SHALL persist embeddings atomically with the ContentItem record

### Requirement 6: Query Similar Items by Cosine Similarity

**User Story:** As a developer, I want to query similar ContentItems using cosine similarity, so that I can build dynamic graph edges.

#### Acceptance Criteria

1. THE System SHALL provide a function that accepts an item_id and Similarity_Threshold and returns similar ContentItems
2. THE function SHALL use cosine distance (1 - cosine_similarity) for similarity calculation
3. THE function SHALL return items ordered by similarity score (highest first)
4. THE function SHALL exclude the query item from results
5. THE function SHALL only return items within the same vault as the query item

### Requirement 7: Build Graph Edges from Vector Similarity

**User Story:** As a user, I want graph edges created based on content similarity, so that I can discover semantically related items without manual tagging.

#### Acceptance Criteria

1. WHEN the semantic graph is built, THE System SHALL create edges between ContentItems with cosine similarity above the Similarity_Threshold
2. THE System SHALL support configurable Similarity_Threshold values (default 0.8)
3. THE System SHALL create bidirectional edges between similar items
4. THE System SHALL assign edge weights based on similarity scores
5. THE System SHALL preserve existing tag-based edges alongside similarity-based edges

### Requirement 8: Filter Graph by Date Range

**User Story:** As a user, I want to filter the semantic graph by date range, so that I can focus on content from specific time periods.

#### Acceptance Criteria

1. THE System SHALL provide a function that accepts start_date and end_date parameters
2. WHEN date filters are applied, THE System SHALL return only ContentItems created within the specified range
3. WHEN date filters are applied, THE System SHALL include only edges between filtered items
4. IF start_date is NULL, THEN THE System SHALL include all items from the beginning
5. IF end_date is NULL, THEN THE System SHALL include all items up to the present

### Requirement 9: Filter Graph by Content Type

**User Story:** As a user, I want to filter the semantic graph by content type, so that I can focus on specific types of content (audio, links, images).

#### Acceptance Criteria

1. THE System SHALL provide a function that accepts a list of ContentType values
2. WHEN content type filters are applied, THE System SHALL return only ContentItems matching the specified types
3. WHEN content type filters are applied, THE System SHALL include only edges between filtered items
4. IF the content type list is empty, THEN THE System SHALL return all content types

### Requirement 10: Filter Graph by User

**User Story:** As a user, I want to filter the semantic graph by creator, so that I can focus on content created by specific vault members.

#### Acceptance Criteria

1. THE System SHALL provide a function that accepts a user_id parameter
2. WHEN user filters are applied, THE System SHALL return only ContentItems created by the specified user
3. WHEN user filters are applied, THE System SHALL include only edges between filtered items
4. IF user_id is NULL, THEN THE System SHALL return items from all users

### Requirement 11: Create Semantic Chat API Endpoint

**User Story:** As a user, I want to send chat messages to a semantic assistant, so that I can query my vault content using natural language.

#### Acceptance Criteria

1. THE System SHALL provide a REST API endpoint that accepts chat messages
2. THE endpoint SHALL accept vault_id, user_id, and message_text as parameters
3. THE endpoint SHALL validate that the user is a member of the vault
4. THE endpoint SHALL return a JSON response containing chat_reply_text and referenced_node_ids
5. IF the user is not a vault member, THEN THE System SHALL return an Unauthorized error

### Requirement 12: Generate Embeddings for Chat Messages

**User Story:** As a developer, I want chat messages converted to embeddings, so that I can perform similarity search against vault content.

#### Acceptance Criteria

1. WHEN a chat message is received, THE RAG_Engine SHALL generate an embedding from the message text
2. THE RAG_Engine SHALL use the same embedding model as ContentItem embeddings
3. THE embedding generation SHALL complete within 2 seconds

### Requirement 13: Retrieve Relevant Content for RAG Context

**User Story:** As a developer, I want the most relevant ContentItems retrieved for each chat message, so that the LLM has appropriate context.

#### Acceptance Criteria

1. WHEN a chat message embedding is generated, THE RAG_Engine SHALL query the Database for the top 5 most similar ContentItems
2. THE RAG_Engine SHALL use cosine similarity for ranking
3. THE RAG_Engine SHALL only retrieve items from the specified vault
4. THE RAG_Engine SHALL include the full transcript or scraped text for each retrieved item
5. THE RAG_Engine SHALL return the item IDs of retrieved items as referenced_node_ids

### Requirement 14: Generate LLM Response with Retrieved Context

**User Story:** As a user, I want chat responses generated using my vault content as context, so that answers are grounded in my actual data.

#### Acceptance Criteria

1. WHEN relevant ContentItems are retrieved, THE RAG_Engine SHALL construct a prompt containing the user message and retrieved content
2. THE RAG_Engine SHALL send the prompt to OpenAI API (gpt-4 or gpt-3.5-turbo)
3. THE RAG_Engine SHALL include instructions to reference specific content items in the response
4. THE RAG_Engine SHALL return the LLM response as chat_reply_text
5. THE LLM response generation SHALL complete within 10 seconds

### Requirement 15: Return Referenced Node IDs in Chat Response

**User Story:** As a frontend developer, I want the chat response to include IDs of referenced ContentItems, so that I can highlight them in the graph visualization.

#### Acceptance Criteria

1. THE RAG_Engine SHALL return an array of referenced_node_ids in the chat response
2. THE referenced_node_ids array SHALL contain the IDs of all ContentItems used as context
3. THE referenced_node_ids array SHALL be ordered by relevance (highest similarity first)
4. IF no relevant items are found, THEN THE referenced_node_ids array SHALL be empty

### Requirement 16: Highlight Referenced Nodes in Graph Visualization

**User Story:** As a user, I want nodes referenced in chat responses to be highlighted in the graph, so that I can visually identify relevant content.

#### Acceptance Criteria

1. WHEN a chat response is received, THE Graph_Visualizer SHALL highlight all nodes in the referenced_node_ids array
2. THE Graph_Visualizer SHALL apply a visual effect (glow, color change, or border) to highlighted nodes
3. THE Graph_Visualizer SHALL maintain highlights until the next chat message is sent
4. THE Graph_Visualizer SHALL clear previous highlights when new highlights are applied

### Requirement 17: Handle Empty or Missing Content Gracefully

**User Story:** As a user, I want the system to handle items without embeddings gracefully, so that the chat feature works even with incomplete data.

#### Acceptance Criteria

1. WHEN a ContentItem has a NULL content_embedding, THE System SHALL exclude it from similarity searches
2. WHEN a ContentItem has a NULL content_embedding, THE System SHALL not create similarity-based edges for it
3. THE System SHALL continue to support tag-based edges for items without embeddings
4. THE System SHALL log a warning when attempting to generate an embedding fails

### Requirement 18: Provide Configuration for Embedding Model

**User Story:** As a system administrator, I want to configure the embedding model, so that I can choose between different OpenAI models or local alternatives.

#### Acceptance Criteria

1. THE System SHALL read the embedding model name from an environment variable (EMBEDDING_MODEL)
2. THE System SHALL support OpenAI models: text-embedding-3-small, text-embedding-3-large, text-embedding-ada-002
3. IF EMBEDDING_MODEL is not set, THEN THE System SHALL default to text-embedding-3-small
4. THE System SHALL validate the embedding model name during initialization

### Requirement 19: Provide Configuration for LLM Model

**User Story:** As a system administrator, I want to configure the LLM model for chat responses, so that I can balance cost and quality.

#### Acceptance Criteria

1. THE System SHALL read the LLM model name from an environment variable (LLM_MODEL)
2. THE System SHALL support OpenAI models: gpt-4, gpt-4-turbo, gpt-3.5-turbo
3. IF LLM_MODEL is not set, THEN THE System SHALL default to gpt-3.5-turbo
4. THE System SHALL validate the LLM model name during initialization

### Requirement 20: Provide Configuration for Similarity Threshold

**User Story:** As a system administrator, I want to configure the similarity threshold for graph edges, so that I can control graph density.

#### Acceptance Criteria

1. THE System SHALL read the Similarity_Threshold from an environment variable (SIMILARITY_THRESHOLD)
2. THE Similarity_Threshold SHALL be a float value between 0.0 and 1.0
3. IF SIMILARITY_THRESHOLD is not set, THEN THE System SHALL default to 0.8
4. THE System SHALL validate that the Similarity_Threshold is within the valid range

### Requirement 21: Handle OpenAI API Errors Gracefully

**User Story:** As a user, I want meaningful error messages when the OpenAI API fails, so that I understand what went wrong.

#### Acceptance Criteria

1. IF the OpenAI API returns an error during embedding generation, THEN THE System SHALL log the error and return an EmbeddingGenerationFailed error
2. IF the OpenAI API returns an error during chat response generation, THEN THE System SHALL log the error and return a ChatGenerationFailed error
3. IF the OpenAI API rate limit is exceeded, THEN THE System SHALL return a RateLimitExceeded error
4. THE System SHALL include the OpenAI error message in the error response for debugging

### Requirement 22: Implement Rust Struct for Embedding Operations

**User Story:** As a developer, I want a Rust struct that encapsulates embedding operations, so that I can reuse embedding logic across the application.

#### Acceptance Criteria

1. THE System SHALL provide an EmbeddingService struct with methods for generating embeddings
2. THE EmbeddingService SHALL have a generate_embedding method that accepts text and returns a vector
3. THE EmbeddingService SHALL have a generate_embeddings_batch method for bulk operations
4. THE EmbeddingService SHALL handle OpenAI API authentication using an API key from environment variables

### Requirement 23: Implement Rust Struct for RAG Chat Operations

**User Story:** As a developer, I want a Rust struct that encapsulates RAG chat operations, so that I can separate chat logic from API routing.

#### Acceptance Criteria

1. THE System SHALL provide a RagChatService struct with methods for handling chat requests
2. THE RagChatService SHALL have a process_message method that accepts a message and returns a chat response
3. THE RagChatService SHALL coordinate between embedding generation, similarity search, and LLM response generation
4. THE RagChatService SHALL depend on EmbeddingService and Database instances

### Requirement 24: Implement Database Access Layer for Vector Operations

**User Story:** As a developer, I want database functions for vector operations, so that I can query embeddings efficiently.

#### Acceptance Criteria

1. THE System SHALL provide a function to insert or update embeddings for a ContentItem
2. THE System SHALL provide a function to query similar items by embedding vector
3. THE System SHALL provide a function to query similar items by item_id
4. THE functions SHALL use parameterized queries to prevent SQL injection

### Requirement 25: Create SQL Migration for pgvector Setup

**User Story:** As a developer, I want a SQL migration file that sets up pgvector, so that I can apply the schema changes consistently.

#### Acceptance Criteria

1. THE System SHALL provide a migration file that enables the pgvector extension
2. THE migration file SHALL add the content_embedding column to content_items table
3. THE migration file SHALL create a vector index on content_embedding
4. THE migration file SHALL be idempotent (safe to run multiple times)

### Requirement 26: Document Frontend Integration Requirements

**User Story:** As a frontend developer, I want documentation on the chat API and response format, so that I can integrate the Chat_Panel with the backend.

#### Acceptance Criteria

1. THE System SHALL provide API documentation for the semantic chat endpoint
2. THE documentation SHALL include request and response schemas
3. THE documentation SHALL include example requests and responses
4. THE documentation SHALL describe the referenced_node_ids array format

### Requirement 27: Support Incremental Embedding Generation

**User Story:** As a system administrator, I want to generate embeddings for existing ContentItems incrementally, so that I can upgrade without downtime.

#### Acceptance Criteria

1. THE System SHALL provide a function to identify ContentItems without embeddings
2. THE System SHALL provide a function to generate embeddings for a batch of items
3. THE System SHALL support rate limiting to avoid exceeding OpenAI API limits
4. THE System SHALL log progress during batch embedding generation

### Requirement 28: Preserve Tag-Based Graph Functionality

**User Story:** As a user, I want existing tag-based graph features to continue working, so that I can use both tagging and vector similarity.

#### Acceptance Criteria

1. THE System SHALL continue to support tag-based edge creation
2. THE System SHALL continue to support special tag weighting (weight 2.0)
3. THE System SHALL merge tag-based edges and similarity-based edges in the graph
4. THE System SHALL allow filtering by tags alongside vector similarity

### Requirement 29: Implement Chat History Storage (Optional)

**User Story:** As a user, I want my chat history saved, so that I can review previous conversations.

#### Acceptance Criteria

1. WHERE chat history is enabled, THE System SHALL store chat messages and responses in the database
2. WHERE chat history is enabled, THE System SHALL associate chat messages with the vault and user
3. WHERE chat history is enabled, THE System SHALL provide an API to retrieve chat history
4. WHERE chat history is enabled, THE System SHALL include timestamps for each message

### Requirement 30: Implement Embedding Caching (Optional)

**User Story:** As a system administrator, I want embeddings cached to reduce API costs, so that repeated queries are faster and cheaper.

#### Acceptance Criteria

1. WHERE embedding caching is enabled, THE System SHALL cache generated embeddings in memory
2. WHERE embedding caching is enabled, THE System SHALL use cached embeddings for identical text inputs
3. WHERE embedding caching is enabled, THE System SHALL provide cache statistics (hit rate, size)
4. WHERE embedding caching is enabled, THE System SHALL support cache eviction policies (LRU)
