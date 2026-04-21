# Implementation Plan: Cofre Vault Platform

## Overview

This implementation plan breaks down the Cofre Vault Platform into discrete, incremental coding tasks using Rust. The tasks are organized by feature area and build upon each other, starting with core infrastructure (authentication, database models), moving through service implementations (vault, content, audio, tags), and concluding with the semantic graph engine and integration testing.

Each task references specific requirements for traceability. Property-based tests are included as optional sub-tasks where the design specifies correctness properties.

---

## Tasks

- [x] 1. Set up project structure and core infrastructure
  - Create Rust project with Cargo.toml dependencies (Supabase client, tokio, serde, uuid, chrono)
  - Define core data models as Rust structs: Vault, VaultMember, VaultInvite, ContentItem, Tag, ItemTag
  - Set up database connection pool and migration framework
  - Create error handling types and Result wrapper
  - _Requirements: 1.1, 2.1, 3.3, 4.1, 6.1, 9.1_

- [x] 2. Implement authentication service
  - [x] 2.1 Implement AuthService with sign-up and sign-in methods
    - Integrate with Supabase Auth client
    - Handle email/password registration and login
    - Return AuthResult with user session token
    - _Requirements: 1.1, 1.2, 1.3_
  
  - [x] 2.2 Implement session management and sign-out
    - Expose getCurrentUser() to retrieve authenticated user
    - Implement sign-out with session termination
    - Set up reactive auth state change callbacks
    - _Requirements: 1.3, 1.4, 1.6_
  
  - [x] 2.3 Write unit tests for AuthService
    - Test successful sign-up and sign-in flows
    - Test invalid credentials handling
    - Test session state changes
    - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [x] 3. Implement vault service - core operations
  - [x] 3.1 Implement createVault method
    - Validate vault name (non-empty, ≤100 characters)
    - Create Vault record with UUID and timestamps
    - Automatically add creator as owner in vault_members
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_
  
  - [x] 3.2 Implement getVaultsForUser method
    - Query all vaults where user is a member
    - Return vault metadata with user's role
    - Handle empty result gracefully
    - _Requirements: 21.1, 21.2, 21.3_
  
  - [x] 3.3 Implement getVaultById method
    - Enforce vault membership check (403 if not member)
    - Return vault with all metadata
    - Do not expose vault existence to non-members
    - _Requirements: 22.1, 22.2, 22.3_
  
  - [x] 3.4 Write unit tests for vault creation and retrieval
    - Test vault creation with valid inputs
    - Test owner role assignment
    - Test access control (non-member rejection)
    - _Requirements: 2.1, 2.2, 2.3, 22.1, 22.2_

- [x] 4. Implement vault membership and invitations
  - [x] 4.1 Implement inviteMember method
    - Generate cryptographically random invite token (UUID v4)
    - Create VaultInvite record with 7-day expiration
    - Generate invite link containing token
    - _Requirements: 4.1, 4.2, 4.3, 34.1, 34.2, 34.3, 34.4_
  
  - [x] 4.2 Implement acceptInvite method with state machine
    - Validate token exists (return error if not found)
    - Check if invite already accepted (return error if used)
    - Check if invite expired (return error if past expiration)
    - Add user to vault_members with member role
    - Mark invite as used (single-use enforcement)
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 26.1, 26.2, 26.3, 26.4_
  
  - [x] 4.3 Implement getMembers method
    - Query all VaultMember records for vault
    - Include user info and role for each member
    - Include join timestamp
    - Enforce vault membership check
    - _Requirements: 20.1, 20.2, 20.3, 20.4_
  
  - [x] 4.4 Write property test for single-use invite tokens
    - **Property 2: Single-Use Invite Tokens**
    - **Validates: Requirements 5.2, 5.3, 5.4, 5.5, 5.6**
  
  - [x] 4.5 Write property test for invite expiration enforcement
    - **Property 11: Invite Expiration Enforcement**
    - **Validates: Requirements 5.4, 5.5, 26.3, 33.2, 33.3**
  
  - [x] 4.6 Write unit tests for invite acceptance
    - Test valid invite acceptance
    - Test already-used invite rejection
    - Test expired invite rejection
    - Test invalid token rejection
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6_

