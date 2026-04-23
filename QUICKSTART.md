# Quick Start Guide

## Prerequisites

- Node.js 18+
- npm or yarn
- Supabase account
- Google Gemini API key
- ElevenLabs API key

## 5-Minute Setup

### 1. Install Dependencies

```bash
cd api
npm install
```

### 2. Configure Environment

```bash
cp .env.example .env
```

Edit `.env` with your credentials:
```
DATABASE_URL=postgresql://user:password@host:5432/database
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_KEY=your-supabase-anon-key
GEMINI_API_KEY=your-gemini-api-key
ELEVENLABS_API_KEY=your-elevenlabs-api-key
```

### 3. Run Migrations

```bash
npm run migrate
```

### 4. Start Development Server

```bash
npm run dev
```

Server runs at `http://localhost:3000`

## Test the API

### 1. Sign Up

```bash
curl -X POST http://localhost:3000/api/auth/signup \
  -H "Content-Type: application/json" \
  -d '{
    "email": "user@example.com",
    "password": "SecurePassword123"
  }'
```

Response:
```json
{
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "created_at": "2024-01-01T00:00:00Z"
  },
  "session_token": "jwt_token"
}
```

### 2. Create a Vault

```bash
curl -X POST http://localhost:3000/api/vaults \
  -H "Authorization: Bearer <session_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My First Vault",
    "description": "A test vault"
  }'
```

### 3. Create a Tag

```bash
curl -X POST http://localhost:3000/api/vaults/<vault_id>/tags \
  -H "Authorization: Bearer <session_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Important",
    "color": "#FF0000"
  }'
```

### 4. Add Content

```bash
curl -X POST http://localhost:3000/api/vaults/<vault_id>/content \
  -H "Authorization: Bearer <session_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "content_type": "link",
    "url": "https://example.com",
    "title": "Example Link"
  }'
```

### 5. Chat

```bash
curl -X POST http://localhost:3000/api/vaults/<vault_id>/chat \
  -H "Authorization: Bearer <session_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "What is in this vault?"
  }'
```

## Common Commands

```bash
# Development
npm run dev          # Start dev server
npm run build        # Build for production
npm run start        # Start production server

# Testing
npm run test         # Run tests in watch mode
npm run test:run     # Run tests once

# Code Quality
npm run lint         # Lint code
npm run migrate      # Run database migrations

# Database
npm run migrate      # Run pending migrations
```

## Troubleshooting

### Database Connection Error

1. Verify DATABASE_URL is correct
2. Check Supabase firewall settings
3. Ensure pgvector extension is enabled

```sql
-- In Supabase SQL Editor
CREATE EXTENSION IF NOT EXISTS vector;
```

### API Key Errors

1. Verify GEMINI_API_KEY is valid
2. Verify ELEVENLABS_API_KEY is valid
3. Check API quotas and limits

### Migration Errors

1. Check migration files exist in `supabase/migrations/`
2. Verify database connection
3. Check for syntax errors in SQL

```bash
# Re-run migrations
npm run migrate
```

### Port Already in Use

```bash
# Use different port
PORT=3001 npm run dev
```

## Project Structure

```
api/
├── src/
│   ├── services/        # Business logic
│   ├── routes/          # API endpoints
│   ├── middleware/      # Express middleware
│   ├── db/              # Database layer
│   └── utils/           # Utilities
├── supabase/
│   └── migrations/      # SQL migrations
├── package.json
└── README.md
```

## Next Steps

1. **Read the Documentation**
   - `README.md` - Project overview
   - `API.md` - API endpoint documentation
   - `ARCHITECTURE.md` - System design

2. **Explore the Code**
   - Services in `src/services/`
   - Routes in `src/routes/`
   - Database layer in `src/db/`

3. **Deploy to Vercel**
   - See `DEPLOYMENT.md` for instructions

4. **Build Frontend**
   - Create React/Next.js frontend
   - Connect to API endpoints

## API Documentation

Full API documentation available in `API.md`

Key endpoints:
- `POST /api/auth/signup` - Create account
- `POST /api/auth/signin` - Sign in
- `POST /api/vaults` - Create vault
- `POST /api/vaults/:vault_id/content` - Add content
- `POST /api/vaults/:vault_id/chat` - Chat

## Support

- Check `README.md` for detailed setup
- Review `API.md` for endpoint details
- See `ARCHITECTURE.md` for system design
- Check `DEPLOYMENT.md` for production deployment

## Tips

1. **Use Postman or Insomnia** for API testing
2. **Check logs** for debugging: `npm run dev`
3. **Monitor database** in Supabase dashboard
4. **Test locally** before deploying
5. **Keep .env secure** - never commit to git

## Performance Tips

1. Use pagination for large datasets
2. Cache frequently accessed data
3. Monitor query performance
4. Use indexes on frequently queried columns
5. Implement rate limiting

## Security Tips

1. Use strong passwords
2. Keep API keys secure
3. Validate all user input
4. Use HTTPS in production
5. Monitor access logs

Happy coding! 🚀
