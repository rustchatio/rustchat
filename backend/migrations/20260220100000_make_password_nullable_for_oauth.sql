-- Make password_hash nullable to support OAuth users
-- Following best practices from Mattermost and GitLab

-- Step 1: Make password_hash nullable
ALTER TABLE users ALTER COLUMN password_hash DROP NOT NULL;

-- Step 2: Add constraint to ensure local users (non-OAuth) have passwords
-- This maintains data integrity while allowing OAuth users
CREATE OR REPLACE FUNCTION check_local_user_password()
RETURNS TRIGGER AS $$
BEGIN
    -- If auth_provider is NULL (local user) and password_hash is NULL/empty, reject
    IF NEW.auth_provider IS NULL AND (NEW.password_hash IS NULL OR NEW.password_hash = '') THEN
        RAISE EXCEPTION 'Local users must have a password';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for the constraint
DROP TRIGGER IF EXISTS trg_check_local_user_password ON users;
CREATE TRIGGER trg_check_local_user_password
    BEFORE INSERT OR UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION check_local_user_password();

-- Step 3: Add comment explaining the constraint
COMMENT ON FUNCTION check_local_user_password() IS 'Ensures local users (non-OAuth) have a password_hash, while allowing OAuth users to have NULL passwords';
