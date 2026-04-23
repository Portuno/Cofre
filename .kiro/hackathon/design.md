# Cofre: Architecture and Design

## System Overview

Cofre is a semantic shared digital vault platform built with Rust backend and Supabase infrastructure. The system enables couples to collaboratively store and discover information through a semantic graph model where content items are nodes and shared tags create edges.

```
┌─────────────────────────────────────────────────────────────┐
│                     Client Application                       │
│  (Web/Mobile - Semantic Graph Visualization & UI)           │
└────────────────────┬────────────────────────────────────────┘
                     │
        ┌────────────┼────────────┬──────────────┐
        │            │            │              │
        ▼            ▼            ▼              ▼
   ┌─────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐
   │Supabase │ │Supabase  │ │Supabase  │ │ElevenLabs    │
   │Auth     │ │Database  │ │Storage   │ │API (STT)     │
   │         │ │(RLS)     │ │(Private) │ │              │
   └─────────┘ └──────────┘ └──────────┘ └──────────────┘
        │            │            │              │
        └────────────┼────────────┴──────────────┘
                     │
        ┌────────────▼────────────┐
        │  Rust Backend Services  │
        │  (Tokio async runtime)  │
        └─────────────────────────┘
```

---

## Core Services Architecture

### 1. AuthService
**Responsibility**: User authentication and session management

**Key Operations**:
- `signUp(email, password)` → AuthResult
- `signIn(email, password)` → AuthResult
- `signOut()` → Void
- `getCurrentUser()` → User | Null
- `onAuthStateChange(callback)` → Unsubscribe

**Implementation Details**:
- Delegates all auth operations to Supabase Auth client
- Exposes reactive session state to the application
- Handles auth errors and surfaces them to the UI
- Session tokens are managed by Supabase

---

### 2. VaultService
**Responsibility**: Vault lifecycle management and membership control

**Key Operations**:
- `createVault(name, description)` → Vault
- `getVaultsForUser(user_id)` → Vault[]
- `getVaultById(vault_id)` → Vault
- `inviteMember(vault_id, email)` → InviteToken
- `acceptInvite(token)` → Vault
- `getMembers(vault_id)` → VaultMember[]

**Implementation Details**:
- Enforces vault membership checks before all operations
- Generates cryptographically random invite tokens (UUID v4)
- Manages invite token lifecycle (creation, validation, expiration)
- Tracks member roles (owner, member)
- Returns 403 Forbidden for non-members without exposing vault existence

---

### 3. ContentService
**Responsibility**: Content item management and storage

**Key Operations**:
- `addItem(vault_id, item)` → ContentItem
- `getItems(vault_id)` → ContentItem[]
- `deleteItem(item_id)` → Void
- `attachTags(item_id, tag_ids)` → Void
- `getItemsByTag(vault_id, tag_id)` → ContentItem[]

**Implementation Details**:
- Supports three content types: audio, image, link
- Stores URLs for audio/image items (Supabase Storage)
- Stores external URLs for link items
- Cascades ItemTag deletion when items are deleted
- Removes files from Supabase Storage on item deletion
- Enforces vault membership for all operations

---

### 4. AudioService
**Responsibility**: Audio recording, upload, and transcription

**Key Operations**:
- `startRecording()` → RecordingSession
- `stopRecording(session)` → AudioBlob
- `uploadAudio(blob, vault_id)` → StorageURL
- `transcribeAudio(blob)` → TranscriptResult

**Implementation Details**:
- Uses browser MediaRecorder API for in-app recording
- Uploads audio blobs to Supabase Storage with unique paths
- Sends audio to ElevenLabs API via server-side proxy
- Transcription is asynchronous; does not block UI
- Failed transcriptions are handled gracefully (null transcript)
- ElevenLabs API key is protected via Supabase Edge Functions

---

### 5. TagService
**Responsibility**: Tag taxonomy and semantic weight management

**Key Operations**:
- `createTag(vault_id, name, isSpecial)` → Tag
- `getTags(vault_id)` → Tag[]
- `updateTag(tag_id, updates)` → Tag
- `deleteTag(tag_id)` → Void

**Implementation Details**:
- Enforces case-insensitive tag name uniqueness within vaults
- Distinguishes between regular tags (weight 1.0) and special tags (weight 2.0)
- Special tags act as primary graph expansion points
- Tags are scoped per vault
- Cascades ItemTag deletion when tags are deleted

