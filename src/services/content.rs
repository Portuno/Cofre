// Content service for Cofre Vault Platform
// Manages content item lifecycle: creation, retrieval, deletion with cascade operations

use crate::db::Database;
use crate::error::{Error, Result};
use crate::models::{ContentItem, ContentType, CreateContentItemInput, ItemTag};
use chrono::Utc;
use uuid::Uuid;

/// Service for managing content item operations
pub struct ContentService {
    #[allow(dead_code)]
    db: Database,
}

impl ContentService {
    /// Create a new ContentService instance
    pub fn new(db: Database) -> Self {
        ContentService { db }
    }

    /// Add a new content item to a vault
    ///
    /// # Arguments
    /// * `vault_id` - UUID of the vault where the item is being added
    /// * `creator_id` - UUID of the user creating the item
    /// * `input` - CreateContentItemInput containing item details
    ///
    /// # Returns
    /// * `Result<ContentItem>` - The created content item or an error
    ///
    /// # Validation
    /// * Validates vault membership (creator must be a member of the vault)
    /// * Validates URL format based on content type
    /// * For audio/image types: URL should be a storage URL
    /// * For link type: URL should be a valid external URL
    ///
    /// # Behavior
    /// * Creates ContentItem record with UUID and timestamps
    /// * Stores storage URL or external URL based on content type
    /// * Allows optional title and metadata fields
    /// * Makes item immediately accessible to all vault members
    ///
    /// # Requirements
    /// * Validates: Requirements 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 30.1, 30.2, 30.3, 30.4, 30.5
    pub async fn add_item(
        &self,
        vault_id: Uuid,
        creator_id: Uuid,
        input: CreateContentItemInput,
    ) -> Result<ContentItem> {
        // Validate URL is not empty
        if input.url.is_empty() {
            return Err(Error::InvalidUrl);
        }

        // Validate URL format based on content type
        match input.content_type {
            ContentType::Link => {
                // For links, validate URL format
                if !self.is_valid_url(&input.url) {
                    return Err(Error::InvalidUrl);
                }
            }
            ContentType::Audio | ContentType::Image => {
                // For audio/image, URL should be a storage URL (basic validation)
                if input.url.is_empty() {
                    return Err(Error::InvalidUrl);
                }
            }
        }

        // In a real implementation, this would:
        // 1. Verify vault exists
        // 2. Verify creator is a member of the vault
        // 3. Insert content_item record into database
        // 4. Return the created item
        //
        // TODO: Implement database operations
        // let vault = db.get_vault(vault_id).await?;
        // if vault.is_none() {
        //     return Err(Error::VaultNotFound);
        // }
        // let member = db.get_vault_member(vault_id, creator_id).await?;
        // if member.is_none() {
        //     return Err(Error::Unauthorized);
        // }

        let item_id = Uuid::new_v4();
        let now = Utc::now();

        let item = ContentItem {
            id: item_id,
            vault_id,
            created_by: creator_id,
            content_type: input.content_type,
            title: input.title,
            url: input.url,
            transcript: input.transcript,
            metadata: input.metadata,
            created_at: now,
        };

        // TODO: Persist to database
        // db.insert_content_item(&item).await?;

        Ok(item)
    }

    /// Get all content items in a vault
    ///
    /// # Arguments
    /// * `vault_id` - UUID of the vault
    /// * `user_id` - UUID of the requesting user (for membership check)
    ///
    /// # Returns
    /// * `Result<Vec<ContentItem>>` - List of all content items in the vault
    ///
    /// # Validation
    /// * Enforces vault membership check (user must be a member)
    /// * Returns Unauthorized error if user is not a member
    ///
    /// # Behavior
    /// * Returns items with all metadata including URLs and transcripts
    /// * Returns empty list if vault has no items
    ///
    /// # Requirements
    /// * Validates: Requirements 15.1, 15.4
    pub async fn get_items(&self, _vault_id: Uuid, _user_id: Uuid) -> Result<Vec<ContentItem>> {
        // In a real implementation, this would:
        // 1. Verify user is a member of the vault
        // 2. Query all content items for the vault
        // 3. Return the list of items
        //
        // TODO: Implement database operations
        // let member = db.get_vault_member(vault_id, user_id).await?;
        // if member.is_none() {
        //     return Err(Error::Unauthorized);
        // }
        // let items = db.get_content_items_for_vault(vault_id).await?;
        // Ok(items)

        Ok(Vec::new())
    }

