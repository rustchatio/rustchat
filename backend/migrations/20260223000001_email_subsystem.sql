-- Email Subsystem Migration
-- Production-grade email system with provider abstraction, templates, workflows, and audit

-- ============================================
-- A) Provider Settings (tenant-aware)
-- ============================================

CREATE TABLE IF NOT EXISTS mail_provider_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NULL REFERENCES organizations(id) ON DELETE CASCADE,
    provider_type VARCHAR(50) NOT NULL DEFAULT 'smtp',
    
    -- SMTP Configuration
    host VARCHAR(255) NOT NULL DEFAULT '',
    port INTEGER NOT NULL DEFAULT 587,
    username VARCHAR(255) NOT NULL DEFAULT '',
    password_encrypted TEXT NOT NULL DEFAULT '',
    tls_mode VARCHAR(20) NOT NULL DEFAULT 'starttls', -- 'starttls', 'implicit_tls', 'none'
    skip_cert_verify BOOLEAN NOT NULL DEFAULT false,
    
    -- Sender Configuration
    from_address VARCHAR(255) NOT NULL DEFAULT '',
    from_name VARCHAR(255) NOT NULL DEFAULT 'RustChat',
    reply_to VARCHAR(255) NULL,
    
    -- Rate Limiting (per tenant)
    max_emails_per_minute INTEGER NOT NULL DEFAULT 60,
    max_emails_per_hour INTEGER NOT NULL DEFAULT 1000,
    
    -- Status
    enabled BOOLEAN NOT NULL DEFAULT true,
    is_default BOOLEAN NOT NULL DEFAULT false,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id),
    
    -- Constraints
    CONSTRAINT valid_provider_type CHECK (provider_type IN ('smtp', 'ses', 'sendgrid')),
    CONSTRAINT valid_tls_mode CHECK (tls_mode IN ('starttls', 'implicit_tls', 'none')),
    CONSTRAINT unique_tenant_default UNIQUE NULLS NOT DISTINCT (tenant_id, is_default)
);

CREATE INDEX IF NOT EXISTS idx_mail_provider_tenant ON mail_provider_settings(tenant_id);
CREATE INDEX IF NOT EXISTS idx_mail_provider_enabled ON mail_provider_settings(enabled);

-- ============================================
-- B) Notification Workflows (fixed registry)
-- ============================================

CREATE TABLE IF NOT EXISTS notification_workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NULL REFERENCES organizations(id) ON DELETE CASCADE,
    
    -- Workflow Identification (fixed keys: user_registration, password_reset, announcements, offline_messages)
    workflow_key VARCHAR(100) NOT NULL,
    
    -- Display
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(50) NOT NULL DEFAULT 'system', -- 'system', 'notification', 'marketing'
    
    -- Status
    enabled BOOLEAN NOT NULL DEFAULT true,
    system_required BOOLEAN NOT NULL DEFAULT false, -- cannot disable if true (security emails)
    
    -- Template Selection
    default_locale VARCHAR(10) NOT NULL DEFAULT 'en',
    selected_template_family_id UUID NULL, -- will be set after template_families is created
    
    -- Policy Configuration (JSON for flexibility)
    policy_json JSONB NOT NULL DEFAULT '{}',
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id),
    
    -- Constraints
    CONSTRAINT unique_workflow_per_tenant UNIQUE (tenant_id, workflow_key),
    CONSTRAINT valid_workflow_key CHECK (
        workflow_key IN (
            'user_registration',
            'email_verification',
            'password_reset',
            'password_changed',
            'security_alert',
            'announcements',
            'offline_messages',
            'mention_notification',
            'admin_invite',
            'weekly_digest'
        )
    )
);

CREATE INDEX IF NOT EXISTS idx_workflows_tenant ON notification_workflows(tenant_id);
CREATE INDEX IF NOT EXISTS idx_workflows_enabled ON notification_workflows(enabled);
CREATE INDEX IF NOT EXISTS idx_workflows_key ON notification_workflows(workflow_key);

-- ============================================
-- C) Email Template Families
-- ============================================

CREATE TABLE IF NOT EXISTS email_template_families (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NULL REFERENCES organizations(id) ON DELETE CASCADE,
    
    -- Identification
    key VARCHAR(100) NOT NULL, -- e.g., 'registration_default'
    name VARCHAR(255) NOT NULL,
    description TEXT,
    
    -- Associated workflow (optional - templates can be reused)
    workflow_key VARCHAR(100) NULL,
    
    -- Metadata
    is_system BOOLEAN NOT NULL DEFAULT false, -- system families cannot be deleted
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id),
    
    -- Constraints
    CONSTRAINT unique_family_key_per_tenant UNIQUE (tenant_id, key)
);

