CREATE TABLE IF NOT EXISTS group_dm_acl_flags (
    group_id UUID PRIMARY KEY REFERENCES groups(id) ON DELETE CASCADE,
    enabled BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_group_dm_acl_flags_enabled
    ON group_dm_acl_flags (enabled)
    WHERE enabled = TRUE;