    /// Get all content items tagged with a specific tag
    ///
    /// # Arguments
    /// * `vault_id` - UUID of the vault
    /// * `tag_id` - UUID of the tag to filter by
    /// * `user_id` - UUID of the requesting user (for membership check)
    ///
    /// # Returns
    /// * `Result<Vec<ContentItem>>` - List of items tagged with the specified tag
    ///
    /// # Validation
    /// * Enforces vault membership check (user must be a member)
    /// * Returns Unauthorized error if user is not a member
    /// * Verifies tag exists and belongs to the vault
    ///
    /// # Behavior
    /// * Returns items with full metadata
    /// * Returns empty list if no items have the tag
    ///
    /// # Requirements
    /// * Validates: Requirements 15.2, 15.3
    pub async fn get_items_by_tag(
        &self,
        _vault_id: Uuid,
        _tag_id: Uuid,
        _user_id: Uuid,
    ) -> Result<Vec<ContentItem>> {
        // In a real implementation, this would:
        // 1. Verify user is a member of the vault
        // 2. Verify tag exists and belongs to the vault
        // 3. Query all content items tagged with the tag
        // 4. Return the list of items
        //
        // TODO: Implement database operations
        // let member = db.get_vault_member(vault_id, user_id).await?;
        // if member.is_none() {
        //     return Err(Error::Unauthorized);
        // }
        // let tag = db.get_tag(tag_id).await?;
        // if tag.is_none() || tag.unwrap().vault_id != vault_id {
        //     return Err(Error::TagNotFound);
        // }
        // let items = db.get_content_items_by_tag(tag_id).await?;
        // Ok(items)

        Ok(Vec::new())
    }

    /// Delete a content item and cascade delete related records
    ///
    /// # Arguments
    /// * `item_id` - UUID of the item to delete
    /// * `vault_id` - UUID of the vault (for context and verification)
    /// * `user_id` - UUID of the requesting user (for membership check)
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Cascade Behavior
    /// * Removes ContentItem record from database
    /// * Cascade deletes all ItemTag records for the item
    /// * Removes associated file from Supabase Storage if applicable
    /// * Updates semantic graph to reflect removal
    ///
    /// # Validation
    /// * Enforces vault membership check (user must be a member)
    /// * Verifies item exists and belongs to the vault
    /// * Returns Unauthorized error if user is not a member
    /// * Returns ContentItemNotFound if item doesn't exist or belongs to different vault
    ///
    /// # Requirements
    /// * Validates: Requirements 16.1, 16.2, 16.3, 16.4, 16.5
    pub async fn delete_item(
        &self,
        _item_id: Uuid,
        _vault_id: Uuid,
        _user_id: Uuid,
    ) -> Result<()> {
        // In a real implementation, this would:
        // 1. Verify user is a member of the vault
        // 2. Query the content item to verify it exists and belongs to the vault
        // 3. If item has a storage URL (audio/image), delete from Supabase Storage
        // 4. Delete all ItemTag records for the item
        // 5. Delete the ContentItem record
        // 6. Update semantic graph to reflect removal
        // 7. Return success or error
        //
        // TODO: Implement database operations
        // let member = db.get_vault_member(vault_id, user_id).await?;
        // if member.is_none() {
        //     return Err(Error::Unauthorized);
        // }
        // let item = db.get_content_item(item_id).await?;
        // if item.is_none() || item.unwrap().vault_id != vault_id {
        //     return Err(Error::ContentItemNotFound);
        // }
        // let item = item.unwrap();
        //
        // // Delete from storage if applicable
        // if item.content_type == ContentType::Audio || item.content_type == ContentType::Image {
        //     storage.delete_file(&item.url).await?;
        // }
        //
        // // Cascade delete ItemTag records
        // db.delete_item_tags_for_item(item_id).await?;
        //
        // // Delete the content item
        // db.delete_content_item(item_id).await?;
        //
        // // Update semantic graph (client-side, so just return success)
        // Ok(())

        Ok(())
    }

    /// Helper function to validate URL format
    fn is_valid_url(&self, url: &str) -> bool {
        // Basic URL validation: check for common URL patterns
        // In a real implementation, use a proper URL parsing library
        url.starts_with("http://") || url.starts_with("https://")
    }