CREATE INDEX IF NOT EXISTS idx_template_families_tenant ON email_template_families(tenant_id);
CREATE INDEX IF NOT EXISTS idx_template_families_workflow ON email_template_families(workflow_key);

-- Add foreign key to workflows after families table exists
ALTER TABLE notification_workflows 
    ADD CONSTRAINT fk_workflows_template_family 
    FOREIGN KEY (selected_template_family_id) 
    REFERENCES email_template_families(id) 
    ON DELETE SET NULL;

-- ============================================
-- D) Email Template Versions
-- ============================================

CREATE TABLE IF NOT EXISTS email_template_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    family_id UUID NOT NULL REFERENCES email_template_families(id) ON DELETE CASCADE,
    
    -- Versioning
    version INTEGER NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'draft', -- 'draft', 'published', 'archived'
    
    -- Localization
    locale VARCHAR(10) NOT NULL DEFAULT 'en',
    
    -- Content
    subject VARCHAR(500) NOT NULL,
    body_text TEXT NOT NULL,
    body_html TEXT NOT NULL,
    
    -- Variable Schema (for validation and UI helpers)
    variables_schema_json JSONB NOT NULL DEFAULT '[]',
    
    -- MJML/Source tracking
    is_compiled_from_mjml BOOLEAN NOT NULL DEFAULT false,
    mjml_source TEXT NULL,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID REFERENCES users(id),
    published_at TIMESTAMPTZ NULL,
    published_by UUID NULL REFERENCES users(id),
    
    -- Constraints
    CONSTRAINT valid_status CHECK (status IN ('draft', 'published', 'archived')),
    CONSTRAINT unique_version_per_family_locale UNIQUE (family_id, version, locale),
    CONSTRAINT unique_published_per_family_locale 
        UNIQUE NULLS NOT DISTINCT (family_id, locale, status) 
        DEFERRABLE INITIALLY DEFERRED
);

CREATE INDEX IF NOT EXISTS idx_template_versions_family ON email_template_versions(family_id);
CREATE INDEX IF NOT EXISTS idx_template_versions_status ON email_template_versions(status);
CREATE INDEX IF NOT EXISTS idx_template_versions_locale ON email_template_versions(locale);
CREATE INDEX IF NOT EXISTS idx_template_versions_published ON email_template_versions(family_id, locale, status) 
    WHERE status = 'published';

-- ============================================
-- E) Email Outbox (Queue)
-- ============================================

CREATE TYPE email_status AS ENUM ('queued', 'sending', 'sent', 'failed', 'cancelled');
CREATE TYPE email_priority AS ENUM ('high', 'normal', 'low');

CREATE TABLE IF NOT EXISTS email_outbox (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Tenant/Organization context
    tenant_id UUID NULL REFERENCES organizations(id) ON DELETE CASCADE,
    
    -- Workflow context
    workflow_key VARCHAR(100) NULL,
    
    -- Template reference
    template_family_id UUID NULL REFERENCES email_template_families(id) ON DELETE SET NULL,
    template_version INTEGER NULL,
    locale VARCHAR(10) NULL,
    
    -- Recipient
    recipient_email VARCHAR(255) NOT NULL,
    recipient_user_id UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    
    -- Rendered Content (stored for audit/debugging)
    subject TEXT NOT NULL,
    body_text TEXT NULL,
    body_html TEXT NULL,
    
    -- Payload (template variables used)
    payload_json JSONB NOT NULL DEFAULT '{}',
    
    -- Headers
    headers_json JSONB NULL, -- List-Unsubscribe, etc.
    
    -- Queue Management
    status email_status NOT NULL DEFAULT 'queued',
    priority email_priority NOT NULL DEFAULT 'normal',
    
    -- Scheduling
    scheduled_at TIMESTAMPTZ NULL, -- for delayed sending
    send_after TIMESTAMPTZ NULL,   -- quiet hours compliance
    
    -- Retry Logic
    attempt_count INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    next_attempt_at TIMESTAMPTZ NULL,
    
    -- Error Tracking
    last_error_category VARCHAR(50) NULL,
    last_error_message TEXT NULL,
    
    -- Provider/Result
    provider_id UUID NULL REFERENCES mail_provider_settings(id) ON DELETE SET NULL,
    provider_message_id VARCHAR(255) NULL,
    sent_at TIMESTAMPTZ NULL,
    
    -- Throttling/Rate Limiting (for offline notifications)
    throttle_key VARCHAR(255) NULL, -- e.g., "user:{user_id}:channel:{channel_id}"
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID NULL REFERENCES users(id), -- NULL for system-generated
    
    -- IP/Origin tracking
    source_ip INET NULL,
    source_service VARCHAR(100) NULL
);

