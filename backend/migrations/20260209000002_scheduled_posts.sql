-- Scheduled Posts table for Mattermost mobile compatibility
-- Migration: 20260209000002_scheduled_posts.sql

CREATE TABLE IF NOT EXISTS scheduled_posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    create_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW()) * 1000)::BIGINT,
    update_at BIGINT NOT NULL DEFAULT (EXTRACT(EPOCH FROM NOW()) * 1000)::BIGINT,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    root_id UUID REFERENCES posts(id) ON DELETE SET NULL,
    message TEXT NOT NULL DEFAULT '',
    props JSONB NOT NULL DEFAULT '{}',
    file_ids UUID[] NOT NULL DEFAULT '{}',
    priority JSONB,
    scheduled_at BIGINT NOT NULL,
    processed_at BIGINT NOT NULL DEFAULT 0,
    error_code VARCHAR(50) NOT NULL DEFAULT ''
);

-- Index for querying scheduled posts by user
CREATE INDEX IF NOT EXISTS idx_scheduled_posts_user_id ON scheduled_posts(user_id);

-- Index for querying scheduled posts by channel
CREATE INDEX IF NOT EXISTS idx_scheduled_posts_channel_id ON scheduled_posts(channel_id);

-- Index for finding pending scheduled posts (not yet processed)
CREATE INDEX IF NOT EXISTS idx_scheduled_posts_pending ON scheduled_posts(scheduled_at) 
    WHERE processed_at = 0;

-- Index for querying by team (via channel)
CREATE INDEX IF NOT EXISTS idx_scheduled_posts_team
ON scheduled_posts(channel_id, scheduled_at) WHERE processed_at = 0;