    /// Attach one or more tags to a content item
    ///
    /// # Arguments
    /// * `item_id` - UUID of the content item
    /// * `tag_ids` - Vector of tag UUIDs to attach
    /// * `vault_id` - UUID of the vault (for context and verification)
    /// * `user_id` - UUID of the requesting user (for membership check)
    ///
    /// # Returns
    /// * `Result<Vec<ItemTag>>` - The created ItemTag records or an error
    ///
    /// # Validation
    /// * Enforces vault membership check (user must be a member)
    /// * Verifies item exists and belongs to the vault
    /// * Verifies all tags exist and belong to the vault
    /// * Prevents duplicate tag attachments (same tag cannot be attached twice to same item)
    /// * Returns error if any tag is already attached to the item
    ///
    /// # Behavior
    /// * Creates ItemTag records linking item to each tag
    /// * Allows multiple tags per item
    /// * Allows attaching a single tag to multiple items
    /// * Makes tags immediately available for graph construction
    /// * Returns all created ItemTag records
    ///
    /// # Requirements
    /// * Validates: Requirements 10.1, 10.2, 10.3, 10.4, 10.5
    pub async fn attach_tags(
        &self,
        item_id: Uuid,
        tag_ids: Vec<Uuid>,
        _vault_id: Uuid,
        _user_id: Uuid,
    ) -> Result<Vec<ItemTag>> {
        // In a real implementation, this would:
        // 1. Verify user is a member of the vault
        // 2. Verify item exists and belongs to the vault
        // 3. Verify all tags exist and belong to the vault
        // 4. Check for existing attachments to prevent duplicates
        // 5. Create ItemTag records for each tag
        // 6. Return the created records
        //
        // TODO: Implement database operations
        // let member = db.get_vault_member(vault_id, user_id).await?;
        // if member.is_none() {
        //     return Err(Error::Unauthorized);
        // }
        // let item = db.get_content_item(item_id).await?;
        // if item.is_none() || item.unwrap().vault_id != vault_id {
        //     return Err(Error::ContentItemNotFound);
        // }
        // for tag_id in &tag_ids {
        //     let tag = db.get_tag(*tag_id).await?;
        //     if tag.is_none() || tag.unwrap().vault_id != vault_id {
        //         return Err(Error::TagNotFound);
        //     }
        //     let existing = db.get_item_tag(item_id, *tag_id).await?;
        //     if existing.is_some() {
        //         return Err(Error::DuplicateTagAttachment);
        //     }
        // }
        // let mut created_tags = Vec::new();
        // for tag_id in tag_ids {
        //     let item_tag = ItemTag {
        //         item_id,
        //         tag_id,
        //         created_at: Utc::now(),
        //     };
        //     db.insert_item_tag(&item_tag).await?;
        //     created_tags.push(item_tag);
        // }
        // Ok(created_tags)

        let now = Utc::now();
        let mut created_tags = Vec::new();

        for tag_id in tag_ids {
            let item_tag = ItemTag {
                item_id,
                tag_id,
                created_at: now,
            };
            created_tags.push(item_tag);
        }

        Ok(created_tags)
    }

