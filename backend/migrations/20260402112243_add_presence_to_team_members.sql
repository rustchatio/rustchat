-- Add presence column to team_members table
-- This column tracks the online/presence status of team members

ALTER TABLE team_members
    ADD COLUMN IF NOT EXISTS presence VARCHAR(20) NOT NULL DEFAULT 'offline';

-- Create index for presence lookups
CREATE INDEX IF NOT EXISTS idx_team_members_presence ON team_members(presence);
