# Requirements Document: Cofre Vault Platform

## Introduction

Cofre is a shared digital vault platform that enables teams and groups to collaboratively store, organize, and discover information through a semantic graph model. Rather than traditional folder hierarchies, content is organized as a network of relationships formed by shared tags. The platform supports multiple content types (audio, images, links), in-app audio recording with automatic transcription, and role-based access control. This requirements document captures the functional and non-functional requirements derived from the technical design.

---

## Glossary

- **Vault**: A shared collaborative space where multiple users can store and organize content
- **Vault Member**: A user who has been added to a vault and has permissions to access its content
- **Vault Owner**: The user who created a vault; has full administrative permissions
- **Content Item**: A discrete piece of content stored in a vault (audio, image, or link)
- **Tag**: A label applied to content items to create semantic relationships
- **Special Tag**: A tag with higher semantic weight that acts as a primary graph expansion point
- **Semantic Graph**: An in-memory graph representation where nodes are content items and edges are relationships formed by shared tags
- **Graph Node**: A content item represented as a vertex in the semantic graph
- **Graph Edge**: A connection between two content items that share one or more tags
- **Invite Token**: A unique, single-use token that allows a user to join a vault
- **Transcription**: Automatic conversion of audio content to text via the ElevenLabs API
- **RLS (Row Level Security)**: Database-level access control policies that enforce vault membership
- **Storage URL**: A reference to a file stored in Supabase Storage
- **System**: The Cofre Vault Platform application

---

## Requirements

### Requirement 1: User Authentication

**User Story:** As a user, I want to create an account and sign in securely, so that I can access my vaults and content.

#### Acceptance Criteria

1. THE System SHALL support user registration via email and password through Supabase Auth
2. THE System SHALL support user login via email and password through Supabase Auth
3. WHEN a user signs in successfully, THE System SHALL establish a session and expose the authenticated user to the application
4. THE System SHALL support user sign-out and session termination
5. WHEN a user is not authenticated, THE System SHALL prevent access to vault content and redirect to the authentication interface
6. THE System SHALL expose reactive session state changes to the application via an authentication state change callback

---

### Requirement 2: Vault Creation and Ownership

**User Story:** As a user, I want to create a new vault, so that I can establish a shared space for my team or group.

#### Acceptance Criteria

1. WHEN an authenticated user creates a vault with a name and optional description, THE System SHALL persist the vault to the database
2. THE System SHALL assign a unique UUID identifier to each vault
3. WHEN a vault is created, THE System SHALL automatically add the creator as a vault member with the owner role
4. THE System SHALL store the vault creation timestamp and creator reference
5. THE System SHALL enforce that vault names are non-empty and not exceed 100 characters
6. WHEN a user creates a vault, THE System SHALL make the vault immediately accessible to that user

---

### Requirement 3: Vault Membership and Access Control

**User Story:** As a vault owner, I want to manage who can access my vault, so that I can control collaboration and maintain privacy.

#### Acceptance Criteria

1. FOR ALL vaults, THE System SHALL enforce that only vault members can read or write content
2. WHEN a user attempts to access a vault they are not a member of, THE System SHALL return a 403 Forbidden error without exposing vault existence
3. THE System SHALL maintain a vault_members table that tracks user membership and role assignments
4. THE System SHALL support two member roles: owner and member
5. WHEN a user is added to a vault, THE System SHALL record the join timestamp
6. THE System SHALL enforce access control at the database level using Row Level Security policies

---

### Requirement 4: Vault Member Invitation

**User Story:** As a vault owner, I want to invite other users to join my vault, so that I can collaborate with them.

#### Acceptance Criteria

1. WHEN a vault owner invites a user by email, THE System SHALL generate a unique, cryptographically random invite token
2. THE System SHALL store the invite with the vault reference, invited email, token, and expiration timestamp
3. THE System SHALL set invite tokens to expire after 7 days
4. WHEN an invite is created, THE System SHALL generate an invite link containing the token
5. THE System SHALL make the invite link available to the vault owner for sharing
6. THE System SHALL store the invite creation timestamp

---

### Requirement 5: Invite Acceptance and Single-Use Enforcement

**User Story:** As an invited user, I want to accept an invitation to join a vault, so that I can access the shared space.

