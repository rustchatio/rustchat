-- GDPR Compliance Tables
-- Migration: B1.2 GDPR Hard Delete Implementation

-- Audit log table for GDPR deletion requests
CREATE TABLE IF NOT EXISTS gdpr_deletion_log (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL,
    deleted_by UUID NOT NULL,
    deleted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reason TEXT,
    files_deleted INTEGER DEFAULT 0,
    posts_anonymized BOOLEAN DEFAULT FALSE,
    deletion_type VARCHAR(50) NOT NULL DEFAULT 'hard_delete', -- 'hard_delete', 'anonymize'
    request_source VARCHAR(255), -- e.g., 'user_request', 'admin_action', 'retention_policy'
    ip_address INET,
    user_agent TEXT
);

-- Index for audit queries
CREATE INDEX IF NOT EXISTS idx_gdpr_deletion_log_user_id ON gdpr_deletion_log(user_id);
CREATE INDEX IF NOT EXISTS idx_gdpr_deletion_log_deleted_at ON gdpr_deletion_log(deleted_at);
CREATE INDEX IF NOT EXISTS idx_gdpr_deletion_log_deletion_type ON gdpr_deletion_log(deletion_type);

-- GDPR data export log
CREATE TABLE IF NOT EXISTS gdpr_export_log (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL,
    requested_by UUID NOT NULL,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    export_format VARCHAR(50) NOT NULL DEFAULT 'json', -- 'json', 'csv', 'xml'
    file_path TEXT, -- Path to stored export file (if persisted)
    file_size BIGINT,
    checksum VARCHAR(64), -- SHA-256 checksum of the export file
    expires_at TIMESTAMPTZ, -- When the export should be deleted
    ip_address INET,
    user_agent TEXT
);

CREATE INDEX IF NOT EXISTS idx_gdpr_export_log_user_id ON gdpr_export_log(user_id);
CREATE INDEX IF NOT EXISTS idx_gdpr_export_log_requested_at ON gdpr_export_log(requested_at);

-- Ensure we have an audit_log table for tracking user actions
-- (This might already exist, but we ensure it has the right columns)
CREATE TABLE IF NOT EXISTS audit_log (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID,
    action VARCHAR(255) NOT NULL,
    entity_type VARCHAR(255) NOT NULL,
    entity_id UUID,
    old_value JSONB,
    new_value JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_log_user_id ON audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_entity ON audit_log(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_created_at ON audit_log(created_at);

-- Comments for documentation
COMMENT ON TABLE gdpr_deletion_log IS 'Audit log of all GDPR Article 17 (Right to Erasure) deletion requests';
COMMENT ON TABLE gdpr_export_log IS 'Audit log of all GDPR Article 20 (Data Portability) export requests';
COMMENT ON TABLE audit_log IS 'General audit log for user actions';
