-- Terms of Service Management System
-- Tracks terms versions and user acceptances

-- Terms of Service versions table
CREATE TABLE IF NOT EXISTS terms_of_service (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version VARCHAR(50) NOT NULL UNIQUE,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    summary TEXT,
    is_active BOOLEAN NOT NULL DEFAULT false,
    effective_date TIMESTAMPTZ NOT NULL,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- User terms acceptance tracking
CREATE TABLE IF NOT EXISTS user_terms_acceptance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    terms_id UUID NOT NULL REFERENCES terms_of_service(id) ON DELETE CASCADE,
    accepted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ip_address INET,
    user_agent TEXT,
    UNIQUE(user_id, terms_id)
);

-- Index for quick lookups
CREATE INDEX IF NOT EXISTS idx_terms_active ON terms_of_service(is_active, effective_date);
CREATE INDEX IF NOT EXISTS idx_user_terms_user ON user_terms_acceptance(user_id);
CREATE INDEX IF NOT EXISTS idx_user_terms_terms ON user_terms_acceptance(terms_id);

-- Trigger to ensure only one active terms version
CREATE OR REPLACE FUNCTION enforce_single_active_terms()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.is_active THEN
        UPDATE terms_of_service 
        SET is_active = false 
        WHERE id != NEW.id AND is_active = true;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_single_active_terms ON terms_of_service;
CREATE TRIGGER trigger_single_active_terms
    BEFORE INSERT OR UPDATE ON terms_of_service
    FOR EACH ROW
    EXECUTE FUNCTION enforce_single_active_terms();

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_terms_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_terms_updated_at ON terms_of_service;
CREATE TRIGGER trigger_terms_updated_at
    BEFORE UPDATE ON terms_of_service
    FOR EACH ROW
    EXECUTE FUNCTION update_terms_updated_at();
