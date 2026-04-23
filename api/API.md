# Cofre Vault Platform - API Documentation

## Base URL

```
https://api.cofre-vault.com
```

## Authentication

All endpoints (except `/api/auth/signup` and `/api/auth/signin`) require a Bearer token in the Authorization header:

```
Authorization: Bearer <session_token>
```

## Response Format

All responses are in JSON format. Successful responses include the requested data:

```json
{
  "user": { ... },
  "vault": { ... },
  "items": [ ... ]
}
```

Error responses follow this format:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable message",
    "details": { ... }
  }
}
```

## Status Codes

- `200 OK` - Successful GET, PUT, or DELETE
- `201 Created` - Successful POST
- `400 Bad Request` - Invalid input
- `401 Unauthorized` - Missing or invalid authentication
- `403 Forbidden` - Insufficient permissions
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server error

## Endpoints

### Authentication

#### Sign Up

```
POST /api/auth/signup
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "secure_password"
}
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

#### Sign In

```
POST /api/auth/signin
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "secure_password"
}
```

Response: Same as Sign Up

#### Sign Out

```
POST /api/auth/signout
Authorization: Bearer <token>
```

Response:
```json
{
  "success": true
}
```

#### Get Current User

```
GET /api/auth/me
Authorization: Bearer <token>
```

