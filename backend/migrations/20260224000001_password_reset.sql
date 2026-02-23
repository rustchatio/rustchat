-- Password Reset System Migration
-- Creates password_reset_tokens table and related infrastructure

-- ============================================
-- A) Password Reset Tokens Table
-- ============================================

CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Token hash (SHA-256, never store raw token)
    token_hash VARCHAR(64) NOT NULL,
    
    -- User reference (nullable for anti-enumeration - set when user found)
    user_id UUID NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Email being reset
    email VARCHAR(255) NOT NULL,
    
    -- Purpose: 'password_reset', 'password_setup', 'email_change'
    purpose VARCHAR(50) NOT NULL DEFAULT 'password_reset',
    
    -- Expiry and usage tracking
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ NULL,
    
    -- Metadata for security auditing
    created_ip INET NULL,
    user_agent TEXT NULL,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient lookups
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_hash ON password_reset_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_user ON password_reset_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_email ON password_reset_tokens(email);
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_created ON password_reset_tokens(created_at);
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_active ON password_reset_tokens(token_hash, used_at, expires_at) 
    WHERE used_at IS NULL AND expires_at > NOW();

-- ============================================
-- B) Default Password Reset Template
-- ============================================

-- Add template family for password reset
INSERT INTO email_template_families (id, key, name, description, workflow_key, is_system) VALUES
    ('00000000-0000-0000-0000-000000000106'::uuid, 'password_reset_default', 'Default Password Reset', 'Standard password reset email template', 'password_reset', true)
ON CONFLICT (tenant_id, key) DO NOTHING;

-- Link workflow to template family
UPDATE notification_workflows 
    SET selected_template_family_id = '00000000-0000-0000-0000-000000000106'::uuid 
    WHERE workflow_key = 'password_reset' AND selected_template_family_id IS NULL;

-- Insert default template version for password reset
INSERT INTO email_template_versions (
    family_id, version, status, locale, subject, body_text, body_html,
    variables_schema_json, is_compiled_from_mjml, created_by, published_at, published_by
)
SELECT 
    '00000000-0000-0000-0000-000000000106'::uuid as family_id,
    1 as version,
    'published'::varchar as status,
    'en'::varchar as locale,
    'Reset your {{site_name}} password' as subject,
    E'Hello {{username}},\n\nWe received a request to reset your password for your {{site_name}} account.\n\nClick the link below to reset your password:\n\n{{reset_link}}\n\nThis link will expire in {{expiry_minutes}} minutes.\n\nIf you did not request this password reset, please ignore this email. Your password will remain unchanged.\n\nFor security, this request was made from IP: {{ip_address}}\n\nBest regards,\nThe {{site_name}} Team' as body_text,
    E'<html><body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">\n<h2>Hello {{username}},</h2>\n<p>We received a request to reset your password for your {{site_name}} account.</p>\n<p style="margin: 20px 0;">\n<a href="{{reset_link}}" style="background: #00FFC2; color: #121213; padding: 12px 24px; text-decoration: none; border-radius: 4px; display: inline-block;">Reset Password</a>\n</p>\n<p style="color: #666; font-size: 14px;">This link will expire in {{expiry_minutes}} minutes.</p>\n<p style="color: #666; font-size: 14px;">If you did not request this password reset, please ignore this email. Your password will remain unchanged.</p>\n<p style="color: #666; font-size: 14px;">For security, this request was made from IP: {{ip_address}}</p>\n<br>\n<p>Best regards,<br>The {{site_name}} Team</p>\n</body></html>' as body_html,
    '[
        {"name": "username", "required": true, "description": "User\'s username"},
        {"name": "email", "required": true, "description": "User\'s email address"},
        {"name": "reset_link", "required": true, "description": "Password reset URL"},
        {"name": "expiry_minutes", "required": true, "description": "Token expiry in minutes"},
        {"name": "site_name", "required": true, "description": "Site name"},
        {"name": "ip_address", "required": false, "description": "IP address of requester"}
    ]'::jsonb as variables_schema_json,
    false as is_compiled_from_mjml,
    NULL::uuid as created_by,
    NOW() as published_at,
    NULL::uuid as published_by
WHERE NOT EXISTS (
    SELECT 1 FROM email_template_versions 
    WHERE family_id = '00000000-0000-0000-0000-000000000106'::uuid AND status = 'published'
);

-- ============================================
-- C) Function to cleanup old tokens
-- ============================================

CREATE OR REPLACE FUNCTION cleanup_expired_password_tokens()
RETURNS void AS $$
BEGIN
    -- Delete tokens older than 7 days
    DELETE FROM password_reset_tokens 
    WHERE created_at < NOW() - INTERVAL '7 days';
    
    -- Delete used tokens older than 30 days
    DELETE FROM password_reset_tokens 
    WHERE used_at IS NOT NULL 
      AND used_at < NOW() - INTERVAL '30 days';
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- D) Comments for documentation
-- ============================================

COMMENT ON TABLE password_reset_tokens IS 'Stores password reset/setup tokens. IMPORTANT: Only SHA-256 hashes are stored, never raw tokens.';
COMMENT ON COLUMN password_reset_tokens.token_hash IS 'SHA-256 hash of the token. Raw tokens are never stored.';
COMMENT ON COLUMN password_reset_tokens.user_id IS 'Nullable for anti-enumeration. Set only when user exists.';
COMMENT ON COLUMN password_reset_tokens.purpose IS 'password_reset: existing user, password_setup: invited user without password';