#### Acceptance Criteria

1. WHEN a user provides a valid, unexpired invite token, THE System SHALL add the user as a member of the vault with the member role
2. WHEN an invite is accepted, THE System SHALL mark the invite as used and prevent subsequent acceptance attempts
3. IF an invite token is invalid or not found, THEN THE System SHALL return a descriptive error message
4. IF an invite has already been accepted, THEN THE System SHALL return an error indicating the invite is already used
5. IF an invite has expired, THEN THE System SHALL return an error indicating the invite is no longer valid
6. WHEN a user accepts an invite, THE System SHALL record the acceptance timestamp and add the user to vault_members

---

### Requirement 6: Content Item Creation and Storage

**User Story:** As a vault member, I want to upload content to the vault, so that I can share information with my team.

#### Acceptance Criteria

1. WHEN a vault member adds a content item to a vault, THE System SHALL persist the item to the database with a unique UUID
2. THE System SHALL support three content types: audio, image, and link
3. WHEN a content item is created, THE System SHALL record the creator reference, creation timestamp, and vault reference
4. THE System SHALL store a storage URL for audio and image items pointing to Supabase Storage
5. THE System SHALL store an external URL for link items
6. THE System SHALL allow optional title and metadata fields for content items
7. WHEN a content item is created, THE System SHALL make it immediately accessible to all vault members

---

### Requirement 7: Audio Recording and Upload

**User Story:** As a vault member, I want to record audio directly in the application and upload it to the vault, so that I can capture voice notes without external tools.

#### Acceptance Criteria

1. THE System SHALL provide an in-app audio recording interface using the browser MediaRecorder API
2. WHEN a user starts recording, THE System SHALL initialize a recording session
3. WHEN a user stops recording, THE System SHALL capture the audio as a Blob
4. WHEN a user uploads a recorded audio file, THE System SHALL send it to Supabase Storage
5. IF the storage upload fails, THEN THE System SHALL surface the error to the user and not create a partial ContentItem record
6. WHEN audio upload succeeds, THE System SHALL store the storage URL in the ContentItem record
7. THE System SHALL allow users to retry audio upload if the initial attempt fails

---

### Requirement 8: Audio Transcription

**User Story:** As a vault member, I want audio content to be automatically transcribed to text, so that I can search and reference audio content by its spoken content.

#### Acceptance Criteria

1. WHEN an audio item is uploaded, THE System SHALL send the audio to the ElevenLabs API for speech-to-text transcription
2. IF transcription succeeds, THE System SHALL store the transcript text in the ContentItem record
3. IF transcription fails or times out, THE System SHALL save the audio item with a null transcript and display a "transcription pending/failed" indicator
4. THE System SHALL allow manual retry of transcription from the item detail view
5. THE System SHALL not block audio item creation on transcription failure
6. THE System SHALL proxy ElevenLabs API requests through a server-side function to protect the API key

---

### Requirement 9: Tag Creation and Management

**User Story:** As a vault member, I want to create and manage tags to organize content, so that I can establish semantic relationships between items.

#### Acceptance Criteria

1. WHEN a vault member creates a tag, THE System SHALL persist it to the database with a unique UUID scoped to the vault
2. THE System SHALL enforce that tag names are unique within a vault (case-insensitive)
3. IF a user attempts to create a tag with a name that already exists in the vault, THEN THE System SHALL return a validation error
4. THE System SHALL support designating tags as "special" tags with higher semantic weight
5. WHEN a tag is created, THE System SHALL record the creator reference and creation timestamp
6. THE System SHALL allow optional color assignment to tags for UI visualization
7. THE System SHALL make tags immediately available for attachment to content items

---

### Requirement 10: Tag Attachment to Content

**User Story:** As a vault member, I want to attach tags to content items, so that I can create semantic relationships and organize information.

#### Acceptance Criteria

1. WHEN a vault member attaches one or more tags to a content item, THE System SHALL create ItemTag records linking the item to each tag
2. THE System SHALL allow attaching multiple tags to a single item
3. THE System SHALL allow attaching a single tag to multiple items
4. WHEN tags are attached to an item, THE System SHALL make them immediately available for graph construction
5. THE System SHALL prevent duplicate tag attachments (same tag cannot be attached twice to the same item)
6. THE System SHALL allow detaching tags from items

