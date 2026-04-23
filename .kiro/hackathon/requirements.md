# Cofre: Semantic Shared Digital Vault for Couples

## Executive Summary

Cofre is a shared digital vault platform designed for couples to collaboratively store, organize, and discover information through a semantic graph model. Rather than traditional folder hierarchies, content is organized as a network of relationships formed by shared tags. The platform supports multiple content types (audio, images, links), in-app audio recording with automatic transcription via ElevenLabs, and role-based access control.

---

## Core Functional Requirements

### 1. User Authentication
- Users can create accounts and sign in securely via Supabase Auth
- Session management with reactive state changes
- Sign-out and session termination support
- Unauthenticated users are redirected to authentication interface

### 2. Vault Creation and Ownership
- Authenticated users can create new vaults with name and optional description
- Vault creator is automatically assigned the owner role
- Each vault has a unique UUID identifier and creation timestamp

### 3. Vault Membership and Access Control
- Only vault members can read or write content
- Non-members receive 403 Forbidden errors without vault existence exposure
- Two member roles: owner and member
- Row Level Security (RLS) enforces access control at the database level

### 4. Vault Member Invitation
- Vault owners can invite other users by email
- Cryptographically random, single-use invite tokens with 7-day expiration
- Invite links are generated and shareable
- Tokens are validated before acceptance

### 5. Content Item Management
- Vault members can upload three content types: audio, images, and links
- Each item has a unique UUID, creator reference, and creation timestamp
- Audio and image items store URLs pointing to Supabase Storage
- Link items store external URLs with optional metadata
- Items are immediately accessible to all vault members

### 6. Audio Recording and Upload
- In-app audio recording using browser MediaRecorder API
- Audio files are uploaded to Supabase Storage in private buckets
- Unique file paths prevent collisions
- Signed URLs provide secure, scoped access to vault members

### 7. Audio Transcription
- Audio items are automatically sent to ElevenLabs API for speech-to-text
- Transcripts are stored in the ContentItem record
- Transcription is asynchronous; audio items are saved immediately
- Failed transcriptions display a "pending/failed" indicator with manual retry option
- ElevenLabs API key is protected via server-side proxy (Supabase Edge Functions)

### 8. Tag Creation and Management
- Vault members can create tags to organize content
- Tag names are unique within a vault (case-insensitive)
- Tags can be designated as "special" with higher semantic weight
- Tags are immediately available for attachment to content items

### 9. Tag Attachment to Content
- Multiple tags can be attached to a single item
- A single tag can be attached to multiple items
- Duplicate tag attachments are prevented
- Tags can be detached from items

### 10. Semantic Graph Construction
- When a vault is opened, the system fetches all items, tags, and tag attachments
- An in-memory semantic graph is built on the client-side
- Each content item is a node; edges connect items sharing tags
- Special tags create edges with weight 2.0; regular tags create edges with weight 1.0
- Graph construction is fast and responsive without server-side processing

### 11. Graph Neighbor Queries
- Users can query for items related to a specific content item
- Neighbors are all items directly connected by at least one shared tag
- Results contain no duplicates regardless of multiple shared tags
- Empty results are returned for untagged items or items with no neighbors

### 12. Content Item Retrieval and Deletion
- Vault members can retrieve all items in a vault
- Items can be filtered by tag
- Deleted items are removed from the database and Supabase Storage
- Associated ItemTag records are cascaded on deletion
- The semantic graph is updated to reflect deletions

### 13. Vault Member Listing
- Vault members can view all members with their roles and join timestamps
- Only vault members can access the member list

### 14. Vault Listing for User
- Authenticated users can view all vaults they are members of
- Vault metadata (name, description, creation timestamp) and user role are included
- Empty list is returned if user is not a member of any vaults

### 15. Error Handling
- Authentication errors return descriptive messages without exposing sensitive information
- Storage upload failures surface errors to users without creating partial records
- Transcription failures save audio items with null transcripts
- Invite token validation provides clear error messages for invalid, used, or expired tokens

---

## Non-Functional Requirements

### Performance
- Semantic graph construction completes quickly on the client-side
- Large vaults (hundreds of items) support pagination and lazy-loading
- Graph traversal and neighbor queries execute without server round-trips

### Security
- All Supabase tables have Row Level Security enabled
- Vault content is only accessible to vault members
- Invite tokens are cryptographically random and single-use
- Audio and image files are stored in private buckets with signed URL access
- ElevenLabs API key is never exposed to the client

### Reliability
- Audio uploads can be retried on failure
- Transcription failures do not block audio item creation
- Invite tokens expire after 7 days to prevent indefinite access

### Scalability
- The system supports multiple vaults per user
- Each vault can contain hundreds of items
- Pagination and lazy-loading support large datasets

---

## Correctness Properties

The following properties define the formal correctness guarantees of the system:

1. **Vault Access Control**: For any vault and any user, if the user is not a member, they cannot read or write content.
2. **Single-Use Invite Tokens**: For any invite token, `acceptInvite` succeeds at most once.
3. **Audio Transcription Completeness**: For any audio item where transcription succeeds, the transcript is non-empty.
4. **Special Tag Edge Weighting**: For any graph, if two items share a special tag, their edge weight is exactly 2.0.
5. **Graph Neighbor Uniqueness**: For any graph and any item, `getNeighbors` returns no duplicate items.
6. **Tag Name Uniqueness**: For any vault, all tag names are unique (case-insensitive).
7. **Content Item Vault Integrity**: For any content item, the vault reference exists and the creator is a vault member.
8. **Graph Node Completeness**: For any graph, every item appears as exactly one node.
9. **Graph Edge Correctness**: For any graph, two items have an edge if and only if they share at least one tag.
10. **Regular Tag Edge Weighting**: For any graph, if two items share only regular tags, their edge weight is exactly 1.0.
11. **Invite Expiration Enforcement**: For any invite token past its expiration, attempting to accept it raises an error.
12. **Cryptographic Token Uniqueness**: For any two invite tokens, they are distinct and cryptographically random.
13. **Vault Member Role Assignment**: For any vault, the creator is assigned owner role; users accepting invites are assigned member role.
14. **Graph Construction Idempotence**: For any vault, building the graph multiple times from the same data produces equivalent structures.
15. **Neighbor Query Correctness**: For any item in a graph, neighbors are exactly those items sharing at least one tag, excluding the queried item itself.

---

## MVP Scope

The MVP prioritizes:
- User authentication and vault creation
- Vault membership and invitations
- Audio recording, upload, and transcription
- Tag creation and attachment
- Semantic graph construction and neighbor queries
- Content item management (create, retrieve, delete)
- Access control via RLS and signed URLs

Optional features for future iterations:
- Image and link content types (basic support included)
- Advanced graph traversal (shortest path, community detection)
- Caching and performance optimization
- Analytics and usage tracking

---

## Success Criteria

- All core services (Auth, Vault, Content, Audio, Tag, Graph) are implemented and tested
- Property-based tests validate correctness properties
- Integration tests verify end-to-end workflows
- Audio transcription works reliably with ElevenLabs API
- Semantic graph construction is fast and accurate
- Access control is enforced at the database level
- Users can create vaults, invite members, upload audio, and explore the semantic graph

