-- Create indexes for performance optimization

-- Vault indexes
CREATE INDEX IF NOT EXISTS idx_vaults_created_by ON vaults(created_by);

-- Vault member indexes
CREATE INDEX IF NOT EXISTS idx_vault_members_user_id ON vault_members(user_id);
CREATE INDEX IF NOT EXISTS idx_vault_members_vault_id ON vault_members(vault_id);

-- Content item indexes
CREATE INDEX IF NOT EXISTS idx_content_items_vault_id ON content_items(vault_id);
CREATE INDEX IF NOT EXISTS idx_content_items_created_by ON content_items(created_by);
CREATE INDEX IF NOT EXISTS idx_content_items_content_type ON content_items(content_type);

-- Embedding indexes
CREATE INDEX IF NOT EXISTS idx_embeddings_content_item_id ON embeddings(content_item_id);
CREATE INDEX IF NOT EXISTS idx_embeddings_vector ON embeddings USING ivfflat (embedding vector_cosine_ops);

-- Tag indexes
CREATE INDEX IF NOT EXISTS idx_tags_vault_id ON tags(vault_id);
CREATE INDEX IF NOT EXISTS idx_tags_created_by ON tags(created_by);

-- Item tags indexes
CREATE INDEX IF NOT EXISTS idx_item_tags_tag_id ON item_tags(tag_id);
CREATE INDEX IF NOT EXISTS idx_item_tags_item_id ON item_tags(item_id);

-- Vault invites indexes
CREATE INDEX IF NOT EXISTS idx_vault_invites_vault_id ON vault_invites(vault_id);
CREATE INDEX IF NOT EXISTS idx_vault_invites_token ON vault_invites(token);
CREATE INDEX IF NOT EXISTS idx_vault_invites_expires_at ON vault_invites(expires_at);
