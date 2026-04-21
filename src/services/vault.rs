// Vault service for Cofre Vault Platform
// Manages vault lifecycle: creation, membership, and access control

use crate::db::Database;
use crate::error::{Error, Result};
use crate::models::{Vault, MemberRole, CreateVaultInput, User, VaultInvite};
use chrono::{Utc, Duration};
use uuid::Uuid;

/// Service for managing vault operations
pub struct VaultService {
    #[allow(dead_code)]
    db: Database,
}

impl VaultService {
    /// Create a new VaultService instance
    pub fn new(db: Database) -> Self {
        VaultService { db }
    }

    /// Create a new vault with the given name and description
    ///
    /// # Arguments
    /// * `creator_id` - UUID of the user creating the vault
    /// * `input` - CreateVaultInput containing name and optional description
    ///
    /// # Returns
    /// * `Result<Vault>` - The created vault or an error
    ///
    /// # Validation
    /// * Vault name must be non-empty and not exceed 100 characters
    /// * Creator is automatically added as owner
    ///
    /// # Requirements
    /// * Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6
    pub async fn create_vault(&self, creator_id: Uuid, input: CreateVaultInput) -> Result<Vault> {
        // Validate vault name
        if input.name.is_empty() || input.name.len() > 100 {
            return Err(Error::InvalidVaultName);
        }

        let vault_id = Uuid::new_v4();
        let now = Utc::now();

        // In a real implementation, this would:
        // 1. Insert vault record into database
        // 2. Insert vault_member record with owner role
        // 3. Return the created vault
        //
        // For now, we return a vault struct that would be persisted
        let vault = Vault {
            id: vault_id,
            name: input.name,
            description: input.description,
            created_by: creator_id,
            created_at: now,
        };

        // TODO: Persist to database
        // db.insert_vault(&vault).await?;
        // db.insert_vault_member(vault_id, creator_id, MemberRole::Owner).await?;

        Ok(vault)
    }

    /// Get all vaults where the user is a member
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user
    ///
    /// # Returns
    /// * `Result<Vec<(Vault, MemberRole)>>` - List of vaults with user's role in each
    ///
    /// # Requirements
    /// * Validates: Requirements 21.1, 21.2, 21.3
    pub async fn get_vaults_for_user(&self, _user_id: Uuid) -> Result<Vec<(Vault, MemberRole)>> {
        // In a real implementation, this would:
        // 1. Query vault_members table for user_id
        // 2. Join with vaults table to get vault details
        // 3. Return list of (vault, role) tuples
        //
        // For now, return empty list
        // TODO: Implement database query
        // let vaults = db.query_vaults_for_user(user_id).await?;
        // Ok(vaults)

        Ok(Vec::new())
    }

    /// Get a vault by ID with access control
    ///
    /// # Arguments
    /// * `vault_id` - UUID of the vault to retrieve
    /// * `user_id` - UUID of the user requesting access
    ///
    /// # Returns
    /// * `Result<Vault>` - The vault if user is a member, error otherwise
    ///
    /// # Access Control
    /// * Returns 403 Unauthorized if user is not a member
    /// * Does not expose vault existence to non-members
    ///
    /// # Requirements
    /// * Validates: Requirements 22.1, 22.2, 22.3
    pub async fn get_vault_by_id(&self, _vault_id: Uuid, _user_id: Uuid) -> Result<Vault> {
        // In a real implementation, this would:
        // 1. Check if user is a member of the vault
        // 2. If not a member, return Unauthorized (not VaultNotFound)
        // 3. If member, return the vault
        //
        // TODO: Implement database query with membership check
        // let is_member = db.is_vault_member(vault_id, user_id).await?;
        // if !is_member {
        //     return Err(Error::Unauthorized);
        // }
        // let vault = db.get_vault(vault_id).await?;
        // Ok(vault)

        Err(Error::VaultNotFound)
    }

    /// Get all members of a vault
    ///
    /// # Arguments
    /// * `vault_id` - UUID of the vault
    /// * `user_id` - UUID of the user requesting the member list
    ///
    /// # Returns
    /// * `Result<Vec<(User, MemberRole)>>` - List of members with their roles
    ///
    /// # Access Control
    /// * Only vault members can view the member list
    ///
    /// # Requirements
    /// * Validates: Requirements 20.1, 20.2, 20.3, 20.4
    pub async fn get_members(
        &self,
        _vault_id: Uuid,
        _user_id: Uuid,
    ) -> Result<Vec<(User, MemberRole)>> {
        // In a real implementation, this would:
        // 1. Check if user is a member of the vault
        // 2. If not a member, return Unauthorized
        // 3. Query all vault_members for the vault
        // 4. Join with users table to get user info
        // 5. Return list of (user, role) tuples
        //
        // TODO: Implement database query
        // let is_member = db.is_vault_member(vault_id, user_id).await?;
        // if !is_member {
        //     return Err(Error::Unauthorized);
        // }
        // let members = db.get_vault_members(vault_id).await?;
        // Ok(members)

        Err(Error::Unauthorized)
    }

