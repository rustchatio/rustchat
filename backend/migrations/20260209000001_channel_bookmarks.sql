-- Channel Bookmarks table for Mattermost mobile compatibility
-- Supports both link and file bookmark types

CREATE TABLE IF NOT EXISTS channel_bookmarks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    create_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW()) * 1000)::BIGINT,
    update_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW()) * 1000)::BIGINT,
    delete_at BIGINT NOT NULL DEFAULT 0,
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
    parent_id UUID
);

-- Index for fetching bookmarks by channel
CREATE INDEX IF NOT EXISTS idx_channel_bookmarks_channel_id ON channel_bookmarks(channel_id);

-- Index for fetching bookmarks since a timestamp
CREATE INDEX IF NOT EXISTS idx_channel_bookmarks_update_at ON channel_bookmarks(update_at);

-- Index for soft delete queries
CREATE INDEX IF NOT EXISTS idx_channel_bookmarks_delete_at ON channel_bookmarks(delete_at);
