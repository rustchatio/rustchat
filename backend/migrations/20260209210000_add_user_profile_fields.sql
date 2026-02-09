-- Add profile fields to users table for Mattermost mobile compatibility
-- These fields are displayed in the mobile profile screen

ALTER TABLE users ADD COLUMN IF NOT EXISTS first_name VARCHAR(64);
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_name VARCHAR(64);
ALTER TABLE users ADD COLUMN IF NOT EXISTS nickname VARCHAR(64);
ALTER TABLE users ADD COLUMN IF NOT EXISTS position VARCHAR(128);