- [x] 5. Implement tag service
  - [x] 5.1 Implement createTag method with uniqueness validation
    - Validate tag name is non-empty
    - Enforce case-insensitive uniqueness within vault
    - Return error if duplicate name exists
    - Create Tag record with is_special flag
    - _Requirements: 9.1, 9.2, 11.1, 11.2, 11.3, 27.1, 27.2, 27.3_
  
  - [x] 5.2 Implement getTags method
    - Query all tags for a vault
    - Return tags with metadata (name, is_special, color)
    - _Requirements: 9.1_
  
  - [x] 5.3 Implement updateTag and deleteTag methods
    - Allow updating tag properties (name, is_special, color)
    - Enforce uniqueness on name updates
    - Delete tag and cascade to ItemTag records
    - _Requirements: 9.1, 9.6_
  
  - [x] 5.4 Write property test for tag name uniqueness
    - **Property 6: Tag Name Uniqueness**
    - **Validates: Requirements 9.2, 11.1, 11.2, 27.1, 27.2, 27.3**
  
  - [x] 5.5 Write unit tests for tag creation and validation
    - Test tag creation with valid inputs
    - Test duplicate name rejection (case-insensitive)
    - Test special tag flag assignment
    - _Requirements: 9.1, 9.2, 11.1, 11.2_

- [x] 6. Implement content service - core operations
  - [x] 6.1 Implement addItem method for content creation
    - Validate vault membership
    - Create ContentItem record with UUID and timestamps
    - Support three content types: audio, image, link
    - Store storage URL or external URL based on type
    - Allow optional title and metadata fields
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 30.1, 30.2, 30.3, 30.4, 30.5_
  
  - [x] 6.2 Implement getItems method
    - Query all ContentItem records for vault
    - Return items with all metadata including URLs and transcripts
    - Enforce vault membership check
    - _Requirements: 15.1, 15.4_
  
  - [x] 6.3 Implement getItemsByTag method
    - Query items tagged with specific tag
    - Return items with full metadata
    - Enforce vault membership check
    - _Requirements: 15.2, 15.3_
  
  - [x] 6.4 Implement deleteItem method
    - Remove ContentItem record from database
    - Cascade delete all ItemTag records for item
    - Remove associated file from Supabase Storage if applicable
    - Update semantic graph to reflect removal
    - _Requirements: 16.1, 16.2, 16.3, 16.4, 16.5_
  
  - [x] 6.5 Write unit tests for content item operations
    - Test item creation with valid inputs
    - Test item retrieval by vault and tag
    - Test item deletion with cascade
    - _Requirements: 6.1, 6.2, 6.3, 15.1, 15.2, 16.1_

- [x] 7. Implement tag attachment to content
  - [x] 7.1 Implement attachTags method
    - Create ItemTag records linking item to tags
    - Allow multiple tags per item
    - Prevent duplicate tag attachments
    - Make tags immediately available for graph construction
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_
  
  - [x] 7.2 Implement detachTags method
    - Remove ItemTag records for specified tags
    - Allow partial detachment (remove some tags, keep others)
    - _Requirements: 10.6_
  
  - [x] 7.3 Write unit tests for tag attachment
    - Test attaching single and multiple tags
    - Test duplicate prevention
    - Test detachment
    - _Requirements: 10.1, 10.2, 10.3, 10.5, 10.6_

- [-] 8. Implement audio service - recording and upload
  - [ ] 8.1 Implement audio recording interface
    - Use browser MediaRecorder API for in-app recording
    - Implement startRecording() to initialize session
    - Implement stopRecording() to capture audio as Blob
    - Return RecordingSession and AudioBlob types
    - _Requirements: 7.1, 7.2, 7.3_
  
  - [ ] 8.2 Implement uploadAudio method
    - Send audio blob to Supabase Storage
    - Generate unique file path per vault
    - Return signed URL for uploaded file
    - Handle storage upload failures without creating partial ContentItem
    - Allow retry on failure
    - _Requirements: 7.4, 7.5, 7.6, 7.7, 17.1, 17.2, 17.3, 17.4, 17.5_
  
  - [ ] 8.3 Write unit tests for audio recording and upload
    - Test recording session lifecycle
    - Test audio blob capture
    - Test storage upload success and failure
    - Test retry mechanism
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6_

- [~] 9. Implement audio transcription
  - [ ] 9.1 Implement transcribeAudio method
    - Send audio blob to ElevenLabs API via server-side proxy
    - Proxy through Supabase Edge Function to protect API key
    - Return transcript text on success
    - Handle transcription failures gracefully (return null)
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 40.1, 40.2, 40.3, 40.4, 40.5_
  
  - [ ] 9.2 Implement processAudioItem method
    - Orchestrate upload and transcription workflow
    - Upload audio to storage first
    - Send transcription request asynchronously
    - Create ContentItem with storage URL
    - Update transcript when transcription completes
    - Do not block on transcription failure
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 36.1, 36.2, 36.3, 36.4, 36.5_
  
  - [ ] 9.3 Write property test for audio transcription completeness
    - **Property 3: Audio Transcription Completeness**
    - **Validates: Requirements 8.2, 8.3**
  
  - [ ] 9.4 Write unit tests for transcription
    - Test successful transcription
    - Test transcription failure handling
    - Test async transcription workflow
    - Test retry mechanism
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 8.6_