CREATE INDEX IF NOT EXISTS idx_email_outbox_status ON email_outbox(status);
CREATE INDEX IF NOT EXISTS idx_email_outbox_status_next_attempt 
    ON email_outbox(status, next_attempt_at) 
    WHERE status IN ('queued', 'failed');
CREATE INDEX IF NOT EXISTS idx_email_outbox_tenant ON email_outbox(tenant_id);
CREATE INDEX IF NOT EXISTS idx_email_outbox_workflow ON email_outbox(workflow_key);
CREATE INDEX IF NOT EXISTS idx_email_outbox_recipient ON email_outbox(recipient_user_id);
CREATE INDEX IF NOT EXISTS idx_email_outbox_throttle ON email_outbox(throttle_key, created_at);
CREATE INDEX IF NOT EXISTS idx_email_outbox_scheduled ON email_outbox(scheduled_at) 
    WHERE scheduled_at IS NOT NULL AND status = 'queued';

-- ============================================
-- F) Email Events (Audit Log)
-- ============================================

CREATE TABLE IF NOT EXISTS email_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Reference to outbox (may be NULL for direct sends)
    outbox_id UUID NULL REFERENCES email_outbox(id) ON DELETE SET NULL,
    
    -- Context
    tenant_id UUID NULL REFERENCES organizations(id) ON DELETE CASCADE,
    workflow_key VARCHAR(100) NULL,
    
    -- Event Type
    event_type VARCHAR(50) NOT NULL, -- 'queued', 'sent', 'delivered', 'bounced', 'failed', 'opened', 'clicked'
    
    -- Recipient (denormalized for audit)
    recipient_email VARCHAR(255) NOT NULL,
    recipient_user_id UUID NULL,
    
    -- Template info
    template_family_id UUID NULL,
    template_version INTEGER NULL,
    locale VARCHAR(10) NULL,
    
    -- Provider info
    provider_id UUID NULL,
    provider_message_id VARCHAR(255) NULL,
    
    -- Event Details
    status_code INTEGER NULL,
    error_category VARCHAR(50) NULL,
    error_message TEXT NULL,
    provider_response JSONB NULL,
    
    -- Metadata
    ip_address INET NULL,
    user_agent TEXT NULL,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_email_events_outbox ON email_events(outbox_id);
CREATE INDEX IF NOT EXISTS idx_email_events_tenant ON email_events(tenant_id);
CREATE INDEX IF NOT EXISTS idx_email_events_recipient ON email_events(recipient_email);
CREATE INDEX IF NOT EXISTS idx_email_events_type ON email_events(event_type);
CREATE INDEX IF NOT EXISTS idx_email_events_created ON email_events(created_at);
CREATE INDEX IF NOT EXISTS idx_email_events_workflow ON email_events(workflow_key);

-- Partitioning for high volume (optional, can be enabled later)
-- CREATE TABLE IF NOT EXISTS email_events_partitioned (LIKE email_events) PARTITION BY RANGE (created_at);

-- ============================================
-- G) User Notification Preferences
-- ============================================

CREATE TABLE IF NOT EXISTS user_notification_prefs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    
    -- Email Preferences
    email_enabled BOOLEAN NOT NULL DEFAULT true,
    
    -- Per-workflow opt-ins (NULL = use tenant default)
    announcements_opt_in BOOLEAN NULL DEFAULT NULL,
    offline_notifications_opt_in BOOLEAN NULL DEFAULT true,
    mention_notifications_opt_in BOOLEAN NULL DEFAULT true,
    digest_opt_in BOOLEAN NULL DEFAULT NULL,
    
    -- Quiet Hours (JSON for flexibility)
    quiet_hours_json JSONB NULL, -- {"enabled": true, "start": "22:00", "end": "08:00", "timezone": "UTC"}
    
    -- Locale Override
    locale VARCHAR(10) NULL,
    
    -- Throttling preferences
    offline_throttle_minutes INTEGER NULL DEFAULT 5, -- min minutes between offline emails per channel
    
    -- Content preferences
    include_message_content BOOLEAN NOT NULL DEFAULT true,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_user_notif_prefs_user ON user_notification_prefs(user_id);

-- ============================================
-- H) Functions and Triggers
-- ============================================