---

### 6. SemanticGraphEngine
**Responsibility**: In-memory semantic graph construction and traversal

**Key Operations**:
- `buildGraph(items, tags, itemTags)` → Graph
- `getNeighbors(graph, item_id)` → ContentItem[]
- `getItemsBySpecialTag(graph, tag_id)` → ContentItem[]
- `getShortestPath(graph, from_id, to_id)` → ContentItem[]

**Implementation Details**:
- Builds graph on client-side for performance
- Represents items as nodes; shared tags create edges
- Assigns edge weights: special tags = 2.0, regular tags = 1.0
- Supports neighborhood expansion for UI graph visualization
- Handles empty inputs gracefully
- Prevents duplicate edges between same item pairs

---

## Data Models

### Vault
```rust
struct Vault {
    id: UUID,
    name: String,
    description: Option<String>,
    created_by: UUID,
    created_at: Timestamp,
}
```

### VaultMember
```rust
struct VaultMember {
    vault_id: UUID,
    user_id: UUID,
    role: Role,  // owner | member
    joined_at: Timestamp,
}
```

### VaultInvite
```rust
struct VaultInvite {
    id: UUID,
    vault_id: UUID,
    invited_email: String,
    token: String,  // unique, URL-safe
    accepted: bool,
    created_at: Timestamp,
    expires_at: Timestamp,
}
```

### ContentItem
```rust
struct ContentItem {
    id: UUID,
    vault_id: UUID,
    created_by: UUID,
    content_type: ContentType,  // audio | image | link
    title: Option<String>,
    url: String,  // storage URL or external link
    transcript: Option<String>,  // for audio items
    metadata: Option<JSON>,
    created_at: Timestamp,
}
```

### Tag
```rust
struct Tag {
    id: UUID,
    vault_id: UUID,
    name: String,
    is_special: bool,
    color: Option<String>,
    created_by: UUID,
    created_at: Timestamp,
}
```

### ItemTag (Join Table)
```rust
struct ItemTag {
    item_id: UUID,
    tag_id: UUID,
    created_at: Timestamp,
}
```

### Graph (In-Memory)
```rust
struct GraphNode {
    item: ContentItem,
    edges: Vec<GraphEdge>,
}

struct GraphEdge {
    target_item_id: UUID,
    shared_tag: Tag,
    weight: f32,  // 2.0 for special, 1.0 for regular
}

struct Graph {
    nodes: HashMap<UUID, GraphNode>,
}
```

---

## Key Algorithms

### buildGraph Algorithm

**Input**: items: Vec<ContentItem>, tags: Vec<Tag>, itemTags: Vec<ItemTag>  
**Output**: Graph

**Pseudocode**:
```
1. Create HashMap of tags indexed by ID
2. Initialize graph nodes for each item
3. Group itemTags by tag_id to find co-tagged items
4. For each tag group:
   a. Calculate edge weight (2.0 for special, 1.0 for regular)
   b. Create bidirectional edges between all pairs of items in group
   c. Prevent duplicate edges
5. Return constructed graph
```

**Correctness Properties**:
- Every item appears as exactly one node
- Two items have an edge if and only if they share at least one tag
- No duplicate edges between same item pairs
- Special tag edges have weight 2.0; regular tags have weight 1.0

**Time Complexity**: O(n + m + k²) where n = items, m = tags, k = max items per tag  
**Space Complexity**: O(n + e) where e = number of edges

---

### getNeighbors Algorithm

**Input**: graph: Graph, item_id: UUID  
**Output**: Vec<ContentItem>

**Pseudocode**:
```
1. Look up item_id in graph.nodes
2. If not found, return empty vector
3. Collect all target_item_ids from edges
4. Remove duplicates (even if multiple shared tags)
5. Exclude the queried item itself
6. Return corresponding ContentItems
```

**Correctness Properties**:
- Returns all items directly connected by shared tags
- No duplicates in result
- Excludes the queried item itself
- Returns empty list for untagged items

**Time Complexity**: O(d) where d = degree of node  
**Space Complexity**: O(d)

---

### acceptInvite State Machine

**Input**: token: String  
**Output**: Vault | Error