- [~] 10. Implement image and link content storage
  - [ ] 10.1 Implement image upload to Supabase Storage
    - Generate unique file path per vault
    - Store image in private bucket
    - Return signed URL for access
    - Enforce vault member access control
    - _Requirements: 18.1, 18.2, 18.3, 18.4, 18.5_
  
  - [ ] 10.2 Implement link content storage
    - Validate link URLs are well-formed
    - Store external URL in ContentItem record
    - Allow optional metadata (preview, title, description)
    - Make links immediately accessible to vault members
    - _Requirements: 19.1, 19.2, 19.3, 19.4_
  
  - [ ] 10.3 Write unit tests for image and link storage
    - Test image upload and signed URL generation
    - Test link URL validation
    - Test metadata storage
    - _Requirements: 18.1, 18.2, 19.1, 19.2, 19.3_

- [x] 11. Implement semantic graph engine - core algorithm
  - [~] 11.1 Implement buildGraph algorithm
    - Create GraphNode for every content item exactly once
    - Index tags by ID for efficient lookup
    - Group ItemTags by tag_id to find co-tagged items
    - Create bidirectional edges between items sharing tags
    - Assign weight 2.0 for special tag edges, 1.0 for regular tags
    - Prevent duplicate edges between same item pairs
    - Handle empty inputs gracefully
    - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5, 12.6, 12.7, 13.1, 13.2, 13.3, 13.4, 29.1, 29.2, 29.3, 29.4_
  
  - [~] 11.2 Write property test for graph node completeness
    - **Property 8: Graph Node Completeness**
    - **Validates: Requirements 12.4, 29.1**
  
  - [~] 11.3 Write property test for graph edge correctness
    - **Property 9: Graph Edge Correctness**
    - **Validates: Requirements 12.3, 13.3, 13.4, 29.2, 29.3**
  
  - [~] 11.4 Write property test for special tag edge weighting
    - **Property 4: Special Tag Edge Weighting**
    - **Validates: Requirements 13.1, 13.2, 31.2, 31.3, 31.4**
  
  - [~] 11.5 Write property test for regular tag edge weighting
    - **Property 10: Regular Tag Edge Weighting**
    - **Validates: Requirements 13.2, 31.2, 31.3, 31.4**
  
  - [~] 11.6 Write property test for graph construction idempotence
    - **Property 14: Graph Construction Idempotence**
    - **Validates: Requirements 12.1, 12.2, 12.6, 12.7, 29.1**
  
  - [~] 11.7 Write unit tests for buildGraph
    - Test with various item/tag combinations
    - Test with empty inputs
    - Test edge weight assignment
    - Test duplicate edge prevention
    - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5, 13.1, 13.2, 13.3, 13.4_

- [~] 12. Implement semantic graph engine - neighbor queries
  - [ ] 12.1 Implement getNeighbors method
    - Return all items directly connected by shared tags
    - Exclude the queried item itself from results
    - Prevent duplicate items in results (even with multiple shared tags)
    - Return empty list if item has no tags or no neighbors
    - _Requirements: 14.1, 14.2, 14.3, 14.4, 14.5_
  
  - [ ] 12.2 Write property test for neighbor query correctness
    - **Property 15: Neighbor Query Correctness**
    - **Validates: Requirements 14.1, 14.4, 14.5**
  
  - [ ] 12.3 Write property test for neighbor uniqueness
    - **Property 5: Graph Neighbor Uniqueness**
    - **Validates: Requirements 14.2, 14.3**
  
  - [ ] 12.4 Write unit tests for getNeighbors
    - Test neighbor retrieval for items with tags
    - Test no-duplicate guarantee
    - Test empty result for untagged items
    - Test self-exclusion
    - _Requirements: 14.1, 14.2, 14.3, 14.4, 14.5_

- [~] 13. Implement graph traversal and special tag queries
  - [ ] 13.1 Implement getItemsBySpecialTag method
    - Query all items tagged with a specific special tag
    - Return items with full metadata
    - _Requirements: 31.1, 31.2_
  
  - [ ] 13.2 Implement getShortestPath method (optional graph traversal)
    - Find shortest path between two items in graph
    - Use BFS or Dijkstra's algorithm
    - Return path as ordered list of items
    - _Requirements: 12.7_
  
  - [ ] 13.3 Write unit tests for graph traversal
    - Test special tag queries
    - Test shortest path calculation
    - _Requirements: 31.1, 31.2_