-- Update timestamp trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply update triggers
CREATE TRIGGER update_mail_provider_settings_updated_at
    BEFORE UPDATE ON mail_provider_settings
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_notification_workflows_updated_at
    BEFORE UPDATE ON notification_workflows
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_email_template_families_updated_at
    BEFORE UPDATE ON email_template_families
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_email_outbox_updated_at
    BEFORE UPDATE ON email_outbox
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_notification_prefs_updated_at
    BEFORE UPDATE ON user_notification_prefs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================
-- I) Seed Data
-- ============================================

-- Seed default mail provider (empty, needs admin configuration)
INSERT INTO mail_provider_settings (
    id, tenant_id, provider_type, host, port, username, password_encrypted,
    tls_mode, from_address, from_name, enabled, is_default
) VALUES (
    '00000000-0000-0000-0000-000000000001'::uuid,
    NULL,
    'smtp',
    '',
    587,
    '',
    '',
    'starttls',
    '',
    'RustChat',
    false,
    true
) ON CONFLICT DO NOTHING;

-- Seed workflow registry
INSERT INTO notification_workflows (workflow_key, name, description, category, enabled, system_required, default_locale, policy_json) VALUES
    ('user_registration', 'User Registration', 'Email verification for new user registration', 'system', true, true, 'en', '{"require_verification": true}'::jsonb),
    ('email_verification', 'Email Verification', 'Verify email address changes', 'system', true, true, 'en', '{}'::jsonb),
    ('password_reset', 'Password Reset', 'Password reset request emails', 'system', true, true, 'en', '{"token_expiry_hours": 24}'::jsonb),
    ('password_changed', 'Password Changed', 'Notification when password is changed', 'system', true, true, 'en', '{}'::jsonb),
    ('security_alert', 'Security Alerts', 'Security-related notifications (new device, etc.)', 'system', true, true, 'en', '{}'::jsonb),
    ('announcements', 'System Announcements', 'Broadcast announcements from admins', 'marketing', true, false, 'en', '{"require_opt_in": true, "list_unsubscribe": true}'::jsonb),
    ('offline_messages', 'Offline Message Notifications', 'Notify users of messages when offline', 'notification', true, false, 'en', '{"throttle_minutes": 5, "max_per_hour": 10, "include_excerpt": true, "respect_quiet_hours": true}'::jsonb),
    ('mention_notifications', 'Mention Notifications', 'Notify when mentioned in messages', 'notification', true, false, 'en', '{"throttle_minutes": 1}'::jsonb),
    ('admin_invite', 'Admin Invites', 'Team/channel invitation emails', 'system', true, false, 'en', '{}'::jsonb),
    ('weekly_digest', 'Weekly Digest', 'Weekly activity summary', 'notification', false, false, 'en', '{"day_of_week": 1, "hour": 9}'::jsonb)
ON CONFLICT (tenant_id, workflow_key) DO NOTHING;

-- Seed default template families
INSERT INTO email_template_families (id, key, name, description, workflow_key, is_system) VALUES
    ('00000000-0000-0000-0000-000000000101'::uuid, 'registration_default', 'Default Registration Template', 'Standard user registration verification email', 'user_registration', true),
    ('00000000-0000-0000-0000-000000000102'::uuid, 'password_reset_default', 'Default Password Reset', 'Standard password reset email', 'password_reset', true),
    ('00000000-0000-0000-0000-000000000103'::uuid, 'announcements_default', 'Default Announcements', 'Standard announcement email template', 'announcements', true),
    ('00000000-0000-0000-0000-000000000104'::uuid, 'offline_messages_default', 'Default Offline Notifications', 'Standard offline message notification template', 'offline_messages', true)
ON CONFLICT (tenant_id, key) DO NOTHING;

-- Link workflows to default template families
UPDATE notification_workflows 
    SET selected_template_family_id = '00000000-0000-0000-0000-000000000101'::uuid 
    WHERE workflow_key = 'user_registration' AND selected_template_family_id IS NULL;
    
UPDATE notification_workflows 
    SET selected_template_family_id = '00000000-0000-0000-0000-000000000102'::uuid 
    WHERE workflow_key = 'password_reset' AND selected_template_family_id IS NULL;
    
UPDATE notification_workflows 
    SET selected_template_family_id = '00000000-0000-0000-0000-000000000103'::uuid 
    WHERE workflow_key = 'announcements' AND selected_template_family_id IS NULL;
    
UPDATE notification_workflows 
    SET selected_template_family_id = '00000000-0000-0000-0000-000000000104'::uuid 
    WHERE workflow_key = 'offline_messages' AND selected_template_family_id IS NULL;