Response:
```json
{
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

### Vaults

#### Create Vault

```
POST /api/vaults
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "My Vault",
  "description": "Optional description"
}
```

Response:
```json
{
  "vault": {
    "id": "uuid",
    "name": "My Vault",
    "description": "Optional description",
    "created_by": "user_uuid",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
}
```

#### List Vaults

```
GET /api/vaults
Authorization: Bearer <token>
```

Response:
```json
{
  "vaults": [
    {
      "vault": { ... },
      "role": "owner"
    }
  ]
}
```

#### Get Vault

```
GET /api/vaults/:vault_id
Authorization: Bearer <token>
```

Response:
```json
{
  "vault": { ... }
}
```

#### Update Vault

```
PUT /api/vaults/:vault_id
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Updated Name",
  "description": "Updated description"
}
```

Response:
```json
{
  "vault": { ... }
}
```

#### Delete Vault

```
DELETE /api/vaults/:vault_id
Authorization: Bearer <token>
```

Response:
```json
{
  "success": true
}
```

#### List Vault Members

```
GET /api/vaults/:vault_id/members
Authorization: Bearer <token>
```

Response:
```json
{
  "members": [
    {
      "vault_id": "uuid",
      "user_id": "uuid",
      "role": "owner",
      "joined_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

#### Invite Member

```
POST /api/vaults/:vault_id/members
Authorization: Bearer <token>
Content-Type: application/json

{
  "email": "newmember@example.com"
}
```

Response:
```json
{
  "invite": {
    "id": "uuid",
    "vault_id": "uuid",
    "invited_email": "newmember@example.com",
    "token": "invite_token",
    "accepted": false,
    "created_at": "2024-01-01T00:00:00Z",
    "expires_at": "2024-01-08T00:00:00Z"
  }
}
```

#### Remove Member

```
DELETE /api/vaults/:vault_id/members/:user_id
Authorization: Bearer <token>
```

Response:
```json
{
  "success": true
}
```

#### Accept Invitation

```
POST /api/vaults/invites/:token/accept
Authorization: Bearer <token>
```

Response:
```json
{
  "vault": { ... }
}
```

### Content

#### Upload Content

```
POST /api/vaults/:vault_id/content
Authorization: Bearer <token>
Content-Type: application/json

{
  "content_type": "audio|image|link",
  "url": "https://example.com/file.mp3",
  "title": "Optional title",
  "metadata": { ... }
}
```

Response:
```json
{
  "content_item": {
    "id": "uuid",
    "vault_id": "uuid",
    "created_by": "user_uuid",
    "content_type": "audio",
    "title": "Optional title",
    "url": "https://example.com/file.mp3",
    "transcript": null,
    "metadata": { ... },
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
}
```

#### List Content

```
GET /api/vaults/:vault_id/content?limit=50&offset=0&tag_id=uuid&type=audio
Authorization: Bearer <token>
```

Query Parameters:
- `limit`: Number of items (default: 50, max: 100)
- `offset`: Pagination offset (default: 0)
- `tag_id`: Filter by tag (optional)
- `type`: Filter by content type (optional)

Response:
```json
{
  "items": [ ... ],
  "total": 100
}
```

#### Get Content

```
GET /api/vaults/:vault_id/content/:item_id
Authorization: Bearer <token>
```

Response:
```json
{
  "content_item": { ... }
}
```

#### Update Content

```
PUT /api/vaults/:vault_id/content/:item_id
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "Updated title",
  "transcript": "Updated transcript",
  "metadata": { ... }
}
```

Response:
```json
{
  "content_item": { ... }
}
```

#### Delete Content

```
DELETE /api/vaults/:vault_id/content/:item_id
Authorization: Bearer <token>
```

Response:
```json
{
  "success": true
}
```

#### Add Tags to Content

```
POST /api/vaults/:vault_id/content/:item_id/tags
Authorization: Bearer <token>
Content-Type: application/json

{
  "tag_ids": ["uuid1", "uuid2"]
}
```

Response:
```json
{
  "content_item": { ... }
}
```

### Tags

#### Create Tag

```
POST /api/vaults/:vault_id/tags
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Important",
  "is_special": false,
  "color": "#FF0000"
}
```

Response:
```json
{
  "tag": {
    "id": "uuid",
    "vault_id": "uuid",
    "name": "Important",
    "is_special": false,
    "color": "#FF0000",
    "created_by": "user_uuid",
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

#### List Tags

```
GET /api/vaults/:vault_id/tags
Authorization: Bearer <token>
```

Response:
```json
{
  "tags": [ ... ]
}
```

#### Update Tag

```
PUT /api/vaults/:vault_id/tags/:tag_id
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Updated Name",
  "color": "#00FF00"
}
```

Response:
```json
{
  "tag": { ... }
}
```

#### Delete Tag

```
DELETE /api/vaults/:vault_id/tags/:tag_id
Authorization: Bearer <token>
```

Response:
```json
{
  "success": true
}
```

### Chat

#### Send Chat Message

```
POST /api/vaults/:vault_id/chat
Authorization: Bearer <token>
Content-Type: application/json

{
  "message": "What is in this vault?"
}
```

Response:
```json
{
  "chat_reply_text": "Based on the content in your vault...",
  "referenced_node_ids": ["uuid1", "uuid2"]
}
```

### Graph

#### Get Semantic Graph

```
GET /api/vaults/:vault_id/graph?tag_id=uuid&content_type=audio
Authorization: Bearer <token>
```

Query Parameters:
- `tag_id`: Filter by tag (optional)
- `content_type`: Filter by content type (optional)

Response:
```json
{
  "nodes": [
    {
      "item": { ... },
      "edges": [
        {
          "target_item_id": "uuid",
          "shared_tag": { ... },
          "weight": 1.0
        }
      ]
    }
  ],
  "edge_count": 42
}
```

## Rate Limiting

- Authentication endpoints: 5 requests per 15 minutes per IP
- API endpoints: 100 requests per 15 minutes per user

Rate limit information is included in response headers:
- `RateLimit-Limit`: Total requests allowed
- `RateLimit-Remaining`: Requests remaining
- `RateLimit-Reset`: Unix timestamp when limit resets

## Error Codes

- `INVALID_INPUT` - Invalid request data
- `UNAUTHORIZED` - Missing or invalid authentication
- `FORBIDDEN` - Insufficient permissions
- `NOT_FOUND` - Resource not found
- `INTERNAL_SERVER_ERROR` - Server error
- `SIGNUP_FAILED` - Signup error
- `SIGNIN_FAILED` - Signin error
- `CREATE_VAULT_ERROR` - Vault creation error
- `CREATE_CONTENT_ERROR` - Content creation error
- `TRANSCRIPTION_ERROR` - Audio transcription error
- `EMBEDDING_ERROR` - Embedding generation error
- `CHAT_ERROR` - Chat processing error

## Examples

### Create a vault and add content

```bash
# Sign up
curl -X POST https://api.cofre-vault.com/api/auth/signup \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"password"}'

# Create vault
curl -X POST https://api.cofre-vault.com/api/vaults \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"name":"My Vault"}'

# Add content
curl -X POST https://api.cofre-vault.com/api/vaults/<vault_id>/content \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"content_type":"link","url":"https://example.com","title":"Example"}'

# Chat
curl -X POST https://api.cofre-vault.com/api/vaults/<vault_id>/chat \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"message":"What is in this vault?"}'
```
