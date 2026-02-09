-- Custom Profile Attributes tables for Mattermost mobile compatibility

-- Field definitions (schema/types of custom attributes)
CREATE TABLE IF NOT EXISTS custom_profile_fields (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    group_id VARCHAR(26) NOT NULL DEFAULT '',
    name VARCHAR(64) NOT NULL,
    field_type VARCHAR(32) NOT NULL DEFAULT 'text',
    attrs JSONB NOT NULL DEFAULT '{}',
    target_id VARCHAR(26) NOT NULL DEFAULT '',
    target_type VARCHAR(32) NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Ensure deleted_at exists if table was created by a previous version of this migration
ALTER TABLE custom_profile_fields ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- User attribute values (actual values per user)
CREATE TABLE IF NOT EXISTS custom_profile_attributes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    field_id UUID NOT NULL REFERENCES custom_profile_fields(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    value TEXT NOT NULL DEFAULT '',
    UNIQUE(field_id, user_id)
);

-- Index for looking up a user's attributes
CREATE INDEX IF NOT EXISTS idx_custom_profile_attributes_user_id 
ON custom_profile_attributes(user_id);

-- Index for looking up all values of a specific field
CREATE INDEX IF NOT EXISTS idx_custom_profile_attributes_field_id 
ON custom_profile_attributes(field_id);

-- Trigger for updated_at
CREATE TRIGGER update_custom_profile_fields_updated_at
    BEFORE UPDATE ON custom_profile_fields
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