- [~] 14. Implement database access control with Row Level Security
  - [ ] 14.1 Define RLS policies for vault_members table
    - Users can only read/write their own vault memberships
    - Enforce vault membership for all content access
    - _Requirements: 3.6, 28.1, 28.2, 28.3, 28.4, 38.1, 38.2, 38.3, 38.4, 38.5_
  
  - [ ] 14.2 Define RLS policies for content_items table
    - Users can only access items in vaults they are members of
    - Enforce at database level, not application level
    - _Requirements: 28.1, 28.2, 28.3, 28.4, 38.1, 38.2, 38.3, 38.4, 38.5_
  
  - [ ] 14.3 Define RLS policies for tags and item_tags tables
    - Users can only access tags in vaults they are members of
    - Enforce cascading access control
    - _Requirements: 28.1, 28.2, 28.3, 28.4, 38.1, 38.2, 38.3, 38.4, 38.5_
  
  - [ ] 14.4 Write property test for vault access control
    - **Property 1: Vault Access Control**
    - **Validates: Requirements 3.1, 28.1, 28.2, 28.3, 28.4**
  
  - [ ] 14.5 Write integration tests for RLS enforcement
    - Test that non-members cannot read vault content
    - Test that non-members cannot write to vault
    - Test that members can access their vaults
    - _Requirements: 3.1, 28.1, 28.2, 28.3, 28.4, 38.1, 38.2, 38.3, 38.4, 38.5_

- [~] 15. Implement signed URL access control for storage
  - [ ] 15.1 Implement signed URL generation for audio files
    - Generate signed URLs scoped to vault members
    - Set expiration times on signed URLs
    - Ensure URLs can only be used by intended recipient
    - Do not expose direct file paths to clients
    - _Requirements: 17.4, 17.5, 39.1, 39.2, 39.3, 39.4_
  
  - [ ] 15.2 Implement signed URL generation for image files
    - Generate signed URLs scoped to vault members
    - Set expiration times on signed URLs
    - Enforce vault member access control
    - _Requirements: 18.4, 18.5, 39.1, 39.2, 39.3, 39.4_
  
  - [ ] 15.3 Write unit tests for signed URL generation
    - Test signed URL creation and expiration
    - Test access control enforcement
    - _Requirements: 39.1, 39.2, 39.3, 39.4_

- [~] 16. Implement error handling and validation
  - [ ] 16.1 Implement authentication error handling
    - Return descriptive error messages on auth failure
    - Redirect to auth interface for unauthenticated access
    - Do not expose sensitive information in errors
    - _Requirements: 23.1, 23.2, 23.3_
  
  - [ ] 16.2 Implement storage upload error handling
    - Surface storage errors to user
    - Do not create partial ContentItem on upload failure
    - Keep blob in memory for retry
    - Allow retry operations
    - _Requirements: 24.1, 24.2, 24.3, 24.4_
  
  - [ ] 16.3 Implement transcription error handling
    - Save audio item with null transcript on transcription failure
    - Display "transcription pending/failed" indicator
    - Allow manual retry from item detail view
    - Do not block audio item creation
    - _Requirements: 25.1, 25.2, 25.3, 25.4_
  
  - [ ] 16.4 Write unit tests for error handling
    - Test auth error messages
    - Test storage failure recovery
    - Test transcription failure handling
    - _Requirements: 23.1, 23.2, 24.1, 24.2, 25.1, 25.2, 25.3_

- [~] 17. Implement data model validation and integrity
  - [ ] 17.1 Implement ContentItem vault integrity validation
    - Verify vault reference exists in database
    - Verify creator is member of vault
    - Enforce on item creation
    - _Requirements: 7.1, 30.1, 30.2, 30.3, 30.4, 30.5_
  
  - [ ] 17.2 Write property test for content item vault integrity
    - **Property 7: Content Item Vault Integrity**
    - **Validates: Requirements 6.1, 6.2, 30.1, 30.2, 30.3, 30.4, 30.5**
  
  - [ ] 17.3 Write property test for vault member role assignment
    - **Property 13: Vault Member Role Assignment**
    - **Validates: Requirements 2.3, 5.1, 32.1, 32.2, 32.3, 32.4, 32.5**
  
  - [ ] 17.4 Write property test for cryptographic token uniqueness
    - **Property 12: Cryptographic Token Uniqueness**
    - **Validates: Requirements 34.1, 34.2, 34.3, 34.4, 34.5**
  
  - [ ] 17.5 Write unit tests for data model validation
    - Test vault name validation (non-empty, ≤100 chars)
    - Test tag name validation
    - Test URL validation for links
    - _Requirements: 2.5, 9.1, 19.3_