---

### Requirement 11: Tag Uniqueness Enforcement

**User Story:** As a system, I want to ensure tag names remain unique within each vault, so that the semantic graph remains unambiguous.

#### Acceptance Criteria

1. FOR ALL tags in a vault, THE System SHALL enforce that tag names are unique (case-insensitive comparison)
2. WHEN a user attempts to create a tag with a duplicate name, THE System SHALL reject the creation and suggest the existing tag
3. THE System SHALL perform uniqueness validation before database insertion

---

### Requirement 12: Semantic Graph Construction

**User Story:** As a vault member, I want the system to automatically build a semantic graph from content and tags, so that I can explore relationships between items.

#### Acceptance Criteria

1. WHEN a vault is opened, THE System SHALL fetch all content items, tags, and tag attachments for that vault
2. THE System SHALL construct an in-memory semantic graph where each content item is a node
3. THE System SHALL create edges between items that share one or more tags
4. FOR ALL items in the graph, THE System SHALL ensure each item appears as a node exactly once
5. THE System SHALL assign edge weights based on tag type: special tags receive weight 2.0, regular tags receive weight 1.0
6. THE System SHALL complete graph construction on the client-side without server-side processing
7. WHEN the graph is built, THE System SHALL make it available for traversal and neighbor queries

---

### Requirement 13: Graph Edge Creation and Weighting

**User Story:** As a system, I want to weight graph edges based on tag importance, so that special tags create stronger semantic connections.

#### Acceptance Criteria

1. FOR ALL graphs built from content and tags, IF two items share a special tag, THEN their edge weight SHALL be 2.0
2. FOR ALL graphs built from content and tags, IF two items share a regular tag, THEN their edge weight SHALL be 1.0
3. THE System SHALL create bidirectional edges between items sharing tags
4. THE System SHALL not create duplicate edges between the same pair of items (even if they share multiple tags)
5. THE System SHALL ensure edge weights are consistently applied across all graph traversals

---

### Requirement 14: Graph Neighbor Queries

**User Story:** As a vault member, I want to find items related to a specific content item, so that I can discover connected information.

#### Acceptance Criteria

1. WHEN a user queries for neighbors of a content item, THE System SHALL return all items directly connected by at least one shared tag
2. THE System SHALL not include the queried item itself in the neighbor results
3. THE System SHALL return no duplicate items in the neighbor results regardless of how many tags two items share
4. THE System SHALL return an empty list if the item has no tags or no other items share its tags
5. THE System SHALL support neighbor queries for any item in the graph

---

### Requirement 15: Content Item Retrieval

**User Story:** As a vault member, I want to retrieve content items from a vault, so that I can view and work with stored information.

#### Acceptance Criteria

1. WHEN a vault member requests all items in a vault, THE System SHALL return all ContentItem records for that vault
2. WHEN a vault member requests items by tag, THE System SHALL return all items tagged with that tag
3. THE System SHALL enforce that only vault members can retrieve items from a vault
4. THE System SHALL return items with all associated metadata including storage URLs and transcripts
5. THE System SHALL support filtering and pagination for large result sets

---

### Requirement 16: Content Item Deletion

**User Story:** As a vault member, I want to delete content items from a vault, so that I can remove outdated or unwanted information.

#### Acceptance Criteria

1. WHEN a vault member deletes a content item, THE System SHALL remove the item from the database
2. WHEN an item is deleted, THE System SHALL remove all associated ItemTag records
3. THE System SHALL remove the associated file from Supabase Storage if applicable
4. THE System SHALL enforce that only vault members can delete items from their vault
5. WHEN an item is deleted, THE System SHALL update the semantic graph to reflect the removal

---

### Requirement 17: Audio File Storage

**User Story:** As a system, I want to securely store audio files, so that vault members can access them reliably.

#### Acceptance Criteria

1. THE System SHALL store audio files in Supabase Storage in private buckets
2. THE System SHALL organize audio files by vault to maintain logical separation
3. THE System SHALL generate unique file paths for each uploaded audio to prevent collisions
4. THE System SHALL provide signed URLs scoped to vault members for secure file access
5. THE System SHALL enforce that only vault members can access audio files from their vault

