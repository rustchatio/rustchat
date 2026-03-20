-- Add API key prefix column for O(1) authentication lookups
ALTER TABLE users ADD COLUMN api_key_prefix VARCHAR(16);

-- Create unique index for fast prefix lookups
-- UNIQUE constraint prevents accidental collisions (database-enforced)
-- Partial index (WHERE NOT NULL) keeps index small and efficient
CREATE UNIQUE INDEX idx_users_api_key_prefix
  ON users(api_key_prefix)
  WHERE api_key_prefix IS NOT NULL;

-- Back up existing API key hashes before clearing (for manual emergency recovery only)
-- Note: TEMP table exists only during this migration transaction
-- For actual rollback: restore from pre-deployment database backup
CREATE TEMP TABLE api_key_backup AS
SELECT id, api_key_hash
FROM users
WHERE api_key_hash IS NOT NULL
  AND entity_type IN ('agent', 'service', 'ci');

-- Mark existing API keys as invalid by clearing their hashes
-- Forces agents to regenerate keys with new format
UPDATE users
SET api_key_hash = NULL
WHERE api_key_hash IS NOT NULL
  AND entity_type IN ('agent', 'service', 'ci');

-- Document the change
COMMENT ON COLUMN users.api_key_prefix IS 'First 16 chars of API key (rck_XXXXXXXXXXXX) for fast O(1) lookups';
