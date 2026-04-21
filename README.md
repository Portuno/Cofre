# Cofre Vault Platform

A shared digital vault platform that enables teams and groups to collaboratively store, organize, and discover information through a semantic graph model.

## Project Structure

```
cofre-vault/
├── Cargo.toml              # Project dependencies and metadata
├── src/
│   ├── lib.rs             # Main library entry point with module organization
│   ├── error.rs           # Error handling types and Result wrapper
│   ├── models.rs          # Core data models (Vault, VaultMember, ContentItem, Tag, etc.)
│   └── db.rs              # Database connection pool and migration framework
├── .env.example           # Environment variables template
└── README.md              # This file
```

## Core Components

### Error Handling (`src/error.rs`)
- Custom `Error` enum with domain-specific error variants
- `Result<T>` type alias for ergonomic error handling
- Comprehensive error messages for debugging and user feedback

### Data Models (`src/models.rs`)
- **Vault**: Shared collaborative space with metadata
- **VaultMember**: User membership with role assignment (owner/member)
- **VaultInvite**: Single-use, time-limited invitation tokens
- **ContentItem**: Discrete content (audio, image, or link) with metadata
- **Tag**: Labels for semantic organization with special tag support
- **ItemTag**: Join table linking items to tags
- **Graph**: In-memory semantic graph representation with nodes and edges

### Database (`src/db.rs`)
- `DatabaseConfig`: Configuration management from environment variables
- `Database`: Connection pool and client for PostgreSQL via Supabase
- Migration framework support
- Health check and initialization methods

## Getting Started

### Prerequisites
- Rust 1.70+ (install from https://rustup.rs/)
- Supabase account (https://supabase.com/)
- PostgreSQL database (provided by Supabase)

### Setup

1. Clone the repository:
```bash
git clone <repository-url>
cd cofre-vault
```

2. Copy the environment template:
```bash
cp .env.example .env
```

3. Configure your environment variables in `.env`:
```
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_KEY=your-anon-key
DATABASE_URL=postgresql://user:password@host:port/database
```

4. Build the project:
```bash
cargo build
```

5. Run tests:
```bash
cargo test
```

## Dependencies

### Core Dependencies
- **tokio**: Async runtime for Rust
- **serde/serde_json**: Serialization/deserialization
- **uuid**: UUID generation (v4 for cryptographic randomness)
- **chrono**: Date/time handling with timezone support
- **sqlx**: Async SQL toolkit with compile-time query verification
- **reqwest**: HTTP client for API calls
- **thiserror**: Ergonomic error handling
- **tracing**: Structured logging and diagnostics

### Development Dependencies
- **tokio-test**: Testing utilities for async code

## Architecture

The platform is built on three pillars:

1. **Authentication**: Supabase Auth for secure user registration and login
2. **Data Storage**: PostgreSQL via Supabase with Row Level Security (RLS)
3. **Semantic Graph**: Client-side graph construction from content and tags

### Key Features

- **Vault Management**: Create vaults, manage members, enforce access control
- **Content Storage**: Support for audio, images, and links with metadata
- **Audio Features**: In-app recording, upload, and automatic transcription
- **Semantic Organization**: Tag-based content relationships with special tag weighting
- **Graph Traversal**: Find related content through shared tags
- **Security**: RLS policies, signed URLs, API key protection

## Development

### Running Tests
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_database_creation
```

### Building for Production
```bash
cargo build --release
```

### Code Quality
```bash
# Check code without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Database Schema

The platform uses the following main tables:

- `vaults`: Vault metadata and ownership
- `vault_members`: User membership with roles
- `vault_invites`: Invitation tokens with expiration
- `content_items`: Stored content with metadata
- `tags`: Semantic labels scoped to vaults
- `item_tags`: Join table for content-tag relationships

All tables have Row Level Security (RLS) policies to enforce vault membership.

## Error Handling

The platform uses a custom `Error` enum with specific variants for different failure modes:

```rust
pub enum Error {
    AuthenticationFailed(String),
    Unauthorized,
    VaultNotFound,
    DuplicateTagName,
    StorageUploadFailed(String),
    TranscriptionFailed(String),
    // ... and more
}
```

All operations return `Result<T>` for ergonomic error handling.

## Security Considerations

1. **Authentication**: Delegated to Supabase Auth
2. **Authorization**: Enforced via Row Level Security at database level
3. **File Access**: Signed URLs with expiration for Supabase Storage
4. **API Keys**: ElevenLabs API key protected via server-side proxy
5. **Invite Tokens**: Cryptographically random (UUID v4), single-use, time-limited

## Performance

- **Graph Construction**: Client-side for responsiveness
- **Transcription**: Asynchronous to avoid UI blocking
- **Pagination**: Support for large vaults with hundreds of items
- **Connection Pooling**: Efficient database resource management

## Contributing

1. Create a feature branch
2. Make your changes
3. Run tests: `cargo test`
4. Format code: `cargo fmt`
5. Lint code: `cargo clippy`
6. Submit a pull request

## License

[License information to be added]

## Support

For issues, questions, or contributions, please open an issue on the repository.
# Cofre
