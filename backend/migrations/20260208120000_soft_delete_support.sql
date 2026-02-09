-- Add soft-delete support to core tables
-- Migration: 009_soft_delete_support

-- Add deleted_at to channels (used in code, missing in initial)
ALTER TABLE channels ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
CREATE INDEX IF NOT EXISTS idx_channels_deleted_at ON channels(deleted_at) WHERE deleted_at IS NULL;

-- Add deleted_at to organizations
ALTER TABLE organizations ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
CREATE INDEX IF NOT EXISTS idx_organizations_deleted_at ON organizations(deleted_at) WHERE deleted_at IS NULL;

-- Add deleted_at to users
ALTER TABLE users ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
CREATE INDEX IF NOT EXISTS idx_users_deleted_at ON users(deleted_at) WHERE deleted_at IS NULL;

-- Add deleted_at to teams
ALTER TABLE teams ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
CREATE INDEX IF NOT EXISTS idx_teams_deleted_at ON teams(deleted_at) WHERE deleted_at IS NULL;