---

### Requirement 18: Image File Storage

**User Story:** As a system, I want to securely store image files, so that vault members can access them reliably.

#### Acceptance Criteria

1. THE System SHALL store image files in Supabase Storage in private buckets
2. THE System SHALL organize image files by vault to maintain logical separation
3. THE System SHALL generate unique file paths for each uploaded image to prevent collisions
4. THE System SHALL provide signed URLs scoped to vault members for secure file access
5. THE System SHALL enforce that only vault members can access image files from their vault

---

### Requirement 19: Link Content Storage

**User Story:** As a vault member, I want to save links to external resources in the vault, so that I can curate and share references.

#### Acceptance Criteria

1. WHEN a vault member adds a link to a vault, THE System SHALL store the external URL in the ContentItem record
2. THE System SHALL allow optional metadata storage for link items (e.g., link preview, title, description)
3. THE System SHALL validate that link URLs are well-formed before storage
4. THE System SHALL make links immediately accessible to all vault members

---

### Requirement 20: Vault Member Listing

**User Story:** As a vault member, I want to see who else has access to a vault, so that I can understand the collaboration scope.

#### Acceptance Criteria

1. WHEN a vault member requests the member list, THE System SHALL return all VaultMember records for that vault
2. THE System SHALL include user information and role for each member
3. THE System SHALL include the join timestamp for each member
4. THE System SHALL enforce that only vault members can view the member list

---

### Requirement 21: Vault Listing for User

**User Story:** As a user, I want to see all vaults I have access to, so that I can navigate between my collaborative spaces.

#### Acceptance Criteria

1. WHEN an authenticated user requests their vault list, THE System SHALL return all vaults where the user is a member
2. THE System SHALL include vault metadata (name, description, creation timestamp)
3. THE System SHALL include the user's role in each vault
4. THE System SHALL return an empty list if the user is not a member of any vaults

---

### Requirement 22: Vault Retrieval by ID

**User Story:** As a vault member, I want to retrieve a specific vault by its ID, so that I can access its details and content.

#### Acceptance Criteria

1. WHEN a vault member requests a vault by ID, THE System SHALL return the Vault record with all metadata
2. IF the requesting user is not a member of the vault, THE System SHALL return a 403 Forbidden error
3. THE System SHALL not expose vault existence to non-members

---

### Requirement 23: Authentication Error Handling

**User Story:** As a system, I want to handle authentication errors gracefully, so that users receive clear feedback.

#### Acceptance Criteria

1. WHEN authentication fails, THE System SHALL return a descriptive error message
2. WHEN a user attempts to access protected resources without authentication, THE System SHALL redirect to the authentication interface
3. THE System SHALL not expose sensitive information in error messages

---

### Requirement 24: Storage Upload Error Handling

**User Story:** As a system, I want to handle storage failures gracefully, so that users can retry and recover from errors.

#### Acceptance Criteria

1. IF a Supabase Storage upload fails, THEN THE System SHALL surface the error to the user
2. THE System SHALL not create a partial ContentItem record if storage upload fails
3. THE System SHALL keep the audio or image blob in memory to allow retry
4. THE System SHALL allow the user to retry the upload operation

---

### Requirement 25: Transcription Error Handling

**User Story:** As a system, I want to handle transcription failures gracefully, so that audio items are not lost.

#### Acceptance Criteria

1. IF ElevenLabs transcription fails or times out, THEN THE System SHALL save the audio item with a null transcript
2. THE System SHALL display a "transcription pending/failed" indicator to the user
3. THE System SHALL allow manual retry of transcription from the item detail view
4. THE System SHALL not block audio item creation on transcription failure

---

### Requirement 26: Invite Token Validation

**User Story:** As a system, I want to validate invite tokens thoroughly, so that only authorized users can join vaults.

#### Acceptance Criteria

1. IF an invite token is not found in the database, THEN THE System SHALL return an error indicating the token is invalid
2. IF an invite token has already been accepted, THEN THE System SHALL return an error indicating the invite is already used
3. IF an invite token has expired, THEN THE System SHALL return an error indicating the invite is no longer valid
4. THE System SHALL perform all validation checks before modifying any database state

---