**State Transitions**:
```
1. Lookup token in database
   → Not found: Error("Invalid invite token")
   → Found: Continue to step 2

2. Check if already accepted
   → accepted = true: Error("Invite already used")
   → accepted = false: Continue to step 3

3. Check if expired
   → expires_at < now(): Error("Invite expired")
   → expires_at >= now(): Continue to step 4

4. Add user to vault_members with role = member
5. Mark invite as accepted
6. Return vault
```

**Correctness Properties**:
- Invite succeeds at most once
- Expired invites are rejected
- Invalid tokens are rejected
- User is added as member (not owner)

---

## Sequence Diagrams

### Vault Creation & Invitation Flow

```
User                App                Database
  │                  │                    │
  ├─ createVault ───>│                    │
  │                  ├─ INSERT vault ────>│
  │                  │<─ vault_id ────────┤
  │                  ├─ INSERT member ───>│
  │                  │<─ ok ──────────────┤
  │<─ vault created ─┤                    │
  │                  │                    │
  ├─ inviteMember ──>│                    │
  │                  ├─ INSERT invite ───>│
  │                  │<─ token ───────────┤
  │<─ invite link ───┤                    │
```

### Audio Upload & Transcription Flow

```
User                App              Storage         ElevenLabs
  │                  │                  │                │
  ├─ recordAudio ───>│                  │                │
  │<─ blob ─────────┤                  │                │
  │                  │                  │                │
  ├─ uploadAudio ───>│                  │                │
  │                  ├─ upload ────────>│                │
  │                  │<─ storage_url ───┤                │
  │                  │                  │                │
  │                  ├─ transcribe ─────────────────────>│
  │                  │                  │                │
  │                  ├─ INSERT item ────>│                │
  │                  │<─ item_id ────────┤                │
  │<─ audio saved ───┤                  │                │
  │                  │                  │    transcript ─┤
  │                  │<─ transcript ─────────────────────┤
  │                  ├─ UPDATE item ────>│                │
  │                  │<─ ok ──────────────┤                │
```

### Semantic Graph Traversal Flow

```
User                App              Database         Graph Engine
  │                  │                  │                │
  ├─ openVault ─────>│                  │                │
  │                  ├─ SELECT items ──>│                │
  │                  │<─ items ──────────┤                │
  │                  ├─ SELECT tags ───>│                │
  │                  │<─ tags ───────────┤                │
  │                  ├─ SELECT itemTags>│                │
  │                  │<─ itemTags ───────┤                │
  │                  │                  │                │
  │                  ├─ buildGraph ─────────────────────>│
  │                  │<─ graph ──────────────────────────┤
  │<─ graph view ────┤                  │                │
  │                  │                  │                │
  ├─ expandNode ────>│                  │                │
  │                  ├─ getNeighbors ───────────────────>│
  │                  │<─ neighbors ──────────────────────┤
  │<─ highlight ─────┤                  │                │
```

---

## Row Level Security (RLS) Policies

### vault_members Table
```sql
-- Users can only read their own memberships
CREATE POLICY "Users can read own memberships"
  ON vault_members FOR SELECT
  USING (auth.uid() = user_id);

-- Users can only write their own memberships (via invite acceptance)
CREATE POLICY "Users can accept invites"
  ON vault_members FOR INSERT
  WITH CHECK (auth.uid() = user_id);
```

### content_items Table
```sql
-- Users can only read items in vaults they are members of
CREATE POLICY "Users can read vault content"
  ON content_items FOR SELECT
  USING (
    vault_id IN (
      SELECT vault_id FROM vault_members
      WHERE user_id = auth.uid()
    )
  );

-- Users can only write items to vaults they are members of
CREATE POLICY "Users can write vault content"
  ON content_items FOR INSERT
  WITH CHECK (
    vault_id IN (
      SELECT vault_id FROM vault_members
      WHERE user_id = auth.uid()
    )
  );

-- Users can only delete their own items
CREATE POLICY "Users can delete own items"
  ON content_items FOR DELETE
  USING (
    vault_id IN (
      SELECT vault_id FROM vault_members
      WHERE user_id = auth.uid()
    )
  );
```