- [~] 18. Implement lazy loading and performance optimization
  - [ ] 18.1 Implement pagination for large vault content
    - Support offset/limit pagination for content items
    - Support lazy-loading of graph data
    - Maintain performance for vaults with hundreds of items
    - _Requirements: 15.5, 37.1, 37.2, 37.3, 37.4_
  
  - [ ] 18.2 Implement incremental graph construction
    - Support building graph incrementally as data loads
    - Avoid loading all items into memory at once
    - Maintain performance for large datasets
    - _Requirements: 37.1, 37.2, 37.3, 37.4_
  
  - [ ] 18.3 Write integration tests for large vault performance
    - Test pagination with hundreds of items
    - Test graph construction performance
    - _Requirements: 37.1, 37.2, 37.3, 37.4_

- [~] 19. Checkpoint - Ensure all core services pass tests
  - Ensure all unit tests pass for AuthService, VaultService, TagService, ContentService, AudioService
  - Ensure all property-based tests pass for graph algorithms and access control
  - Verify error handling works as expected
  - Ask the user if questions arise.

- [~] 20. Implement integration tests - authentication and vault workflows
  - [ ] 20.1 Write integration test for full auth flow
    - Sign up → sign in → access vault → sign out
    - Verify session state changes
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_
  
  - [ ] 20.2 Write integration test for vault creation and membership
    - Create vault → invite user → accept invite → verify membership
    - Verify owner and member roles
    - _Requirements: 2.1, 2.2, 2.3, 4.1, 4.2, 5.1, 5.2, 32.1, 32.2_
  
  - [ ] 20.3 Write integration test for audio upload and transcription
    - Record audio → upload to storage → transcribe → verify transcript stored
    - Test async transcription workflow
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 8.1, 8.2, 8.3, 8.4, 8.5, 8.6_
  
  - [ ] 20.4 Write integration test for content tagging and graph construction
    - Create items → attach tags → build graph → query neighbors
    - Verify graph correctness
    - _Requirements: 6.1, 10.1, 10.2, 12.1, 12.2, 12.3, 14.1, 14.2, 14.3, 14.4, 14.5_

- [~] 21. Implement integration tests - semantic graph operations
  - [ ] 21.1 Write integration test for graph neighbor queries
    - Build graph → query neighbors → verify correctness
    - Test with special and regular tags
    - _Requirements: 12.1, 12.2, 12.3, 14.1, 14.2, 14.3, 14.4, 14.5_
  
  - [ ] 21.2 Write integration test for graph traversal with special tags
    - Build graph → query by special tag → verify edge weights
    - _Requirements: 13.1, 31.1, 31.2, 31.3, 31.4_
  
  - [ ] 21.3 Write integration test for content deletion and graph updates
    - Create items with tags → delete item → rebuild graph → verify removal
    - _Requirements: 16.1, 16.2, 16.3, 16.4, 16.5_

- [~] 22. Implement integration tests - access control and security
  - [ ] 22.1 Write integration test for vault content isolation
    - Create two vaults → verify users cannot access other vault's content
    - Test RLS enforcement at database level
    - _Requirements: 28.1, 28.2, 28.3, 28.4, 38.1, 38.2, 38.3, 38.4, 38.5_
  
  - [ ] 22.2 Write integration test for signed URL access control
    - Generate signed URLs → verify only intended recipient can access
    - Test URL expiration
    - _Requirements: 39.1, 39.2, 39.3, 39.4_
  
  - [ ] 22.3 Write integration test for ElevenLabs API key protection
    - Verify API key is never exposed to client
    - Verify transcription requests are proxied through server
    - _Requirements: 40.1, 40.2, 40.3, 40.4, 40.5_

- [~] 23. Final checkpoint - Ensure all tests pass
  - Ensure all unit tests pass
  - Ensure all property-based tests pass
  - Ensure all integration tests pass
  - Verify no regressions in core functionality
  - Ask the user if questions arise.

---

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP, but are strongly recommended for correctness validation
- Each task references specific requirements for traceability
- Property-based tests use fast-check library for Rust (or equivalent)
- Checkpoints ensure incremental validation and catch issues early
- All code should follow Rust best practices: proper error handling with Result types, idiomatic patterns, and comprehensive documentation
- The semantic graph is built client-side for performance; server provides data via single query per vault open
- Audio transcription is asynchronous; UI should not block on it
- Row Level Security policies are enforced at database level for security
- Signed URLs provide secure file access without exposing direct paths
