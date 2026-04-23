# Implementation Tasks: Cofre Hackathon MVP

## Phase 1: Core Infrastructure & Authentication

- [x] 1. Project setup and core infrastructure
  - [x] 1.1 Initialize Rust project with Cargo.toml
  - [x] 1.2 Add dependencies: supabase-rs, tokio, serde, uuid, chrono
  - [x] 1.3 Define core data models (Vault, VaultMember, ContentItem, Tag, ItemTag)
  - [x] 1.4 Set up database connection pool
  - [x] 1.5 Create error handling types and Result wrapper

- [x] 2. Implement AuthService
  - [x] 2.1 Implement signUp(email, password) with Supabase Auth
  - [x] 2.2 Implement signIn(email, password) with Supabase Auth
  - [x] 2.3 Implement signOut() and session termination
  - [x] 2.4 Implement getCurrentUser() to retrieve authenticated user
  - [x] 2.5 Set up reactive auth state change callbacks
  - [x] 2.6 Write unit tests for AuthService

---

## Phase 2: Vault Management & Membership

- [x] 3. Implement VaultService - core operations
  - [x] 3.1 Implement createVault(name, description)
  - [x] 3.2 Implement getVaultsForUser(user_id)
  - [x] 3.3 Implement getVaultById(vault_id) with access control
  - [x] 3.4 Write unit tests for vault operations

- [x] 4. Implement vault membership and invitations
  - [x] 4.1 Implement inviteMember(vault_id, email) with token generation
  - [x] 4.2 Implement acceptInvite(token) with state machine validation
  - [x] 4.3 Implement getMembers(vault_id)
  - [x] 4.4 Write unit tests for invite acceptance
  - [x] 4.5 Write property test for single-use invite tokens
  - [x] 4.6 Write property test for invite expiration enforcement

---

## Phase 3: Tag Management

- [x] 5. Implement TagService
  - [x] 5.1 Implement createTag(vault_id, name, isSpecial) with uniqueness validation
  - [x] 5.2 Implement getTags(vault_id)
  - [x] 5.3 Implement updateTag(tag_id, updates) and deleteTag(tag_id)
  - [x] 5.4 Write property test for tag name uniqueness
  - [x] 5.5 Write unit tests for tag creation and validation

---

## Phase 4: Content Management

- [x] 6. Implement ContentService - core operations
  - [x] 6.1 Implement addItem(vault_id, item) for content creation
  - [x] 6.2 Implement getItems(vault_id)
  - [x] 6.3 Implement getItemsByTag(vault_id, tag_id)
  - [x] 6.4 Implement deleteItem(item_id) with cascade deletion
  - [x] 6.5 Write unit tests for content operations

- [x] 7. Implement tag attachment to content
  - [x] 7.1 Implement attachTags(item_id, tag_ids)
  - [x] 7.2 Implement detachTags(item_id, tag_ids)
  - [x] 7.3 Write unit tests for tag attachment

---

## Phase 5: Audio Features

- [~] 8. Implement AudioService - recording and upload
  - [ ] 8.1 Implement audio recording interface using MediaRecorder API
  - [ ] 8.2 Implement uploadAudio(blob, vault_id) to Supabase Storage
  - [ ] 8.3 Write unit tests for audio recording and upload

- [~] 9. Implement audio transcription
  - [ ] 9.1 Implement transcribeAudio(blob) via ElevenLabs API proxy
  - [ ] 9.2 Implement processAudioItem(blob, vault_id, tags) orchestration
  - [ ] 9.3 Write property test for audio transcription completeness
  - [ ] 9.4 Write unit tests for transcription workflow

---

## Phase 6: Semantic Graph Engine

- [~] 10. Implement SemanticGraphEngine - core algorithm
  - [~] 10.1 Implement buildGraph(items, tags, itemTags) algorithm
  - [~] 10.2 Write property test for graph node completeness
  - [~] 10.3 Write property test for graph edge correctness
  - [~] 10.4 Write property test for special tag edge weighting
  - [~] 10.5 Write property test for regular tag edge weighting
  - [~] 10.6 Write property test for graph construction idempotence
  - [~] 10.7 Write unit tests for buildGraph

