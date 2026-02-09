-- Channel Bookmarks table for Mattermost mobile compatibility
-- Supports both link and file bookmark types

CREATE TABLE IF NOT EXISTS channel_bookmarks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    file_id UUID REFERENCES files(id) ON DELETE SET NULL,
    display_name VARCHAR(256) NOT NULL,
    sort_order BIGINT NOT NULL DEFAULT 0,
    link_url TEXT,
    image_url TEXT,
    emoji VARCHAR(64),
    bookmark_type VARCHAR(16) NOT NULL CHECK (bookmark_type IN ('link', 'file')),
    original_id UUID,
    parent_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Ensure deleted_at exists if table was created by a previous version of this migration
ALTER TABLE channel_bookmarks ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- Index for fetching bookmarks by channel
CREATE INDEX IF NOT EXISTS idx_channel_bookmarks_channel_id ON channel_bookmarks(channel_id);

-- Index for soft delete queries  
CREATE INDEX IF NOT EXISTS idx_channel_bookmarks_deleted_at ON channel_bookmarks(deleted_at);

-- Trigger for updated_at
CREATE TRIGGER update_channel_bookmarks_updated_at
    BEFORE UPDATE ON channel_bookmarks
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
