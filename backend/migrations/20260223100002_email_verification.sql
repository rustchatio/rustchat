-- Email Verification System
-- Adds email verification tokens and updates user registration workflow

-- ============================================
-- A) Email Verification Tokens
-- ============================================

CREATE TABLE IF NOT EXISTS email_verification_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL, -- Hashed token for security
    email VARCHAR(255) NOT NULL, -- The email being verified
    purpose VARCHAR(50) NOT NULL DEFAULT 'registration', -- 'registration', 'email_change', 'password_reset'
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Each user can only have one active token per purpose
    CONSTRAINT unique_active_token UNIQUE (user_id, purpose)
);

CREATE INDEX IF NOT EXISTS idx_email_verification_tokens_hash ON email_verification_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_email_verification_tokens_user ON email_verification_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_email_verification_tokens_expires ON email_verification_tokens(expires_at) WHERE used_at IS NULL;

-- ============================================
-- B) Add email_verified to users
-- ============================================

-- Add email_verified column if not exists
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'email_verified'
    ) THEN
        ALTER TABLE users ADD COLUMN email_verified BOOLEAN NOT NULL DEFAULT false;
    END IF;
END $$;

-- Add email_verified_at timestamp
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'users' AND column_name = 'email_verified_at'
    ) THEN
        ALTER TABLE users ADD COLUMN email_verified_at TIMESTAMPTZ NULL;
    END IF;
END $$;

-- ============================================
-- C) Create default email verification template
-- ============================================

-- Add template family for email verification
INSERT INTO email_template_families (id, key, name, description, workflow_key, is_system) VALUES
    ('00000000-0000-0000-0000-000000000105'::uuid, 'email_verification_default', 'Default Email Verification', 'Standard email verification template', 'email_verification', true)
ON CONFLICT (tenant_id, key) DO NOTHING;

-- Link workflow to template family
UPDATE notification_workflows 
    SET selected_template_family_id = '00000000-0000-0000-0000-000000000105'::uuid 
    WHERE workflow_key = 'email_verification' AND selected_template_family_id IS NULL;

-- Insert default template version for email verification
-- This uses the same basic structure as registration
INSERT INTO email_template_versions (
    family_id, version, status, locale, subject, body_text, body_html,
    variables_schema_json, is_compiled_from_mjml, created_by, published_at, published_by
)
SELECT 
    '00000000-0000-0000-0000-000000000105'::uuid as family_id,
    1 as version,
    'published'::varchar as status,
    'en'::varchar as locale,
    'Verify your email address' as subject,
    E'Hello {{username}},\n\nPlease verify your email address by clicking the link below:\n\n{{verification_link}}\n\nThis link will expire in {{expiry_hours}} hours.\n\nIf you did not create an account, please ignore this email.\n\nBest regards,\n{{site_name}}' as body_text,
    E'<html><body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">\n<h2>Hello {{username}},</h2>\n<p>Please verify your email address by clicking the link below:</p>\n<p style="margin: 20px 0;">\n<a href="{{verification_link}}" style="background: #00FFC2; color: #121213; padding: 12px 24px; text-decoration: none; border-radius: 4px; display: inline-block;">Verify Email Address</a>\n</p>\n<p style="color: #666; font-size: 14px;">This link will expire in {{expiry_hours}} hours.</p>\n<p style="color: #666; font-size: 14px;">If you did not create an account, please ignore this email.</p>\n<br>\n<p>Best regards,<br>{{site_name}}</p>\n</body></html>' as body_html,
    '[
        {"name": "username", "required": true, "description": "User\'s username"},
        {"name": "email", "required": true, "description": "User\'s email address"},
        {"name": "verification_link", "required": true, "description": "Verification URL"},
        {"name": "expiry_hours", "required": true, "description": "Token expiry in hours"},
        {"name": "site_name", "required": true, "description": "Site name"}
    ]'::jsonb as variables_schema_json,
    false as is_compiled_from_mjml,
    NULL::uuid as created_by,
    NOW() as published_at,
    NULL::uuid as published_by
WHERE NOT EXISTS (
    SELECT 1 FROM email_template_versions 
    WHERE family_id = '00000000-0000-0000-0000-000000000105'::uuid AND status = 'published'
);

-- ============================================
-- D) Add default template for user_registration workflow
-- ============================================

INSERT INTO email_template_versions (
    family_id, version, status, locale, subject, body_text, body_html,
    variables_schema_json, is_compiled_from_mjml, created_by, published_at, published_by
)
SELECT 
    '00000000-0000-0000-0000-000000000101'::uuid as family_id,
    1 as version,
    'published'::varchar as status,
    'en'::varchar as locale,
    'Welcome to {{site_name}} - Verify your email' as subject,
    E'Welcome to {{site_name}}, {{username}}!\n\nThank you for registering. Please verify your email address by clicking the link below:\n\n{{verification_link}}\n\nThis link will expire in {{expiry_hours}} hours.\n\nIf you did not create an account, please ignore this email.\n\nBest regards,\nThe {{site_name}} Team' as body_text,
    E'<html><body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">\n<h2>Welcome to {{site_name}}, {{username}}!</h2>\n<p>Thank you for registering. Please verify your email address by clicking the link below:</p>\n<p style="margin: 20px 0;">\n<a href="{{verification_link}}" style="background: #00FFC2; color: #121213; padding: 12px 24px; text-decoration: none; border-radius: 4px; display: inline-block;">Verify Email Address</a>\n</p>\n<p style="color: #666; font-size: 14px;">This link will expire in {{expiry_hours}} hours.</p>\n<p style="color: #666; font-size: 14px;">If you did not create an account, please ignore this email.</p>\n<br>\n<p>Best regards,<br>The {{site_name}} Team</p>\n</body></html>' as body_html,
    '[
        {"name": "username", "required": true, "description": "User\'s username"},
        {"name": "email", "required": true, "description": "User\'s email address"},
        {"name": "verification_link", "required": true, "description": "Verification URL"},
        {"name": "expiry_hours", "required": true, "description": "Token expiry in hours"},
        {"name": "site_name", "required": true, "description": "Site name"}
    ]'::jsonb as variables_schema_json,
    false as is_compiled_from_mjml,
    NULL::uuid as created_by,
    NOW() as published_at,
    NULL::uuid as published_by
WHERE NOT EXISTS (
    SELECT 1 FROM email_template_versions 
    WHERE family_id = '00000000-0000-0000-0000-000000000101'::uuid AND status = 'published'
);

-- ============================================
-- E) Function to clean up expired tokens
-- ============================================

CREATE OR REPLACE FUNCTION cleanup_expired_verification_tokens()
RETURNS void AS $$
BEGIN
    DELETE FROM email_verification_tokens 
    WHERE expires_at < NOW() - INTERVAL '7 days' 
       OR (used_at IS NOT NULL AND used_at < NOW() - INTERVAL '30 days');
END;
$$ LANGUAGE plpgsql;