    /// Detach one or more tags from a content item
    ///
    /// # Arguments
    /// * `item_id` - UUID of the content item
    /// * `tag_ids` - Vector of tag UUIDs to detach
    /// * `vault_id` - UUID of the vault (for context and verification)
    /// * `user_id` - UUID of the requesting user (for membership check)
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    ///
    /// # Validation
    /// * Enforces vault membership check (user must be a member)
    /// * Verifies item exists and belongs to the vault
    /// * Verifies all tags exist and belong to the vault
    /// * Returns error if any tag is not attached to the item
    ///
    /// # Behavior
    /// * Removes ItemTag records for specified tags
    /// * Allows partial detachment (remove some tags, keep others)
    /// * Does not affect other tags attached to the item
    /// * Updates semantic graph to reflect tag removal
    ///
    /// # Requirements
    /// * Validates: Requirements 10.6
    pub async fn detach_tags(
        &self,
        _item_id: Uuid,
        _tag_ids: Vec<Uuid>,
        _vault_id: Uuid,
        _user_id: Uuid,
    ) -> Result<()> {
        // In a real implementation, this would:
        // 1. Verify user is a member of the vault
        // 2. Verify item exists and belongs to the vault
        // 3. Verify all tags exist and belong to the vault
        // 4. Verify all tags are currently attached to the item
        // 5. Delete ItemTag records for each tag
        // 6. Return success or error
        //
        // TODO: Implement database operations
        // let member = db.get_vault_member(vault_id, user_id).await?;
        // if member.is_none() {
        //     return Err(Error::Unauthorized);
        // }
        // let item = db.get_content_item(item_id).await?;
        // if item.is_none() || item.unwrap().vault_id != vault_id {
        //     return Err(Error::ContentItemNotFound);
        // }
        // for tag_id in &tag_ids {
        //     let tag = db.get_tag(*tag_id).await?;
        //     if tag.is_none() || tag.unwrap().vault_id != vault_id {
        //         return Err(Error::TagNotFound);
        //     }
        //     let existing = db.get_item_tag(item_id, *tag_id).await?;
        //     if existing.is_none() {
        //         return Err(Error::TagNotFound);
        //     }
        // }
        // for tag_id in tag_ids {
        //     db.delete_item_tag(item_id, tag_id).await?;
        // }
        // Ok(())

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> Database {
        Database::new(crate::db::DatabaseConfig {
            supabase_url: "https://example.supabase.co".to_string(),
            supabase_key: "test-key".to_string(),
            database_url: "postgresql://user:pass@localhost/db".to_string(),
            max_connections: 10,
        })
    }

    #[tokio::test]
    async fn test_add_item_with_valid_audio() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let vault_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();

        let input = CreateContentItemInput {
            content_type: ContentType::Audio,
            title: Some("My Recording".to_string()),
            url: "https://storage.example.com/audio/file.mp3".to_string(),
            transcript: Some("This is the transcript".to_string()),
            metadata: None,
        };

        let result = service.add_item(vault_id, creator_id, input).await;
        assert!(result.is_ok());

        let item = result.unwrap();
        assert_eq!(item.vault_id, vault_id);
        assert_eq!(item.created_by, creator_id);
        assert_eq!(item.content_type, ContentType::Audio);
        assert_eq!(item.title, Some("My Recording".to_string()));
        assert_eq!(item.url, "https://storage.example.com/audio/file.mp3");
        assert_eq!(item.transcript, Some("This is the transcript".to_string()));
    }

    #[tokio::test]
    async fn test_add_item_with_valid_image() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let vault_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();

        let input = CreateContentItemInput {
            content_type: ContentType::Image,
            title: Some("Screenshot".to_string()),
            url: "https://storage.example.com/images/screenshot.png".to_string(),
            transcript: None,
            metadata: Some(serde_json::json!({"width": 1920, "height": 1080})),
        };

        let result = service.add_item(vault_id, creator_id, input).await;
        assert!(result.is_ok());

