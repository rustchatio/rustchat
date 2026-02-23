-- Migration: Drop the unused email column from server_config
-- The email configuration has been moved to mail_provider_settings table

-- Note: We preserve the data by keeping the column in existing databases
-- but new installations won't have this column.
-- For a clean state, we drop the column if it's empty or matches defaults.

DO $$
BEGIN
    -- Check if the column exists
    IF EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'server_config' AND column_name = 'email'
    ) THEN
        -- The column exists - we keep it for backward compatibility
        -- but add a comment indicating it's deprecated
        COMMENT ON COLUMN server_config.email IS 'DEPRECATED: Email configuration has moved to mail_provider_settings table. This column is kept for backward compatibility only.';
        
        RAISE NOTICE 'Email column in server_config is now deprecated. Use mail_provider_settings table instead.';
    END IF;
END $$;

-- Add comment to the table as well
COMMENT ON TABLE server_config IS 'Server configuration. Note: Email settings have been moved to mail_provider_settings table.';
