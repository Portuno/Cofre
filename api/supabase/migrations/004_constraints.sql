-- Add additional constraints and triggers for data integrity

-- Update timestamps on vault updates
CREATE OR REPLACE FUNCTION update_vault_timestamp()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = CURRENT_TIMESTAMP;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER vault_update_timestamp
BEFORE UPDATE ON vaults
FOR EACH ROW
EXECUTE FUNCTION update_vault_timestamp();

-- Update timestamps on content_items updates
CREATE OR REPLACE FUNCTION update_content_item_timestamp()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = CURRENT_TIMESTAMP;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER content_item_update_timestamp
BEFORE UPDATE ON content_items
FOR EACH ROW
EXECUTE FUNCTION update_content_item_timestamp();

-- Ensure vault_members has at least one owner
CREATE OR REPLACE FUNCTION ensure_vault_has_owner()
RETURNS TRIGGER AS $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM vault_members 
    WHERE vault_id = NEW.vault_id AND role = 'owner'
  ) THEN
    RAISE EXCEPTION 'Vault must have at least one owner';
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER vault_members_ensure_owner
AFTER DELETE ON vault_members
FOR EACH ROW
WHEN (OLD.role = 'owner')
EXECUTE FUNCTION ensure_vault_has_owner();
