-- Team enhancements: privacy, icon, scheme columns and invitations table
--
-- NOTE: This migration includes forward-compatible schema additions:
-- - user_id is nullable to support email-based invitations (guests/non-users)
-- - email column added for email-based invitation flows
-- - invitation_type distinguishes 'member' vs 'guest' invitations
-- - scheme_id FK intentionally omitted: 'schemes' table does not exist yet.
--   Add FK constraint in a follow-up migration once schemes table is created.

-- Add privacy column to teams
ALTER TABLE teams
    ADD COLUMN IF NOT EXISTS privacy TEXT NOT NULL DEFAULT 'open';

-- Add icon_path column to teams
ALTER TABLE teams
    ADD COLUMN IF NOT EXISTS icon_path TEXT;

-- Add scheme_id column to teams
-- TODO: Add FK constraint REFERENCES schemes(id) when schemes table is created
ALTER TABLE teams
    ADD COLUMN IF NOT EXISTS scheme_id UUID;

-- team_invitations: full invitation records (distinct from one-time invite tokens)
CREATE TABLE IF NOT EXISTS team_invitations (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id        UUID        NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id        UUID        REFERENCES users(id) ON DELETE CASCADE,
    invited_by     UUID        NOT NULL REFERENCES users(id),
    email          TEXT,
    token          TEXT        NOT NULL UNIQUE,
    invitation_type TEXT       NOT NULL DEFAULT 'member',
    expires_at     TIMESTAMPTZ NOT NULL,
    used           BOOLEAN     NOT NULL DEFAULT false,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes on team_invitations
CREATE INDEX IF NOT EXISTS idx_team_invitations_team
    ON team_invitations(team_id);

CREATE INDEX IF NOT EXISTS idx_team_invitations_user
    ON team_invitations(user_id);

CREATE INDEX IF NOT EXISTS idx_team_invitations_token
    ON team_invitations(token);

CREATE INDEX IF NOT EXISTS idx_team_invitations_email
    ON team_invitations(email);

-- Prevent duplicate pending invitations for the same user on the same team
CREATE UNIQUE INDEX IF NOT EXISTS idx_team_invitations_team_user_pending
    ON team_invitations(team_id, user_id)
    WHERE used = false AND user_id IS NOT NULL;

-- Prevent duplicate pending invitations for the same email on the same team
CREATE UNIQUE INDEX IF NOT EXISTS idx_team_invitations_team_email_pending
    ON team_invitations(team_id, email)
    WHERE used = false AND email IS NOT NULL;