### Requirement 27: Duplicate Tag Prevention

**User Story:** As a system, I want to prevent duplicate tag names within a vault, so that the tag taxonomy remains clean.

#### Acceptance Criteria

1. WHEN a user attempts to create a tag with a name that already exists in the vault, THE System SHALL reject the creation
2. THE System SHALL perform case-insensitive comparison for tag name uniqueness
3. THE System SHALL suggest the existing tag to the user
4. THE System SHALL perform validation before database insertion

---

### Requirement 28: Vault Content Isolation

**User Story:** As a system, I want to ensure content from different vaults is isolated, so that users cannot access content from vaults they don't belong to.

#### Acceptance Criteria

1. FOR ALL content items, THE System SHALL enforce that items can only be accessed by members of their vault
2. WHEN a user requests content from a vault they are not a member of, THE System SHALL return a 403 Forbidden error
3. THE System SHALL enforce isolation at the database level using Row Level Security policies
4. THE System SHALL not expose the existence of content in vaults the user cannot access

---

### Requirement 29: Graph Construction Correctness

**User Story:** As a system, I want to ensure the semantic graph is constructed correctly, so that relationships are accurate.

#### Acceptance Criteria

1. FOR ALL valid inputs to buildGraph, THE System SHALL create a node for every content item exactly once
2. FOR ALL items sharing a tag, THE System SHALL create edges between all pairs of those items
3. THE System SHALL not create duplicate edges between the same pair of items
4. THE System SHALL correctly assign edge weights based on tag type (special: 2.0, regular: 1.0)
5. THE System SHALL handle empty inputs gracefully (empty items, tags, or itemTags arrays)

---

### Requirement 30: Content Item Metadata Persistence

**User Story:** As a vault member, I want content metadata to be preserved, so that I can access complete information about stored items.

#### Acceptance Criteria

1. WHEN a content item is created, THE System SHALL persist all provided metadata to the database
2. THE System SHALL store the creator reference for each item
3. THE System SHALL store the creation timestamp for each item
4. THE System SHALL store the vault reference for each item
5. THE System SHALL preserve optional fields (title, metadata JSON) if provided

---

### Requirement 31: Special Tag Semantic Weight

**User Story:** As a vault member, I want special tags to create stronger semantic connections, so that important relationships are more prominent.

#### Acceptance Criteria

1. WHEN a special tag is created, THE System SHALL mark it with is_special = true
2. WHEN the semantic graph is built, THE System SHALL assign weight 2.0 to edges formed by special tags
3. WHEN the semantic graph is built, THE System SHALL assign weight 1.0 to edges formed by regular tags
4. THE System SHALL ensure special tag edges are consistently weighted across all graph operations

---

### Requirement 32: Vault Member Role Assignment

**User Story:** As a system, I want to track member roles, so that I can enforce appropriate permissions.

#### Acceptance Criteria

1. WHEN a user creates a vault, THE System SHALL assign them the owner role
2. WHEN a user accepts an invite, THE System SHALL assign them the member role
3. THE System SHALL store the role in the VaultMember record
4. THE System SHALL support two roles: owner and member
5. THE System SHALL use role information for future permission enforcement

---

### Requirement 33: Invite Expiration

**User Story:** As a system, I want invite tokens to expire, so that old invites cannot be used indefinitely.

#### Acceptance Criteria

1. WHEN an invite is created, THE System SHALL set an expiration timestamp 7 days in the future
2. WHEN a user attempts to accept an expired invite, THE System SHALL return an error
3. THE System SHALL check expiration status before processing invite acceptance
4. THE System SHALL store the expiration timestamp in the VaultInvite record

---

### Requirement 34: Cryptographic Token Generation

**User Story:** As a system, I want to generate secure invite tokens, so that invites cannot be guessed or forged.

#### Acceptance Criteria

1. WHEN an invite is created, THE System SHALL generate a cryptographically random token
2. THE System SHALL use UUID v4 or equivalent cryptographic randomness
3. THE System SHALL ensure tokens are URL-safe for inclusion in invite links
4. THE System SHALL store the token in the VaultInvite record
5. THE System SHALL ensure tokens are unique across all invites

---

### Requirement 35: Client-Side Graph Building

