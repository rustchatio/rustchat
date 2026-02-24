-- Password Reset System
-- Adds password reset tokens and templates

-- ============================================
-- A) Password Reset Tokens
-- ============================================

CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    token_hash VARCHAR(64) NOT NULL,
    user_id UUID NULL REFERENCES users(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    purpose VARCHAR(50) NOT NULL DEFAULT 'password_reset',
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ NULL,
    created_ip INET NULL,
    user_agent TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_hash ON password_reset_tokens(token_hash);
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_email ON password_reset_tokens(email);
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_expires ON password_reset_tokens(expires_at) WHERE used_at IS NULL;

-- ============================================
-- B) Create default password reset template
-- ============================================

INSERT INTO email_template_families (id, key, name, description, workflow_key, is_system) VALUES
    ('00000000-0000-0000-0000-000000000106'::uuid, 'password_reset_default', 'Default Password Reset', 'Standard password reset template', 'password_reset', true)
ON CONFLICT (tenant_id, key) DO NOTHING;

UPDATE notification_workflows 
    SET selected_template_family_id = '00000000-0000-0000-0000-000000000106'::uuid 
    WHERE workflow_key = 'password_reset' AND selected_template_family_id IS NULL;

INSERT INTO email_template_versions (
    family_id, version, status, locale, subject, body_text, body_html,
    variables_schema_json, is_compiled_from_mjml, created_by, published_at, published_by
)
SELECT 
    '00000000-0000-0000-0000-000000000106'::uuid as family_id,
    1 as version,
    'published'::varchar as status,
    'en'::varchar as locale,
    'Reset your password' as subject,
    E'Hello {{username}},

We received a request to reset your password. Click the link below to create a new password:

{{reset_link}}

This link will expire in {{expiry_minutes}} minutes.

If you did not request this password reset, please ignore this email. Your password will remain unchanged.

Best regards,
{{site_name}}' as body_text,
    E'<html><body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
<h2>Hello {{username}},</h2>
<p>We received a request to reset your password. Click the link below to create a new password:</p>
<p style="margin: 20px 0;">
<a href="{{reset_link}}" style="background: #00FFC2; color: #121213; padding: 12px 24px; text-decoration: none; border-radius: 4px; display: inline-block;">Reset Password</a>
</p>
<p style="color: #666; font-size: 14px;">This link will expire in {{expiry_minutes}} minutes.</p>
<p style="color: #666; font-size: 14px;">If you did not request this password reset, please ignore this email. Your password will remain unchanged.</p>
<br>
<p>Best regards,<br>{{site_name}}</p>
</body></html>' as body_html,
    '[{"name":"username","required":true,"description":"User username"},{"name":"email","required":true,"description":"User email"},{"name":"reset_link","required":true,"description":"Password reset URL"},{"name":"expiry_minutes","required":true,"description":"Expiry in minutes"},{"name":"site_name","required":true,"description":"Site name"}]'::jsonb as variables_schema_json,
    false as is_compiled_from_mjml,
    NULL::uuid as created_by,
    NOW() as published_at,
    NULL::uuid as published_by
WHERE NOT EXISTS (
    SELECT 1 FROM email_template_versions 
    WHERE family_id = '00000000-0000-0000-0000-000000000106'::uuid AND status = 'published'
);

-- ============================================
-- C) Function to clean up expired tokens
-- ============================================

CREATE OR REPLACE FUNCTION cleanup_expired_password_reset_tokens()
RETURNS void AS $$
BEGIN
    DELETE FROM password_reset_tokens 
    WHERE expires_at < NOW() - INTERVAL '7 days' 
       OR (used_at IS NOT NULL AND used_at < NOW() - INTERVAL '30 days');
END;
$$ LANGUAGE plpgsql;