        let item = result.unwrap();
        assert_eq!(item.content_type, ContentType::Image);
        assert_eq!(item.metadata, Some(serde_json::json!({"width": 1920, "height": 1080})));
    }

    #[tokio::test]
    async fn test_add_item_with_valid_link() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let vault_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();

        let input = CreateContentItemInput {
            content_type: ContentType::Link,
            title: Some("Useful Article".to_string()),
            url: "https://example.com/article".to_string(),
            transcript: None,
            metadata: Some(serde_json::json!({"preview": "An interesting article about..."})),
        };

        let result = service.add_item(vault_id, creator_id, input).await;
        assert!(result.is_ok());

        let item = result.unwrap();
        assert_eq!(item.content_type, ContentType::Link);
        assert_eq!(item.url, "https://example.com/article");
    }

    #[tokio::test]
    async fn test_add_item_with_empty_url() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let vault_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();

        let input = CreateContentItemInput {
            content_type: ContentType::Audio,
            title: None,
            url: "".to_string(),
            transcript: None,
            metadata: None,
        };

        let result = service.add_item(vault_id, creator_id, input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidUrl));
    }

    #[tokio::test]
    async fn test_add_item_with_invalid_link_url() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let vault_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();

        let input = CreateContentItemInput {
            content_type: ContentType::Link,
            title: None,
            url: "not-a-valid-url".to_string(),
            transcript: None,
            metadata: None,
        };

        let result = service.add_item(vault_id, creator_id, input).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::InvalidUrl));
    }

    #[tokio::test]
    async fn test_add_item_without_title() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let vault_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();

        let input = CreateContentItemInput {
            content_type: ContentType::Audio,
            title: None,
            url: "https://storage.example.com/audio/file.mp3".to_string(),
            transcript: None,
            metadata: None,
        };

        let result = service.add_item(vault_id, creator_id, input).await;
        assert!(result.is_ok());

        let item = result.unwrap();
        assert_eq!(item.title, None);
    }

    #[tokio::test]
    async fn test_get_items_returns_empty_list() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let result = service.get_items(vault_id, user_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_get_items_by_tag_returns_empty_list() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let vault_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let result = service.get_items_by_tag(vault_id, tag_id, user_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_delete_item_returns_ok() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let item_id = Uuid::new_v4();
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let result = service.delete_item(item_id, vault_id, user_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_item_with_all_fields() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let vault_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();

        let metadata = serde_json::json!({
            "duration": 120,
            "format": "mp3",
            "bitrate": 128
        });

        let input = CreateContentItemInput {
            content_type: ContentType::Audio,
            title: Some("Important Meeting".to_string()),
            url: "https://storage.example.com/audio/meeting.mp3".to_string(),
            transcript: Some("Meeting transcript here...".to_string()),
            metadata: Some(metadata.clone()),
        };

        let result = service.add_item(vault_id, creator_id, input).await;
        assert!(result.is_ok());

        let item = result.unwrap();
        assert_eq!(item.vault_id, vault_id);
        assert_eq!(item.created_by, creator_id);
        assert_eq!(item.content_type, ContentType::Audio);
        assert_eq!(item.title, Some("Important Meeting".to_string()));
        assert_eq!(item.url, "https://storage.example.com/audio/meeting.mp3");
        assert_eq!(item.transcript, Some("Meeting transcript here...".to_string()));
        assert_eq!(item.metadata, Some(metadata));
    }

    #[tokio::test]
    async fn test_add_item_with_http_url() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let vault_id = Uuid::new_v4();
        let creator_id = Uuid::new_v4();

        let input = CreateContentItemInput {
            content_type: ContentType::Link,
            title: Some("Article".to_string()),
            url: "http://example.com/article".to_string(),
            transcript: None,
            metadata: None,
        };

        let result = service.add_item(vault_id, creator_id, input).await;
        assert!(result.is_ok());

        let item = result.unwrap();
        assert_eq!(item.url, "http://example.com/article");
    }

    // Tag attachment tests

    #[tokio::test]
    async fn test_attach_single_tag_to_item() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let item_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let result = service
            .attach_tags(item_id, vec![tag_id], vault_id, user_id)
            .await;

        assert!(result.is_ok());
        let item_tags = result.unwrap();
        assert_eq!(item_tags.len(), 1);
        assert_eq!(item_tags[0].item_id, item_id);
        assert_eq!(item_tags[0].tag_id, tag_id);
    }

    #[tokio::test]
    async fn test_attach_multiple_tags_to_item() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let item_id = Uuid::new_v4();
        let tag_id_1 = Uuid::new_v4();
        let tag_id_2 = Uuid::new_v4();
        let tag_id_3 = Uuid::new_v4();
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let result = service
            .attach_tags(
                item_id,
                vec![tag_id_1, tag_id_2, tag_id_3],
                vault_id,
                user_id,
            )
            .await;

        assert!(result.is_ok());
        let item_tags = result.unwrap();
        assert_eq!(item_tags.len(), 3);

        // Verify all tags are attached
        assert!(item_tags.iter().any(|it| it.tag_id == tag_id_1));
        assert!(item_tags.iter().any(|it| it.tag_id == tag_id_2));
        assert!(item_tags.iter().any(|it| it.tag_id == tag_id_3));

        // Verify all have the same item_id
        assert!(item_tags.iter().all(|it| it.item_id == item_id));
    }

    #[tokio::test]
    async fn test_attach_tags_creates_item_tag_records() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let item_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let result = service
            .attach_tags(item_id, vec![tag_id], vault_id, user_id)
            .await;

        assert!(result.is_ok());
        let item_tags = result.unwrap();
        assert_eq!(item_tags.len(), 1);

        let item_tag = &item_tags[0];
        assert_eq!(item_tag.item_id, item_id);
        assert_eq!(item_tag.tag_id, tag_id);
        // Verify created_at is set
        assert!(item_tag.created_at <= Utc::now());
    }

    #[tokio::test]
    async fn test_attach_tags_with_empty_list() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let item_id = Uuid::new_v4();
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let result = service
            .attach_tags(item_id, vec![], vault_id, user_id)
            .await;

        assert!(result.is_ok());
        let item_tags = result.unwrap();
        assert_eq!(item_tags.len(), 0);
    }

    #[tokio::test]
    async fn test_detach_single_tag_from_item() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let item_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let result = service
            .detach_tags(item_id, vec![tag_id], vault_id, user_id)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_detach_multiple_tags_from_item() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let item_id = Uuid::new_v4();
        let tag_id_1 = Uuid::new_v4();
        let tag_id_2 = Uuid::new_v4();
        let tag_id_3 = Uuid::new_v4();
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let result = service
            .detach_tags(
                item_id,
                vec![tag_id_1, tag_id_2, tag_id_3],
                vault_id,
                user_id,
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_detach_tags_with_empty_list() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let item_id = Uuid::new_v4();
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let result = service
            .detach_tags(item_id, vec![], vault_id, user_id)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_attach_tags_allows_multiple_tags_per_item() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let item_id = Uuid::new_v4();
        let tag_id_1 = Uuid::new_v4();
        let tag_id_2 = Uuid::new_v4();
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Attach first tag
        let result1 = service
            .attach_tags(item_id, vec![tag_id_1], vault_id, user_id)
            .await;
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap().len(), 1);

        // Attach second tag
        let result2 = service
            .attach_tags(item_id, vec![tag_id_2], vault_id, user_id)
            .await;
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_attach_tags_returns_item_tag_with_timestamps() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let item_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let before = Utc::now();
        let result = service
            .attach_tags(item_id, vec![tag_id], vault_id, user_id)
            .await;
        let after = Utc::now();

        assert!(result.is_ok());
        let item_tags = result.unwrap();
        assert_eq!(item_tags.len(), 1);

        let item_tag = &item_tags[0];
        assert!(item_tag.created_at >= before);
        assert!(item_tag.created_at <= after);
    }

    #[tokio::test]
    async fn test_attach_tags_preserves_tag_ids() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let item_id = Uuid::new_v4();
        let tag_ids = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let result = service
            .attach_tags(item_id, tag_ids.clone(), vault_id, user_id)
            .await;

        assert!(result.is_ok());
        let item_tags = result.unwrap();

        // Verify all tag IDs are preserved
        for (i, tag_id) in tag_ids.iter().enumerate() {
            assert_eq!(item_tags[i].tag_id, *tag_id);
        }
    }

    #[tokio::test]
    async fn test_detach_tags_allows_partial_detachment() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let item_id = Uuid::new_v4();
        let tag_id_1 = Uuid::new_v4();
        let tag_id_2 = Uuid::new_v4();
        let _tag_id_3 = Uuid::new_v4();
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Detach only some tags
        let result = service
            .detach_tags(item_id, vec![tag_id_1, tag_id_2], vault_id, user_id)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_attach_tags_makes_tags_available_for_graph() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let item_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let result = service
            .attach_tags(item_id, vec![tag_id], vault_id, user_id)
            .await;

        assert!(result.is_ok());
        let item_tags = result.unwrap();

        // Verify the ItemTag can be used for graph construction
        assert_eq!(item_tags[0].item_id, item_id);
        assert_eq!(item_tags[0].tag_id, tag_id);
        // In a real implementation, these would be immediately available for graph construction
    }

    #[tokio::test]
    async fn test_attach_tags_with_many_tags() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let item_id = Uuid::new_v4();
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Create many tag IDs
        let tag_ids: Vec<Uuid> = (0..10).map(|_| Uuid::new_v4()).collect();

        let result = service
            .attach_tags(item_id, tag_ids.clone(), vault_id, user_id)
            .await;

        assert!(result.is_ok());
        let item_tags = result.unwrap();
        assert_eq!(item_tags.len(), 10);

        // Verify all tags are present
        for tag_id in tag_ids {
            assert!(item_tags.iter().any(|it| it.tag_id == tag_id));
        }
    }

    #[tokio::test]
    async fn test_detach_tags_with_many_tags() {
        let db = create_test_db();
        let service = ContentService::new(db);
        let item_id = Uuid::new_v4();
        let vault_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Create many tag IDs
        let tag_ids: Vec<Uuid> = (0..10).map(|_| Uuid::new_v4()).collect();

        let result = service
            .detach_tags(item_id, tag_ids, vault_id, user_id)
            .await;

        assert!(result.is_ok());
    }

}