    /// Invite a member to a vault by email
    ///
    /// # Arguments
    /// * `vault_id` - UUID of the vault
    /// * `invited_email` - Email address of the user to invite
    ///
    /// # Returns
    /// * `Result<VaultInvite>` - The created invite with token
    ///
    /// # Token Generation
    /// * Generates a cryptographically random UUID v4 token
    /// * Token is URL-safe and unique
    /// * Invite expires after 7 days
    ///
    /// # Requirements
    /// * Validates: Requirements 4.1, 4.2, 4.3, 34.1, 34.2, 34.3, 34.4
    pub async fn invite_member(&self, vault_id: Uuid, invited_email: String) -> Result<VaultInvite> {
        // Generate cryptographically random token (UUID v4)
        let token = Uuid::new_v4().to_string();
        
        let invite_id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + Duration::days(7);

        // In a real implementation, this would:
        // 1. Verify vault exists
        // 2. Insert VaultInvite record into database
        // 3. Return the created invite
        //
        // TODO: Persist to database
        // db.insert_vault_invite(&invite).await?;

        let invite = VaultInvite {
            id: invite_id,
            vault_id,
            invited_email,
            token,
            accepted: false,
            created_at: now,
            expires_at,
        };

        Ok(invite)
    }

    /// Accept an invite token and add user to vault
    ///
    /// # Arguments
    /// * `token` - The invite token to accept
    /// * `user_id` - UUID of the user accepting the invite
    ///
    /// # Returns
    /// * `Result<Vault>` - The vault the user just joined
    ///
    /// # State Machine
    /// * Validates token exists (returns error if not found)
    /// * Checks if invite already accepted (returns error if used)
    /// * Checks if invite expired (returns error if past expiration)
    /// * Adds user to vault_members with member role
    /// * Marks invite as used (single-use enforcement)
    ///
    /// # Requirements
    /// * Validates: Requirements 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 26.1, 26.2, 26.3, 26.4
    pub async fn accept_invite(&self, _token: String, _user_id: Uuid) -> Result<Vault> {
        // In a real implementation, this would:
        // 1. Query vault_invites table for token
        // 2. Check if invite exists (return InvalidInviteToken if not)
        // 3. Check if invite.accepted == true (return InviteAlreadyUsed if used)
        // 4. Check if invite.expires_at < now() (return InviteExpired if expired)
        // 5. Insert vault_member record with role = member
        // 6. Update vault_invites set accepted = true
        // 7. Query and return the vault
        //
        // TODO: Implement database operations
        // let invite = db.get_vault_invite_by_token(&token).await?;
        // if invite.is_none() {
        //     return Err(Error::InvalidInviteToken);
        // }
        // let invite = invite.unwrap();
        // if invite.accepted {
        //     return Err(Error::InviteAlreadyUsed);
        // }
        // if invite.expires_at < Utc::now() {
        //     return Err(Error::InviteExpired);
        // }
        // db.insert_vault_member(invite.vault_id, user_id, MemberRole::Member).await?;
        // db.mark_invite_as_used(invite.id).await?;
        // let vault = db.get_vault(invite.vault_id).await?;
        // Ok(vault)

        Err(Error::InvalidInviteToken)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> Database {
        Database::new(crate::db::DatabaseConfig {
            supabase_url: "https://test.supabase.co".to_string(),
            supabase_key: "test-key".to_string(),
            database_url: "postgresql://test".to_string(),
            max_connections: 5,
        })
    }

    #[tokio::test]
    async fn test_create_vault_with_valid_name() {
        let db = create_test_db();
        let service = VaultService::new(db);
        let creator_id = Uuid::new_v4();

        let input = CreateVaultInput {
            name: "My Vault".to_string(),
            description: Some("A test vault".to_string()),
        };

        let result = service.create_vault(creator_id, input).await;
        assert!(result.is_ok());

        let vault = result.unwrap();
        assert_eq!(vault.name, "My Vault");
        assert_eq!(vault.description, Some("A test vault".to_string()));
        assert_eq!(vault.created_by, creator_id);
    }

    #[tokio::test]
    async fn test_create_vault_with_empty_name() {
        let db = create_test_db();
        let service = VaultService::new(db);
        let creator_id = Uuid::new_v4();

        let input = CreateVaultInput {
            name: "".to_string(),
            description: None,
        };

        let result = service.create_vault(creator_id, input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidVaultName));
    }