**User Story:** As a system, I want to build the semantic graph on the client, so that graph operations are fast and responsive.

#### Acceptance Criteria

1. WHEN a vault is opened, THE System SHALL fetch all necessary data (items, tags, itemTags) from the server
2. THE System SHALL construct the semantic graph in-memory on the client-side
3. THE System SHALL not require server-side processing for graph construction
4. THE System SHALL support graph traversal and neighbor queries without server round-trips
5. THE System SHALL rebuild the graph when vault data changes

---

### Requirement 36: Asynchronous Transcription

**User Story:** As a system, I want transcription to be asynchronous, so that audio uploads don't block the user interface.

#### Acceptance Criteria

1. WHEN an audio item is uploaded, THE System SHALL save the item immediately without waiting for transcription
2. THE System SHALL send the transcription request to ElevenLabs asynchronously
3. WHEN transcription completes, THE System SHALL update the ContentItem record with the transcript
4. THE System SHALL not block the UI on transcription completion
5. THE System SHALL allow users to view audio items before transcription completes

---

### Requirement 37: Lazy Loading for Large Vaults

**User Story:** As a system, I want to support large vaults efficiently, so that performance remains acceptable.

#### Acceptance Criteria

1. FOR vaults with hundreds of items, THE System SHALL support pagination or lazy-loading of graph data
2. THE System SHALL not require loading all items into memory at once
3. THE System SHALL allow incremental graph construction as data is loaded
4. THE System SHALL maintain performance for vault operations on large datasets

---

### Requirement 38: Row Level Security Enforcement

**User Story:** As a system, I want to enforce access control at the database level, so that security is not dependent on application logic.

#### Acceptance Criteria

1. THE System SHALL enable Row Level Security on all Supabase tables
2. THE System SHALL define RLS policies that restrict vault content access to vault members
3. THE System SHALL enforce that users can only read/write content in vaults they are members of
4. THE System SHALL not rely on application-level permission checks alone
5. THE System SHALL prevent direct database access from bypassing vault membership checks

---

### Requirement 39: Signed URL Access Control

**User Story:** As a system, I want to control file access via signed URLs, so that only authorized users can download files.

#### Acceptance Criteria

1. WHEN a vault member requests access to a file, THE System SHALL generate a signed URL scoped to that user
2. THE System SHALL set expiration times on signed URLs to limit access duration
3. THE System SHALL ensure signed URLs can only be used by the intended recipient
4. THE System SHALL not expose direct file paths to clients

---

### Requirement 40: ElevenLabs API Key Protection

**User Story:** As a system, I want to protect the ElevenLabs API key, so that it cannot be exposed to clients.

#### Acceptance Criteria

1. THE System SHALL never expose the ElevenLabs API key to the client application
2. THE System SHALL proxy all ElevenLabs requests through a server-side function
3. THE System SHALL use Supabase Edge Functions or equivalent for server-side proxying
4. THE System SHALL validate transcription requests before forwarding to ElevenLabs
5. THE System SHALL handle API responses securely on the server before returning to client

---

## Testing Strategy

### Unit Testing

Unit tests verify individual components in isolation with mocked dependencies:
- AuthService sign-up, sign-in, sign-out, and session management
- VaultService vault creation, member listing, and retrieval
- TagService tag creation and uniqueness validation
- ContentService item creation and retrieval
- buildGraph algorithm with various input combinations
- getNeighbors correctness and duplicate prevention
- acceptInvite state machine (valid, already used, expired)

### Property-Based Testing

Property-based tests verify universal properties across generated inputs using fast-check:
- For any valid items, tags, and itemTags, buildGraph creates exactly one node per item
- For any items sharing a tag, getNeighbors returns all connected items without duplicates
- For any graph, edge count equals the sum of C(n,2) for each tag group of size n
- For any special tag edges, weight is strictly 2.0; for regular tags, weight is 1.0
- For any vault, only members can access content
- For any invite token, acceptInvite succeeds at most once

### Integration Testing

Integration tests verify end-to-end workflows:
- Full authentication flow: sign up → sign in → access vault → sign out
- Vault creation and member invitation
- Audio upload and transcription pipeline
- Content item creation with tag attachment
- Semantic graph construction and traversal
- Invite acceptance and membership verification

