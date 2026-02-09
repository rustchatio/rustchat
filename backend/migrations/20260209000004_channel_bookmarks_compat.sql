-- Backfill and normalize channel_bookmarks schema for legacy installs.
-- Older migration 20260121000011 created this table with different column names.

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = 'channel_bookmarks' AND column_name = 'user_id'
    ) AND NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = 'channel_bookmarks' AND column_name = 'owner_id'
    ) THEN
        ALTER TABLE channel_bookmarks RENAME COLUMN user_id TO owner_id;
    END IF;
END
$$;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = 'channel_bookmarks' AND column_name = 'title'
    ) AND NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = 'channel_bookmarks' AND column_name = 'display_name'
    ) THEN
        ALTER TABLE channel_bookmarks RENAME COLUMN title TO display_name;
    END IF;
END
$$;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = 'channel_bookmarks' AND column_name = 'url'
    ) AND NOT EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = 'channel_bookmarks' AND column_name = 'link_url'
    ) THEN
        ALTER TABLE channel_bookmarks RENAME COLUMN url TO link_url;
    END IF;
END
$$;

ALTER TABLE IF EXISTS channel_bookmarks
    ADD COLUMN IF NOT EXISTS file_id UUID REFERENCES files(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS image_url TEXT,
    ADD COLUMN IF NOT EXISTS bookmark_type VARCHAR(16),
    ADD COLUMN IF NOT EXISTS original_id UUID,
    ADD COLUMN IF NOT EXISTS parent_id UUID,
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

UPDATE channel_bookmarks
SET bookmark_type = 'link'
WHERE bookmark_type IS NULL;

UPDATE channel_bookmarks
SET updated_at = COALESCE(updated_at, created_at, NOW())
WHERE updated_at IS NULL;

ALTER TABLE IF EXISTS channel_bookmarks
    ALTER COLUMN sort_order TYPE BIGINT USING sort_order::BIGINT;

CREATE INDEX IF NOT EXISTS idx_channel_bookmarks_deleted_at ON channel_bookmarks(deleted_at);
