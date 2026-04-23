# Cofre Vault Platform - Backend API

A modern web application for collaborative vault management, content organization, and RAG-powered chat, built with Node.js/TypeScript and deployed on Vercel with Supabase.

## Features

- **Vault Management**: Create and manage collaborative vaults with role-based access control
- **Content Organization**: Store audio, images, and links with automatic transcription and embedding generation
- **Semantic Search**: Find related content using vector embeddings and semantic similarity
- **RAG Chat**: Ask questions about vault content and get contextual answers powered by Gemini AI
- **Collaboration**: Invite team members and manage permissions
- **Semantic Graph**: Visualize relationships between content items based on shared tags

## Tech Stack

- **Runtime**: Node.js 18+
- **Language**: TypeScript
- **Framework**: Express.js
- **Database**: Supabase PostgreSQL with pgvector
- **Authentication**: Supabase Auth
- **External APIs**: Google Gemini, ElevenLabs
- **Deployment**: Vercel Serverless Functions

## Prerequisites

- Node.js 18+
- npm or yarn
- Supabase account and project
- Google Gemini API key
- ElevenLabs API key

## Setup

### 1. Clone the repository

```bash
git clone <repository-url>
cd api
```

### 2. Install dependencies

```bash
npm install
```

### 3. Configure environment variables

Copy `.env.example` to `.env` and fill in your credentials:

```bash
cp .env.example .env
```

Required variables:
- `DATABASE_URL`: Supabase PostgreSQL connection string
- `SUPABASE_URL`: Your Supabase project URL
- `SUPABASE_KEY`: Supabase anonymous key
- `GEMINI_API_KEY`: Google Gemini API key
- `ELEVENLABS_API_KEY`: ElevenLabs API key

### 4. Run database migrations

```bash
npm run migrate
```

### 5. Start development server

```bash
npm run dev
```

The API will be available at `http://localhost:3000`

## API Endpoints

### Authentication

- `POST /api/auth/signup` - Create a new account
- `POST /api/auth/signin` - Sign in with email and password
- `POST /api/auth/signout` - Sign out
- `GET /api/auth/me` - Get current user info

### Vaults

- `POST /api/vaults` - Create a new vault
- `GET /api/vaults` - List user's vaults
- `GET /api/vaults/:vault_id` - Get vault details
- `PUT /api/vaults/:vault_id` - Update vault
- `DELETE /api/vaults/:vault_id` - Delete vault
- `GET /api/vaults/:vault_id/members` - List vault members
- `POST /api/vaults/:vault_id/members` - Invite member
- `DELETE /api/vaults/:vault_id/members/:user_id` - Remove member
- `POST /api/vaults/invites/:token/accept` - Accept invitation

### Content

- `POST /api/vaults/:vault_id/content` - Upload content
- `GET /api/vaults/:vault_id/content` - List content
- `GET /api/vaults/:vault_id/content/:item_id` - Get content details
- `PUT /api/vaults/:vault_id/content/:item_id` - Update content
- `DELETE /api/vaults/:vault_id/content/:item_id` - Delete content
- `POST /api/vaults/:vault_id/content/:item_id/tags` - Add tags to content

### Tags

- `POST /api/vaults/:vault_id/tags` - Create tag
- `GET /api/vaults/:vault_id/tags` - List tags
- `PUT /api/vaults/:vault_id/tags/:tag_id` - Update tag
- `DELETE /api/vaults/:vault_id/tags/:tag_id` - Delete tag

### Chat & Graph

- `POST /api/vaults/:vault_id/chat` - Send chat message
- `GET /api/vaults/:vault_id/graph` - Get semantic graph

## Development

### Run tests

```bash
npm run test:run
```

### Lint code

```bash
npm run lint
```

### Build for production

```bash
npm run build
```

## Deployment

### Deploy to Vercel

1. Connect your repository to Vercel
2. Set environment variables in Vercel project settings
3. Deploy:

```bash
vercel deploy
```

### Database Setup

1. Create a Supabase project
2. Enable pgvector extension
3. Run migrations on Supabase database
4. Configure connection pooling

## Architecture

### Services

- **AuthService**: User authentication and JWT validation
- **VaultService**: Vault management and collaboration
- **ContentService**: Content CRUD and storage
- **TagService**: Tag management
- **AudioService**: Audio transcription via ElevenLabs
- **EmbeddingService**: Vector generation and similarity search
- **GraphService**: Semantic graph construction
- **RagChatService**: RAG-powered chat interface

### Database Schema

- `users`: User metadata
- `vaults`: Vault containers
- `vault_members`: Vault membership and roles
- `vault_invites`: Invitation tokens
- `content_items`: Content metadata
- `embeddings`: Vector embeddings (pgvector)
- `tags`: Content tags
- `item_tags`: Content-tag relationships

## Error Handling

All errors are returned in a consistent JSON format:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {}
  }
}
```

## Performance

- Database connection pooling for efficient resource usage
- In-memory caching for frequently accessed data
- Pagination for large result sets
- Exponential backoff for external API failures
- Request timeouts to prevent hanging requests

## Security

- JWT-based authentication via Supabase Auth
- Role-based access control (Owner, Member)
- Input validation and sanitization
- SQL injection prevention via parameterized queries
- Rate limiting on authentication endpoints
- CORS configuration
- Security headers via Helmet

## Monitoring

- Structured JSON logging
- Request ID tracking for tracing
- Performance metrics (response time, query time)
- Error logging with context
- Integration with Vercel logs

## Contributing

1. Create a feature branch
2. Make your changes
3. Run tests and linting
4. Submit a pull request

## License

MIT