- [~] 11. Implement SemanticGraphEngine - neighbor queries
  - [ ] 11.1 Implement getNeighbors(graph, item_id)
  - [ ] 11.2 Write property test for neighbor query correctness
  - [ ] 11.3 Write property test for neighbor uniqueness
  - [ ] 11.4 Write unit tests for getNeighbors

- [~] 12. Implement graph traversal and special tag queries
  - [ ] 12.1 Implement getItemsBySpecialTag(graph, tag_id)
  - [ ] 12.2 Implement getShortestPath(graph, from_id, to_id) (optional)
  - [ ] 12.3 Write unit tests for graph traversal

---

## Phase 7: Security & Access Control

- [~] 13. Implement Row Level Security (RLS) policies
  - [ ] 13.1 Define RLS policies for vault_members table
  - [ ] 13.2 Define RLS policies for content_items table
  - [ ] 13.3 Define RLS policies for tags and item_tags tables
  - [ ] 13.4 Write property test for vault access control
  - [ ] 13.5 Write integration tests for RLS enforcement

- [~] 14. Implement signed URL access control
  - [ ] 14.1 Implement signed URL generation for audio files
  - [ ] 14.2 Implement signed URL generation for image files
  - [ ] 14.3 Write unit tests for signed URL generation

---

## Phase 8: Error Handling & Validation

- [~] 15. Implement error handling and validation
  - [ ] 15.1 Implement authentication error handling
  - [ ] 15.2 Implement storage upload error handling
  - [ ] 15.3 Implement transcription error handling
  - [ ] 15.4 Write unit tests for error handling

- [~] 16. Implement data model validation
  - [ ] 16.1 Implement ContentItem vault integrity validation
  - [ ] 16.2 Write property test for content item vault integrity
  - [ ] 16.3 Write property test for vault member role assignment
  - [ ] 16.4 Write property test for cryptographic token uniqueness
  - [ ] 16.5 Write unit tests for data model validation

---

## Phase 9: Performance & Optimization

- [~] 17. Implement lazy loading and performance optimization
  - [ ] 17.1 Implement pagination for large vault content
  - [ ] 17.2 Implement incremental graph construction
  - [ ] 17.3 Write integration tests for large vault performance

---

## Phase 10: Integration Testing

- [~] 18. Implement integration tests - authentication and vault workflows
  - [ ] 18.1 Write integration test for full auth flow
  - [ ] 18.2 Write integration test for vault creation and membership
  - [ ] 18.3 Write integration test for audio upload and transcription
  - [ ] 18.4 Write integration test for content tagging and graph construction

- [~] 19. Implement integration tests - semantic graph operations
  - [ ] 19.1 Write integration test for graph neighbor queries
  - [ ] 19.2 Write integration test for graph traversal with special tags
  - [ ] 19.3 Write integration test for content deletion and graph updates

- [~] 20. Implement integration tests - access control and security
  - [ ] 20.1 Write integration test for vault content isolation
  - [ ] 20.2 Write integration test for signed URL access control
  - [ ] 20.3 Write integration test for ElevenLabs API key protection

---

## Phase 11: Final Validation

- [~] 21. Final checkpoint - Ensure all tests pass
  - [ ] 21.1 Verify all unit tests pass
  - [ ] 21.2 Verify all property-based tests pass
  - [ ] 21.3 Verify all integration tests pass
  - [ ] 21.4 Verify no regressions in core functionality

---

## Summary

**Completed**: 
- Project infrastructure and core models
- AuthService with sign-up, sign-in, sign-out
- VaultService with vault creation and membership
- TagService with uniqueness validation
- ContentService with CRUD operations
- Tag attachment to content items
- Property-based tests for correctness validation

**In Progress**:
- SemanticGraphEngine (buildGraph algorithm partially implemented)
- Audio recording and transcription
- RLS policies and access control

**Remaining**:
- Audio transcription via ElevenLabs
- Complete graph engine implementation
- Neighbor queries and graph traversal
- Error handling and validation
- Integration tests
- Performance optimization

**MVP Scope**: All completed and in-progress tasks are required for the hackathon MVP. Remaining tasks can be prioritized based on time constraints.

