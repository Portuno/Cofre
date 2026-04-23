# Design Document: Web Transformation to Vercel + Supabase

## Overview

This document specifies the technical design for transforming the Cofre Vault Platform from a Rust backend/CLI application into a modern web application deployable on Vercel with Supabase as the backend database.

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend (Vercel)                        │
│                    React/Next.js SPA or SSR                      │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Pages: Vaults, Content, Chat, Settings                 │   │
│  │  Components: VaultList, ContentGrid, ChatInterface      │   │
│  │  State Management: React Context or Redux               │   │
│  │  HTTP Client: Axios or Fetch with JWT auth              │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              ↓ REST API
┌─────────────────────────────────────────────────────────────────┐
│                    Backend API (Vercel Functions)                │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  /api/auth/*          - Authentication endpoints         │   │
│  │  /api/vaults/*        - Vault management                 │   │
│  │  /api/content/*       - Content operations               │   │
│  │  /api/tags/*          - Tag management                   │   │
│  │  /api/chat            - RAG chat interface               │   │
│  │  /api/graph/*         - Semantic graph queries           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Services Layer:                                         │   │
│  │  - AuthService (JWT validation, user management)         │   │
│  │  - VaultService (vault CRUD, collaboration)              │   │
│  │  - ContentService (content storage, retrieval)           │   │
│  │  - TagService (tag management)                           │   │
│  │  - AudioService (transcription via ElevenLabs)           │   │
│  │  - EmbeddingService (vector generation via Gemini)       │   │
│  │  - GraphService (semantic graph construction)            │   │
│  │  - RagChatService (context retrieval + LLM)              │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Infrastructure:                                         │   │
│  │  - Database connection pooling                           │   │
│  │  - Request validation & error handling                   │   │
│  │  - Logging & monitoring                                  │   │
│  │  - Rate limiting & caching                               │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              ↓ SQL
┌─────────────────────────────────────────────────────────────────┐
│                    Supabase PostgreSQL                           │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Tables:                                                 │   │
│  │  - users (Supabase Auth)                                 │   │
│  │  - vaults                                                │   │
│  │  - vault_members                                         │   │
│  │  - vault_invites                                         │   │
│  │  - content_items                                         │   │
│  │  - embeddings (pgvector)                                 │   │
│  │  - tags                                                  │   │
│  │  - item_tags                                             │   │
│  │  - migrations (tracking)                                 │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Extensions:                                             │   │
│  │  - pgvector (vector similarity search)                   │   │
│  │  - uuid-ossp (UUID generation)                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Storage:                                                │   │
│  │  - Supabase Storage buckets for audio/image files        │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    External Services                             │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  ElevenLabs API - Audio transcription                    │   │
│  │  Gemini API - Embeddings & LLM responses                 │   │
│  │  Supabase Auth - User authentication                     │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Technology Stack

### Frontend
- **Framework**: Next.js 14+ (React 18+) or similar SPA framework
- **Styling**: Tailwind CSS or similar
- **State Management**: React Context API or Redux
- **HTTP Client**: Axios or Fetch API
- **Authentication**: Supabase Auth client library
- **Deployment**: Vercel

### Backend API
- **Runtime**: Node.js (Express, Fastify, or similar) or Python (FastAPI, Flask)
- **Language**: TypeScript or Python
- **Database Driver**: node-postgres (pg) or sqlalchemy
- **Vector Operations**: pgvector client library
- **External APIs**: axios or httpx for ElevenLabs and Gemini
- **Deployment**: Vercel Serverless Functions

### Database
- **Provider**: Supabase (managed PostgreSQL)
- **Extensions**: pgvector, uuid-ossp
- **Connection Pooling**: PgBouncer (Supabase built-in)
- **Migrations**: Supabase migrations or Flyway

### External Services
- **Authentication**: Supabase Auth (JWT-based)
- **Audio Transcription**: ElevenLabs API
- **Embeddings & LLM**: Google Gemini API
- **File Storage**: Supabase Storage

## Database Schema

### Core Tables

#### users
```sql
CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email VARCHAR(255) UNIQUE NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```
*Note: User authentication is managed by Supabase Auth. This table stores minimal user metadata.*

#### vaults
```sql
CREATE TABLE vaults (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name VARCHAR(255) NOT NULL,
  description TEXT,
  created_by UUID NOT NULL REFERENCES users(id),
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

#### vault_members
```sql
CREATE TABLE vault_members (
  vault_id UUID NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  role VARCHAR(50) NOT NULL CHECK (role IN ('owner', 'member')),
  joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (vault_id, user_id)
);
```

#### vault_invites
```sql
CREATE TABLE vault_invites (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  vault_id UUID NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
  invited_email VARCHAR(255) NOT NULL,
  token VARCHAR(255) UNIQUE NOT NULL,
  accepted BOOLEAN DEFAULT FALSE,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  expires_at TIMESTAMP NOT NULL
);
```

#### content_items
```sql
CREATE TABLE content_items (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  vault_id UUID NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
  created_by UUID NOT NULL REFERENCES users(id),
  content_type VARCHAR(50) NOT NULL CHECK (content_type IN ('audio', 'image', 'link')),
  title VARCHAR(255),
  url TEXT NOT NULL,
  transcript TEXT,
  metadata JSONB,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

#### embeddings
```sql
CREATE TABLE embeddings (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  content_item_id UUID NOT NULL REFERENCES content_items(id) ON DELETE CASCADE,
  embedding vector(768),
  model VARCHAR(100) NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

#### tags
```sql
CREATE TABLE tags (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  vault_id UUID NOT NULL REFERENCES vaults(id) ON DELETE CASCADE,
  name VARCHAR(255) NOT NULL,
  is_special BOOLEAN DEFAULT FALSE,
  color VARCHAR(7),
  created_by UUID NOT NULL REFERENCES users(id),
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(vault_id, name)
);
```

#### item_tags
```sql
CREATE TABLE item_tags (
  item_id UUID NOT NULL REFERENCES content_items(id) ON DELETE CASCADE,
  tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (item_id, tag_id)
);
```

#### migrations
```sql
CREATE TABLE migrations (
  id SERIAL PRIMARY KEY,
  name VARCHAR(255) UNIQUE NOT NULL,
  applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Indexes

```sql
CREATE INDEX idx_vaults_created_by ON vaults(created_by);
CREATE INDEX idx_vault_members_user_id ON vault_members(user_id);
CREATE INDEX idx_vault_members_vault_id ON vault_members(vault_id);
CREATE INDEX idx_content_items_vault_id ON content_items(vault_id);
CREATE INDEX idx_content_items_created_by ON content_items(created_by);
CREATE INDEX idx_embeddings_content_item_id ON embeddings(content_item_id);
CREATE INDEX idx_tags_vault_id ON tags(vault_id);
CREATE INDEX idx_item_tags_tag_id ON item_tags(tag_id);
CREATE INDEX idx_embeddings_vector ON embeddings USING ivfflat (embedding vector_cosine_ops);
```

## API Endpoints

### Authentication

```
POST /api/auth/signup
  Request: { email: string, password: string }
  Response: { user: User, session_token: string }
  Status: 201 Created

POST /api/auth/signin
  Request: { email: string, password: string }
  Response: { user: User, session_token: string }
  Status: 200 OK

POST /api/auth/signout
  Headers: Authorization: Bearer <token>
  Response: { success: boolean }
  Status: 200 OK

GET /api/auth/me
  Headers: Authorization: Bearer <token>
  Response: { user: User }
  Status: 200 OK
```

### Vaults

```
POST /api/vaults
  Headers: Authorization: Bearer <token>
  Request: { name: string, description?: string }
  Response: { vault: Vault }
  Status: 201 Created

GET /api/vaults
  Headers: Authorization: Bearer <token>
  Response: { vaults: Array<{ vault: Vault, role: MemberRole }> }
  Status: 200 OK

GET /api/vaults/:vault_id
  Headers: Authorization: Bearer <token>
  Response: { vault: Vault }
  Status: 200 OK

PUT /api/vaults/:vault_id
  Headers: Authorization: Bearer <token>
  Request: { name?: string, description?: string }
  Response: { vault: Vault }
  Status: 200 OK

DELETE /api/vaults/:vault_id
  Headers: Authorization: Bearer <token>
  Response: { success: boolean }
  Status: 200 OK

POST /api/vaults/:vault_id/members
  Headers: Authorization: Bearer <token>
  Request: { email: string }
  Response: { invite: VaultInvite }
  Status: 201 Created

GET /api/vaults/:vault_id/members
  Headers: Authorization: Bearer <token>
  Response: { members: Array<VaultMember> }
  Status: 200 OK

DELETE /api/vaults/:vault_id/members/:user_id
  Headers: Authorization: Bearer <token>
  Response: { success: boolean }
  Status: 200 OK

POST /api/vaults/invites/:token/accept
  Headers: Authorization: Bearer <token>
  Response: { vault: Vault }
  Status: 200 OK
```

### Content

```
POST /api/vaults/:vault_id/content
  Headers: Authorization: Bearer <token>
  Request: FormData with file and metadata
  Response: { content_item: ContentItem }
  Status: 201 Created

GET /api/vaults/:vault_id/content
  Headers: Authorization: Bearer <token>
  Query: ?limit=50&offset=0&tag_id=<uuid>&type=audio
  Response: { items: Array<ContentItem>, total: number }
  Status: 200 OK

GET /api/vaults/:vault_id/content/:item_id
  Headers: Authorization: Bearer <token>
  Response: { content_item: ContentItem }
  Status: 200 OK

PUT /api/vaults/:vault_id/content/:item_id
  Headers: Authorization: Bearer <token>
  Request: { title?: string, transcript?: string, metadata?: object }
  Response: { content_item: ContentItem }
  Status: 200 OK

DELETE /api/vaults/:vault_id/content/:item_id
  Headers: Authorization: Bearer <token>
  Response: { success: boolean }
  Status: 200 OK

POST /api/vaults/:vault_id/content/:item_id/tags
  Headers: Authorization: Bearer <token>
  Request: { tag_ids: Array<UUID> }
  Response: { content_item: ContentItem }
  Status: 200 OK
```

### Tags

```
POST /api/vaults/:vault_id/tags
  Headers: Authorization: Bearer <token>
  Request: { name: string, is_special?: boolean, color?: string }
  Response: { tag: Tag }
  Status: 201 Created

GET /api/vaults/:vault_id/tags
  Headers: Authorization: Bearer <token>
  Response: { tags: Array<Tag> }
  Status: 200 OK

PUT /api/vaults/:vault_id/tags/:tag_id
  Headers: Authorization: Bearer <token>
  Request: { name?: string, color?: string }
  Response: { tag: Tag }
  Status: 200 OK

DELETE /api/vaults/:vault_id/tags/:tag_id
  Headers: Authorization: Bearer <token>
  Response: { success: boolean }
  Status: 200 OK
```

### Chat

```
POST /api/vaults/:vault_id/chat
  Headers: Authorization: Bearer <token>
  Request: { message: string }
  Response: { 
    chat_reply_text: string,
    referenced_node_ids: Array<UUID>
  }
  Status: 200 OK
```

### Graph

```
GET /api/vaults/:vault_id/graph
  Headers: Authorization: Bearer <token>
  Query: ?tag_id=<uuid>&content_type=audio
  Response: {
    nodes: Array<{
      item: ContentItem,
      edges: Array<{ target_item_id: UUID, shared_tag: Tag, weight: number }>
    }>,
    edge_count: number
  }
  Status: 200 OK
```

## Service Layer Design

### AuthService
- Validates credentials against Supabase Auth
- Issues JWT tokens
- Validates JWT tokens on each request
- Manages user sessions

### VaultService
- CRUD operations for vaults
- Manages vault members and roles
- Handles vault invitations
- Enforces access control

### ContentService
- CRUD operations for content items
- Manages file uploads to Supabase Storage
- Handles content tagging
- Supports filtering and pagination

### TagService
- CRUD operations for tags
- Manages tag-content relationships
- Supports tag-based filtering

### AudioService
- Handles audio file uploads
- Calls ElevenLabs API for transcription
- Stores transcripts in database
- Handles transcription errors and retries

### EmbeddingService
- Generates embeddings using Gemini API
- Stores embeddings in pgvector
- Supports similarity search
- Implements caching for unchanged content

### GraphService
- Reconstructs semantic graph from database
- Identifies edges based on shared tags and embedding similarity
- Supports graph filtering
- Implements caching for performance

### RagChatService
- Retrieves relevant content using semantic search
- Constructs context window from top-K items
- Calls Gemini API for response generation
- Tracks referenced content items

## Error Handling Strategy

### Error Categories

1. **Validation Errors** (400 Bad Request)
   - Invalid input data
   - Missing required fields
   - Invalid data types

2. **Authentication Errors** (401 Unauthorized)
   - Invalid or expired JWT token
   - Missing authentication header

3. **Authorization Errors** (403 Forbidden)
   - User lacks permissions for operation
   - User is not a vault member

4. **Not Found Errors** (404 Not Found)
   - Resource does not exist
   - Vault, content item, or tag not found

5. **Server Errors** (500 Internal Server Error)
   - Database connection failures
   - External API failures
   - Unexpected exceptions

### Error Response Format

```json
{
  "error": {
    "code": "INVALID_INPUT",
    "message": "Email is required",
    "details": {
      "field": "email"
    }
  }
}
```

### Retry Strategy

- **Database failures**: Exponential backoff (100ms, 200ms, 400ms, 800ms, 1600ms)
- **External API failures**: Exponential backoff with jitter
- **Max retries**: 3 attempts
- **Circuit breaker**: Open after 5 consecutive failures, half-open after 30 seconds

## Caching Strategy

### Cache Layers

1. **Application-level caching** (in-memory)
   - Vault metadata (TTL: 5 minutes)
   - Tag lists (TTL: 5 minutes)
   - User permissions (TTL: 10 minutes)

2. **Database query caching** (Redis or Supabase caching)
   - Semantic graph (TTL: 10 minutes)
   - Content search results (TTL: 5 minutes)

3. **Embedding caching**
   - Cache embeddings to avoid regeneration
   - Invalidate on content update

### Cache Invalidation

- Vault updates: Invalidate vault cache
- Content updates: Invalidate graph and search caches
- Tag updates: Invalidate tag and graph caches
- Member changes: Invalidate permission cache

## Security Design

### Authentication
- JWT tokens issued by Supabase Auth
- Tokens include user ID and email
- Tokens expire after 1 hour (configurable)
- Refresh tokens for long-lived sessions

### Authorization
- Role-based access control (Owner, Member)
- Vault membership verification on each request
- Content ownership verification for modifications

### Input Validation
- All user input validated before processing
- Parameterized queries to prevent SQL injection
- File upload validation (size, type, content)

### Rate Limiting
- Authentication endpoints: 5 requests per minute per IP
- API endpoints: 100 requests per minute per user
- External API calls: Respect service quotas

### Data Protection
- HTTPS for all communication
- Sensitive data encrypted at rest (Supabase built-in)
- Error messages sanitized to avoid information disclosure

## Deployment Configuration

### Vercel Configuration (vercel.json)

```json
{
  "buildCommand": "npm run build",
  "outputDirectory": ".next",
  "env": {
    "DATABASE_URL": "@database_url",
    "SUPABASE_URL": "@supabase_url",
    "SUPABASE_KEY": "@supabase_key",
    "GEMINI_API_KEY": "@gemini_api_key",
    "ELEVENLABS_API_KEY": "@elevenlabs_api_key"
  },
  "functions": {
    "api/**/*.ts": {
      "memory": 1024,
      "maxDuration": 60
    }
  }
}
```

### Environment Variables

**Required:**
- `DATABASE_URL`: PostgreSQL connection string (Supabase)
- `SUPABASE_URL`: Supabase project URL
- `SUPABASE_KEY`: Supabase anonymous key
- `GEMINI_API_KEY`: Google Gemini API key
- `ELEVENLABS_API_KEY`: ElevenLabs API key

**Optional:**
- `EMBEDDING_MODEL`: Gemini embedding model (default: text-embedding-004)
- `LLM_MODEL`: Gemini LLM model (default: gemini-1.5-flash)
- `SIMILARITY_THRESHOLD`: Embedding similarity threshold (default: 0.8)
- `RUST_LOG`: Logging level (default: info)

## Performance Considerations

### Database Optimization
- Connection pooling with PgBouncer
- Indexes on frequently queried columns
- Query optimization to avoid N+1 queries
- Pagination for large result sets

### API Optimization
- Response compression (gzip)
- Caching headers for static content
- Lazy loading for large datasets
- Batch operations where possible

### External API Optimization
- Rate limiting to respect quotas
- Caching to avoid redundant calls
- Async processing for long-running operations
- Circuit breakers to prevent cascading failures

## Monitoring and Observability

### Logging
- Structured logging (JSON format)
- Log levels: DEBUG, INFO, WARN, ERROR
- Request ID for tracing
- Performance metrics (response time, query time)

### Metrics
- Request count by endpoint
- Error rate by endpoint
- Response time percentiles (p50, p95, p99)
- Database connection pool status
- External API call success rate

### Alerting
- Error rate > 5%
- Response time p95 > 1 second
- Database connection pool exhaustion
- External API failures

## Testing Strategy

### Unit Tests
- Service layer logic
- Error handling
- Input validation
- Business logic correctness

### Integration Tests
- API endpoint contracts
- Database operations
- External API interactions (mocked)
- Authentication and authorization

### Property-Based Tests
- Embedding generation and storage (round-trip property)
- Semantic graph construction (invariants)
- Error handling and recovery (fault injection)
- Authorization enforcement (access control properties)

### Performance Tests
- Response time under load
- Database query performance
- Caching effectiveness
- Connection pool behavior

## Migration Path

### Phase 1: Backend API Development
1. Set up Vercel project structure
2. Implement database schema and migrations
3. Implement service layer
4. Implement API endpoints
5. Implement authentication and authorization

### Phase 2: Frontend Development
1. Set up Next.js project
2. Implement authentication UI
3. Implement vault management UI
4. Implement content management UI
5. Implement chat interface

### Phase 3: Integration and Testing
1. Integration testing
2. Performance testing
3. Security testing
4. User acceptance testing

### Phase 4: Deployment
1. Deploy backend to Vercel
2. Deploy frontend to Vercel
3. Configure environment variables
4. Run database migrations
5. Monitor and optimize

## Correctness Properties

### Property 1: Authentication Invariant
**Property**: For all valid JWT tokens, the decoded user ID matches the user making the request.
**Test**: Generate valid tokens, decode them, verify user ID consistency.
**Rationale**: Ensures authentication cannot be spoofed.

### Property 2: Authorization Invariant
**Property**: A user can only access vaults where they are a member.
**Test**: For each vault operation, verify user is in vault_members table.
**Rationale**: Prevents unauthorized access to vault data.

### Property 3: Embedding Round-Trip
**Property**: For all content items, storing and retrieving embeddings produces equivalent vectors.
**Test**: Store embedding → retrieve → compare vectors (within floating-point tolerance).
**Rationale**: Ensures embeddings are correctly persisted and retrieved.

### Property 4: Graph Invariant
**Property**: All edges in the semantic graph connect content items that share at least one tag.
**Test**: For each edge, verify both items have the shared tag.
**Rationale**: Ensures graph structure is semantically correct.

### Property 5: Pagination Invariant
**Property**: Paginating through all results returns all items exactly once.
**Test**: Paginate through results with various page sizes, verify no duplicates and no missing items.
**Rationale**: Ensures pagination doesn't lose or duplicate data.

### Property 6: Error Recovery
**Property**: After a transient failure, retrying the operation succeeds.
**Test**: Inject transient failures, verify retry succeeds.
**Rationale**: Ensures resilience to temporary failures.

### Property 7: Cache Consistency
**Property**: Cached data matches database data (within TTL).
**Test**: Compare cached values with database queries.
**Rationale**: Ensures cache doesn't serve stale data.

### Property 8: Role-Based Access Control
**Property**: Only owners can perform owner-level operations on vaults.
**Test**: Attempt owner operations with member role, verify rejection.
**Rationale**: Ensures role-based permissions are enforced.

