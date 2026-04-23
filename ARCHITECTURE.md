# Architecture Documentation

## System Overview

The Cofre Vault Platform is a modern web application built with a serverless architecture, designed for scalability and reliability.

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend (Vercel)                        │
│                    React/Next.js SPA or SSR                      │
└─────────────────────────────────────────────────────────────────┘
                              ↓ REST API
┌─────────────────────────────────────────────────────────────────┐
│                    Backend API (Vercel Functions)                │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Routes Layer                                            │   │
│  │  - /api/auth/*                                           │   │
│  │  - /api/vaults/*                                         │   │
│  │  - /api/content/*                                        │   │
│  │  - /api/tags/*                                           │   │
│  │  - /api/chat                                             │   │
│  │  - /api/graph/*                                          │   │
│  └──────────────────────────────────────────────────────────┘   │
│                              ↓                                    │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Services Layer                                          │   │
│  │  - AuthService                                           │   │
│  │  - VaultService                                          │   │
│  │  - ContentService                                        │   │
│  │  - TagService                                            │   │
│  │  - AudioService                                          │   │
│  │  - EmbeddingService                                      │   │
│  │  - GraphService                                          │   │
│  │  - RagChatService                                        │   │
│  └──────────────────────────────────────────────────────────┘   │
│                              ↓                                    │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Data Access Layer                                       │   │
│  │  - Database Connection Pool                              │   │
│  │  - Query Execution                                       │   │
│  │  - Migration Runner                                      │   │
│  └──────────────────────────────────────────────────────────┘   │
│                              ↓                                    │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Infrastructure                                          │   │
│  │  - Error Handling                                        │   │
│  │  - Logging                                               │   │
│  │  - Caching                                               │   │
│  │  - Rate Limiting                                         │   │
│  │  - Authentication Middleware                             │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    Supabase PostgreSQL                           │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Core Tables                                             │   │
│  │  - users                                                 │   │
│  │  - vaults                                                │   │
│  │  - vault_members                                         │   │
│  │  - vault_invites                                         │   │
│  │  - content_items                                         │   │
│  │  - embeddings (pgvector)                                 │   │
│  │  - tags                                                  │   │
│  │  - item_tags                                             │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    External Services                             │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  - Supabase Auth (JWT)                                   │   │
│  │  - Supabase Storage (Files)                              │   │
│  │  - Google Gemini API (Embeddings & LLM)                  │   │
│  │  - ElevenLabs API (Transcription)                        │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Directory Structure

```
api/
├── src/
│   ├── config.ts                 # Configuration management
│   ├── logger.ts                 # Logging setup
│   ├── constants.ts              # Application constants
│   ├── index.ts                  # Express app entry point
│   ├── types/
│   │   └── index.ts              # TypeScript type definitions
│   ├── db/
│   │   ├── pool.ts               # Database connection pooling
│   │   ├── migrate.ts            # Migration runner
│   │   └── index.ts              # Database module exports
│   ├── middleware/
│   │   ├── auth.ts               # Authentication middleware
│   │   └── errorHandler.ts       # Error handling middleware
│   ├── services/
│   │   ├── AuthService.ts        # Authentication logic
│   │   ├── VaultService.ts       # Vault management
│   │   ├── ContentService.ts     # Content management
│   │   ├── TagService.ts         # Tag management
│   │   ├── AudioService.ts       # Audio processing
│   │   ├── EmbeddingService.ts   # Embedding generation
│   │   ├── GraphService.ts       # Semantic graph
│   │   ├── RagChatService.ts     # RAG chat
│   │   ├── __tests__/            # Service tests
│   │   └── index.ts              # Services module exports
│   ├── routes/
│   │   ├── auth.ts               # Auth endpoints
│   │   ├── vaults.ts             # Vault endpoints
│   │   ├── content.ts            # Content endpoints
│   │   ├── tags.ts               # Tag endpoints
│   │   ├── chat.ts               # Chat endpoints
│   │   ├── graph.ts              # Graph endpoints
│   │   └── index.ts              # Routes module exports
│   └── utils/
│       ├── cache.ts              # Caching utilities
│       ├── validation.ts         # Input validation
│       └── retry.ts              # Retry and circuit breaker
├── supabase/
│   └── migrations/
│       ├── 001_initial_schema.sql
│       ├── 002_pgvector_extension.sql
│       ├── 003_indexes.sql
│       └── 004_constraints.sql
├── package.json
├── tsconfig.json
├── vercel.json
├── .env.example
├── .eslintrc.json
├── vitest.config.ts
├── README.md
└── API.md
```

## Data Flow

### 1. Request Flow

```
HTTP Request
    ↓
Request ID Middleware (adds request ID)
    ↓
Logging Middleware (logs request)
    ↓
Rate Limiting Middleware (checks rate limits)
    ↓
Authentication Middleware (validates JWT)
    ↓
Route Handler
    ↓
Service Layer (business logic)
    ↓
Database Layer (data access)
    ↓
Response
    ↓
Error Handler (if error)
    ↓
HTTP Response
```

### 2. Authentication Flow

```
User Credentials
    ↓
POST /api/auth/signin
    ↓
AuthService.signin()
    ↓
Supabase Auth (validate credentials)
    ↓
Generate JWT Token
    ↓
Return Token to Client
    ↓
Client stores token (HttpOnly cookie or localStorage)
    ↓
Client includes token in Authorization header
    ↓
authMiddleware validates token
    ↓
Request proceeds with userId
```

### 3. Content Creation Flow

```
User uploads content
    ↓
POST /api/vaults/:vault_id/content
    ↓
ContentService.createContent()
    ↓
Verify vault membership
    ↓
Store content metadata in database
    ↓
Return content item
    ↓
Async: Generate embedding
    ↓
EmbeddingService.generateEmbedding()
    ↓
Call Gemini API
    ↓
Store embedding in pgvector
```

### 4. Chat Flow

```
User sends message
    ↓
POST /api/vaults/:vault_id/chat
    ↓
RagChatService.chat()
    ↓
Generate embedding for message
    ↓
EmbeddingService.similaritySearch()
    ↓
Find similar content items
    ↓
Fetch content details
    ↓
Build context from content
    ↓
Call Gemini API with context
    ↓
Return response with references
```

## Service Layer Design

### AuthService
- Handles user signup and signin
- Validates JWT tokens
- Manages user sessions
- Integrates with Supabase Auth

### VaultService
- CRUD operations for vaults
- Manages vault members and roles
- Handles vault invitations
- Enforces access control

### ContentService
- CRUD operations for content items
- Manages file uploads
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
- Implements retry logic

### EmbeddingService
- Generates embeddings using Gemini API
- Stores embeddings in pgvector
- Supports similarity search
- Implements caching

### GraphService
- Reconstructs semantic graph from database
- Identifies edges based on shared tags
- Supports graph filtering
- Implements caching

### RagChatService
- Retrieves relevant content using semantic search
- Constructs context window
- Calls Gemini API for response generation
- Tracks referenced content items

## Database Schema

### Core Tables

**users**
- id (UUID, PK)
- email (VARCHAR, UNIQUE)
- created_at (TIMESTAMP)

**vaults**
- id (UUID, PK)
- name (VARCHAR)
- description (TEXT)
- created_by (UUID, FK)
- created_at (TIMESTAMP)
- updated_at (TIMESTAMP)

**vault_members**
- vault_id (UUID, FK, PK)
- user_id (UUID, FK, PK)
- role (VARCHAR: 'owner', 'member')
- joined_at (TIMESTAMP)

**vault_invites**
- id (UUID, PK)
- vault_id (UUID, FK)
- invited_email (VARCHAR)
- token (VARCHAR, UNIQUE)
- accepted (BOOLEAN)
- created_at (TIMESTAMP)
- expires_at (TIMESTAMP)

**content_items**
- id (UUID, PK)
- vault_id (UUID, FK)
- created_by (UUID, FK)
- content_type (VARCHAR: 'audio', 'image', 'link')
- title (VARCHAR)
- url (TEXT)
- transcript (TEXT)
- metadata (JSONB)
- created_at (TIMESTAMP)
- updated_at (TIMESTAMP)

**embeddings**
- id (UUID, PK)
- content_item_id (UUID, FK)
- embedding (vector(768))
- model (VARCHAR)
- created_at (TIMESTAMP)

**tags**
- id (UUID, PK)
- vault_id (UUID, FK)
- name (VARCHAR)
- is_special (BOOLEAN)
- color (VARCHAR)
- created_by (UUID, FK)
- created_at (TIMESTAMP)

**item_tags**
- item_id (UUID, FK, PK)
- tag_id (UUID, FK, PK)
- created_at (TIMESTAMP)

## Error Handling

### Error Categories

1. **Validation Errors** (400)
   - Invalid input data
   - Missing required fields

2. **Authentication Errors** (401)
   - Invalid or expired JWT
   - Missing authentication header

3. **Authorization Errors** (403)
   - User lacks permissions
   - Not a vault member

4. **Not Found Errors** (404)
   - Resource doesn't exist

5. **Server Errors** (500)
   - Database failures
   - External API failures
   - Unexpected exceptions

### Error Response Format

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable message",
    "details": {}
  }
}
```

## Caching Strategy

### Cache Layers

1. **Application-level caching** (in-memory)
   - Vault metadata (TTL: 5 minutes)
   - Tag lists (TTL: 5 minutes)
   - User permissions (TTL: 10 minutes)

2. **Database query caching**
   - Semantic graph (TTL: 10 minutes)
   - Content search results (TTL: 5 minutes)

### Cache Invalidation

- Vault updates: Invalidate vault cache
- Content updates: Invalidate graph and search caches
- Tag updates: Invalidate tag and graph caches
- Member changes: Invalidate permission cache

## Security

### Authentication
- JWT tokens issued by Supabase Auth
- Tokens expire after 1 hour
- Refresh tokens for long-lived sessions

### Authorization
- Role-based access control (Owner, Member)
- Vault membership verification
- Content ownership verification

### Input Validation
- All user input validated
- Parameterized queries prevent SQL injection
- File upload validation

### Rate Limiting
- Auth endpoints: 5 requests per 15 minutes per IP
- API endpoints: 100 requests per 15 minutes per user

### Data Protection
- HTTPS for all communication
- Sensitive data encrypted at rest
- Error messages sanitized

## Performance Optimization

### Database Optimization
- Connection pooling
- Indexes on frequently queried columns
- Query optimization
- Pagination for large result sets

### API Optimization
- Response compression
- Caching headers
- Lazy loading
- Batch operations

### External API Optimization
- Rate limiting
- Caching
- Async processing
- Circuit breakers

## Monitoring and Observability

### Logging
- Structured JSON logging
- Request ID tracking
- Performance metrics
- Error logging with context

### Metrics
- Request count by endpoint
- Error rate by endpoint
- Response time percentiles
- Database connection pool status
- External API call success rate

## Deployment

### Vercel Deployment
- Serverless functions
- Automatic scaling
- Environment variable management
- Automatic SSL certificates

### Database Deployment
- Supabase PostgreSQL
- Automatic backups
- Connection pooling
- pgvector extension

## Scalability

### Horizontal Scaling
- Stateless API design
- Database connection pooling
- Caching for reduced database load

### Vertical Scaling
- Optimized queries
- Efficient algorithms
- Resource management

## Disaster Recovery

### Backup Strategy
- Supabase automatic daily backups
- Database migrations in version control
- Environment variables documented

### Recovery Procedures
- Database restoration from backups
- Migration re-execution
- Service restart procedures