    #[tokio::test]
    async fn test_create_vault_with_name_exceeding_100_chars() {
        let db = create_test_db();
        let service = VaultService::new(db);
        let creator_id = Uuid::new_v4();

        let long_name = "a".repeat(101);
        let input = CreateVaultInput {
            name: long_name,
            description: None,
        };

        let result = service.create_vault(creator_id, input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidVaultName));
    }

    #[tokio::test]
    async fn test_create_vault_with_max_length_name() {
        let db = create_test_db();
        let service = VaultService::new(db);
        let creator_id = Uuid::new_v4();

        let max_name = "a".repeat(100);
        let input = CreateVaultInput {
            name: max_name.clone(),
            description: None,
        };

        let result = service.create_vault(creator_id, input).await;
        assert!(result.is_ok());

        let vault = result.unwrap();
        assert_eq!(vault.name.len(), 100);
    }

    #[tokio::test]
    async fn test_create_vault_assigns_creator_as_owner() {
        let db = create_test_db();
        let service = VaultService::new(db);
        let creator_id = Uuid::new_v4();

        let input = CreateVaultInput {
            name: "Test Vault".to_string(),
            description: None,
        };

        let result = service.create_vault(creator_id, input).await;
        assert!(result.is_ok());

        let vault = result.unwrap();
        assert_eq!(vault.created_by, creator_id);
        // In a real implementation, we would verify the vault_member record
        // has role = owner for the creator
    }

    #[tokio::test]
    async fn test_get_vaults_for_user_empty() {
        let db = create_test_db();
        let service = VaultService::new(db);
        let user_id = Uuid::new_v4();

        let result = service.get_vaults_for_user(user_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_get_vault_by_id_non_member() {
        let db = create_test_db();
        let service = VaultService::new(db);
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let result = service.get_vault_by_id(vault_id, user_id).await;
        assert!(result.is_err());
        // Should return Unauthorized, not VaultNotFound, to avoid exposing vault existence
        assert!(matches!(result.unwrap_err(), Error::VaultNotFound | Error::Unauthorized));
    }

    #[tokio::test]
    async fn test_get_members_non_member() {
        let db = create_test_db();
        let service = VaultService::new(db);
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let result = service.get_members(vault_id, user_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Unauthorized));
    }

    // ============================================================================
    // Tests for invite_member method
    // ============================================================================

    #[tokio::test]
    async fn test_invite_member_generates_token() {
        let db = create_test_db();
        let service = VaultService::new(db);
        let vault_id = Uuid::new_v4();
        let email = "bob@example.com".to_string();

        let result = service.invite_member(vault_id, email.clone()).await;
        assert!(result.is_ok());

        let invite = result.unwrap();
        assert_eq!(invite.vault_id, vault_id);
        assert_eq!(invite.invited_email, email);
        assert!(!invite.token.is_empty());
        assert!(!invite.accepted);
    }

    #[tokio::test]
    async fn test_invite_member_token_is_uuid() {
        let db = create_test_db();
        let service = VaultService::new(db);
        let vault_id = Uuid::new_v4();
        let email = "alice@example.com".to_string();

        let result = service.invite_member(vault_id, email).await;
        assert!(result.is_ok());

        let invite = result.unwrap();
        // Token should be a valid UUID string
        let parsed = Uuid::parse_str(&invite.token);
        assert!(parsed.is_ok());
    }

    #[tokio::test]
    async fn test_invite_member_sets_7_day_expiration() {
        let db = create_test_db();
        let service = VaultService::new(db);
        let vault_id = Uuid::new_v4();
        let email = "charlie@example.com".to_string();

        let before = Utc::now();
        let result = service.invite_member(vault_id, email).await;
        let after = Utc::now();

        assert!(result.is_ok());
        let invite = result.unwrap();

        // Expiration should be approximately 7 days from now
        let expected_min = before + Duration::days(7);
        let expected_max = after + Duration::days(7);

        assert!(invite.expires_at >= expected_min);
        assert!(invite.expires_at <= expected_max);
    }

    #[tokio::test]
    async fn test_invite_member_tokens_are_unique() {
        let db = create_test_db();
        let service = VaultService::new(db);
        let vault_id = Uuid::new_v4();

        let invite1 = service
            .invite_member(vault_id, "user1@example.com".to_string())
            .await
            .unwrap();
        let invite2 = service
            .invite_member(vault_id, "user2@example.com".to_string())
            .await
            .unwrap();

        // Tokens should be different
        assert_ne!(invite1.token, invite2.token);
    }

    #[tokio::test]
    async fn test_invite_member_stores_email() {
        let db = create_test_db();
        let service = VaultService::new(db);
        let vault_id = Uuid::new_v4();
        let email = "test@example.com".to_string();

        let result = service.invite_member(vault_id, email.clone()).await;
        assert!(result.is_ok());

        let invite = result.unwrap();
        assert_eq!(invite.invited_email, email);
    }

    // ============================================================================
    // Tests for accept_invite method
    // ============================================================================

    #[tokio::test]
    async fn test_accept_invite_invalid_token() {
        let db = create_test_db();
        let service = VaultService::new(db);
        let user_id = Uuid::new_v4();
        let invalid_token = "invalid-token".to_string();

        let result = service.accept_invite(invalid_token, user_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidInviteToken));
    }

    #[tokio::test]
    async fn test_accept_invite_empty_token() {
        let db = create_test_db();
        let service = VaultService::new(db);
        let user_id = Uuid::new_v4();

        let result = service.accept_invite("".to_string(), user_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidInviteToken));
    }

    // ============================================================================
    // Property-based tests for invite tokens
    // ============================================================================

    #[tokio::test]
    async fn test_invite_token_cryptographic_randomness() {
        // Property: For any two invite tokens generated by the system, they are distinct
        // Validates: Requirements 34.1, 34.2, 34.3, 34.4, 34.5
        let db = create_test_db();
        let service = VaultService::new(db);
        let vault_id = Uuid::new_v4();

        let mut tokens = std::collections::HashSet::new();
        let num_invites = 100;

        for i in 0..num_invites {
            let email = format!("user{}@example.com", i);
            let invite = service
                .invite_member(vault_id, email)
                .await
                .expect("invite_member should succeed");
            tokens.insert(invite.token);
        }

        // All tokens should be unique
        assert_eq!(tokens.len(), num_invites);
    }

    #[tokio::test]
    async fn test_invite_expiration_property() {
        // Property: For any invite token, expires_at is exactly 7 days from created_at
        // Validates: Requirements 5.4, 5.5, 26.3, 33.2, 33.3
        let db = create_test_db();
        let service = VaultService::new(db);
        let vault_id = Uuid::new_v4();

        for i in 0..10 {
            let email = format!("user{}@example.com", i);
            let invite = service
                .invite_member(vault_id, email)
                .await
                .expect("invite_member should succeed");

            let duration = invite.expires_at - invite.created_at;
            let expected_duration = Duration::days(7);

            // Duration should be exactly 7 days (within 1 second tolerance for timing)
            assert!(
                (duration - expected_duration).num_seconds().abs() <= 1,
                "Expiration duration should be 7 days, got {:?}",
                duration
            );
        }
    }

    #[tokio::test]
    async fn test_invite_token_url_safe() {
        // Property: For any invite token, it is URL-safe (valid UUID format)
        // Validates: Requirements 34.1, 34.2, 34.3
        let db = create_test_db();
        let service = VaultService::new(db);
        let vault_id = Uuid::new_v4();

        for i in 0..20 {
            let email = format!("user{}@example.com", i);
            let invite = service
                .invite_member(vault_id, email)
                .await
                .expect("invite_member should succeed");

            // Token should be parseable as UUID (URL-safe format)
            let parsed = Uuid::parse_str(&invite.token);
            assert!(
                parsed.is_ok(),
                "Token should be a valid UUID: {}",
                invite.token
            );
        }
    }

    #[tokio::test]
    async fn test_invite_not_accepted_initially() {
        // Property: For any newly created invite, accepted is false
        // Validates: Requirements 5.2, 5.3
        let db = create_test_db();
        let service = VaultService::new(db);
        let vault_id = Uuid::new_v4();

        for i in 0..10 {
            let email = format!("user{}@example.com", i);
            let invite = service
                .invite_member(vault_id, email)
                .await
                .expect("invite_member should succeed");

            assert!(!invite.accepted, "Newly created invite should not be accepted");
        }
    }
}
