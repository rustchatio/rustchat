-- Migration: Copy existing SMTP settings from server_config to mail_provider_settings
-- This migrates the legacy email configuration to the new provider-based system

-- Only run if there's an existing SMTP configuration with a host set
DO $$
DECLARE
    legacy_config RECORD;
    provider_exists BOOLEAN;
    new_provider_id UUID;
BEGIN
    -- Get the legacy email config from server_config
    SELECT 
        email->>'smtp_host' as smtp_host,
        COALESCE((email->>'smtp_port')::int, 587) as smtp_port,
        email->>'smtp_username' as smtp_username,
        email->>'smtp_password_encrypted' as smtp_password,
        COALESCE(email->>'smtp_security', 'starttls') as smtp_security,
        COALESCE((email->>'smtp_skip_cert_verify')::boolean, false) as skip_cert_verify,
        email->>'from_address' as from_address,
        COALESCE(email->>'from_name', 'RustChat') as from_name,
        email->>'reply_to' as reply_to,
        COALESCE((email->>'send_email_notifications')::boolean, true) as send_email_notifications
    INTO legacy_config
    FROM server_config
    WHERE id = 'default';

    -- Check if we have a valid SMTP host in legacy config
    IF legacy_config IS NOT NULL AND legacy_config.smtp_host IS NOT NULL AND legacy_config.smtp_host <> '' THEN
        -- Check if a default provider already exists
        SELECT EXISTS(
            SELECT 1 FROM mail_provider_settings 
            WHERE is_default = true AND (tenant_id IS NULL)
        ) INTO provider_exists;

        -- Only create provider if one doesn't already exist
        IF NOT provider_exists THEN
            -- Map smtp_security to tls_mode
            -- legacy: "tls" = implicit TLS, "starttls" = STARTTLS, "none" = no encryption
            -- new: "implicit_tls", "starttls", "none"
            
            INSERT INTO mail_provider_settings (
                id,
                tenant_id,
                provider_type,
                host,
                port,
                username,
                password_encrypted,
                tls_mode,
                skip_cert_verify,
                from_address,
                from_name,
                reply_to,
                enabled,
                is_default,
                max_emails_per_minute,
                max_emails_per_hour,
                created_at,
                updated_at
            ) VALUES (
                '00000000-0000-0000-0000-000000000002'::uuid, -- Use a different ID than the seed
                NULL, -- Global provider (no tenant)
                'smtp',
                legacy_config.smtp_host,
                legacy_config.smtp_port,
                COALESCE(legacy_config.smtp_username, ''),
                COALESCE(legacy_config.smtp_password, ''),
                CASE 
                    WHEN legacy_config.smtp_security = 'tls' THEN 'implicit_tls'::varchar
                    WHEN legacy_config.smtp_security = 'none' THEN 'none'::varchar
                    ELSE 'starttls'::varchar -- default
                END,
                legacy_config.skip_cert_verify,
                COALESCE(legacy_config.from_address, ''),
                COALESCE(legacy_config.from_name, 'RustChat'),
                NULLIF(legacy_config.reply_to, ''),
                legacy_config.send_email_notifications,
                true, -- is_default
                60,   -- max_emails_per_minute
                1000, -- max_emails_per_hour
                NOW(),
                NOW()
            )
            ON CONFLICT (id) DO UPDATE SET
                host = EXCLUDED.host,
                port = EXCLUDED.port,
                username = EXCLUDED.username,
                password_encrypted = EXCLUDED.password_encrypted,
                tls_mode = EXCLUDED.tls_mode,
                skip_cert_verify = EXCLUDED.skip_cert_verify,
                from_address = EXCLUDED.from_address,
                from_name = EXCLUDED.from_name,
                reply_to = EXCLUDED.reply_to,
                enabled = EXCLUDED.enabled,
                is_default = true,
                updated_at = NOW();
            
            RAISE NOTICE 'Migrated legacy SMTP configuration to mail_provider_settings (host: %)', legacy_config.smtp_host;
        ELSE
            RAISE NOTICE 'Default mail provider already exists, skipping migration of legacy SMTP config';
        END IF;
    ELSE
        RAISE NOTICE 'No legacy SMTP configuration found to migrate';
    END IF;
END $$;

-- Add a comment explaining the migration
COMMENT ON TABLE mail_provider_settings IS 'Email provider settings including migrated legacy SMTP configurations';