### tags Table
```sql
-- Users can only read tags in vaults they are members of
CREATE POLICY "Users can read vault tags"
  ON tags FOR SELECT
  USING (
    vault_id IN (
      SELECT vault_id FROM vault_members
      WHERE user_id = auth.uid()
    )
  );

-- Users can only write tags to vaults they are members of
CREATE POLICY "Users can write vault tags"
  ON tags FOR INSERT
  WITH CHECK (
    vault_id IN (
      SELECT vault_id FROM vault_members
      WHERE user_id = auth.uid()
    )
  );
```

---

## Error Handling Strategy

### Authentication Errors
- Invalid credentials: Return descriptive error without exposing user existence
- Session expired: Redirect to login with clear message
- Unauthorized access: Return 403 Forbidden without exposing resource existence

### Storage Errors
- Upload failure: Surface error to user, keep blob in memory for retry
- File not found: Return 404 with descriptive message
- Quota exceeded: Return error with guidance on cleanup

### Transcription Errors
- API failure: Save audio item with null transcript, show "pending/failed" indicator
- Timeout: Retry asynchronously, allow manual retry from UI
- Invalid audio: Return error, allow user to re-record

### Invite Errors
- Invalid token: Return "Invalid invite token"
- Already used: Return "Invite already used"
- Expired: Return "Invite expired"
- User already member: Return "User already member of vault"

---

## Performance Considerations

### Client-Side Graph Construction
- Graph is built on client after fetching data in single query
- Avoids server-side processing overhead
- Enables fast, responsive graph traversal
- For large vaults (100+ items), consider pagination

### Asynchronous Transcription
- Audio items are saved immediately without waiting for transcription
- Transcription request is sent asynchronously
- UI updates when transcript becomes available
- Failed transcriptions do not block user workflow

### Lazy Loading for Large Vaults
- Implement pagination for content items (offset/limit)
- Load graph data incrementally as user explores
- Maintain performance for vaults with hundreds of items

### Database Query Optimization
- Use indexes on vault_id, user_id, tag_id for fast lookups
- Batch queries where possible (SELECT items + tags + itemTags in single round-trip)
- Leverage RLS policies for efficient access control

---

## Security Considerations

### Authentication & Authorization
- All Supabase tables have RLS enabled
- Vault content is only accessible to vault members
- Non-members receive 403 Forbidden without resource exposure
- Session tokens are managed by Supabase Auth

### Data Protection
- Audio and image files stored in private Supabase Storage buckets
- Access via signed URLs scoped to vault members
- Signed URLs have expiration times
- Direct file paths are never exposed to clients

### API Key Protection
- ElevenLabs API key is never exposed to client
- All transcription requests proxied through Supabase Edge Functions
- Server-side validation before forwarding to ElevenLabs
- Responses are sanitized before returning to client

### Invite Token Security
- Tokens are cryptographically random (UUID v4)
- Tokens are single-use (marked as accepted after first use)
- Tokens expire after 7 days
- Tokens are URL-safe for inclusion in invite links

---

## Dependencies

| Dependency | Purpose | Version |
|---|---|---|
| Supabase Auth | User authentication | Latest |
| Supabase Database | PostgreSQL with RLS | Latest |
| Supabase Storage | Private file storage | Latest |
| Supabase Edge Functions | Server-side proxy | Latest |
| ElevenLabs API | Speech-to-text transcription | v1 |
| Tokio | Async runtime | 1.x |
| Serde | Serialization | 1.x |
| UUID | Unique identifiers | 1.x |
| Chrono | Timestamps | 0.4.x |

---

## Testing Strategy

### Unit Tests
- AuthService: sign-up, sign-in, sign-out, session management
- VaultService: vault creation, member listing, retrieval
- TagService: tag creation, uniqueness validation
- ContentService: item creation, retrieval, deletion
- SemanticGraphEngine: buildGraph, getNeighbors, edge weighting

### Property-Based Tests
- buildGraph: For any valid inputs, every item appears as exactly one node
- getNeighbors: For any graph, result contains no duplicates
- Graph edges: For any items sharing a tag, an edge exists between them
- Edge weights: Special tags always have weight 2.0; regular tags have weight 1.0
- Invite tokens: For any token, acceptInvite succeeds at most once

### Integration Tests
- Full auth flow: sign up → sign in → access vault → sign out
- Vault creation and member invitation
- Audio upload and transcription pipeline
- Content tagging and graph construction
- Semantic graph traversal and neighbor queries
- Access control enforcement (RLS)
- Invite acceptance and membership verification

