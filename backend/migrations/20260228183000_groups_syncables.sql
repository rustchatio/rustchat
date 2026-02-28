-- Groups + syncables persistence for Mattermost-compatible v4 group-sync endpoints

CREATE TABLE IF NOT EXISTS groups (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(64),
    display_name VARCHAR(128) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    source VARCHAR(64) NOT NULL,
    remote_id VARCHAR(64),
    allow_reference BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_groups_source_remote_id
    ON groups (source, remote_id)
    WHERE remote_id IS NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_groups_name_lower
    ON groups (LOWER(name))
    WHERE name IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_groups_source ON groups (source);
CREATE INDEX IF NOT EXISTS idx_groups_deleted_at ON groups (deleted_at) WHERE deleted_at IS NULL;

CREATE TRIGGER update_groups_updated_at
    BEFORE UPDATE ON groups
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE IF NOT EXISTS group_members (
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (group_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_group_members_user_id ON group_members (user_id);

CREATE TABLE IF NOT EXISTS group_syncables (
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    syncable_type VARCHAR(16) NOT NULL,
    syncable_id UUID NOT NULL,
    auto_add BOOLEAN NOT NULL DEFAULT FALSE,
    scheme_admin BOOLEAN NOT NULL DEFAULT FALSE,
    create_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    update_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    delete_at TIMESTAMPTZ,
    PRIMARY KEY (group_id, syncable_type, syncable_id),
    CONSTRAINT group_syncables_type_check CHECK (syncable_type IN ('team', 'channel'))
);

CREATE INDEX IF NOT EXISTS idx_group_syncables_syncable
    ON group_syncables (syncable_type, syncable_id);

CREATE INDEX IF NOT EXISTS idx_group_syncables_group_id
    ON group_syncables (group_id);

-- Tracks memberships inserted by syncable reconciliation to support safe cleanup on unlink.
CREATE TABLE IF NOT EXISTS group_syncable_memberships (
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    syncable_type VARCHAR(16) NOT NULL,
    syncable_id UUID NOT NULL,
    target_type VARCHAR(16) NOT NULL,
    target_id UUID NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (group_id, syncable_type, syncable_id, target_type, target_id, user_id),
    CONSTRAINT group_syncable_memberships_syncable_type_check CHECK (syncable_type IN ('team', 'channel')),
    CONSTRAINT group_syncable_memberships_target_type_check CHECK (target_type IN ('team', 'channel'))
);

CREATE INDEX IF NOT EXISTS idx_group_syncable_memberships_target_user
    ON group_syncable_memberships (target_type, target_id, user_id);

CREATE INDEX IF NOT EXISTS idx_group_syncable_memberships_syncable
    ON group_syncable_memberships (group_id, syncable_type, syncable_id);
