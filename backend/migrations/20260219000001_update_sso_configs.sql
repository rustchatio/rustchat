-- Update SSO configs table for clean OIDC/OAuth2 implementation
-- Supports: github, google, and generic oidc providers

-- Step 1: Add new columns for provider types and configuration options
ALTER TABLE sso_configs 
    ADD COLUMN IF NOT EXISTS provider_key VARCHAR(64) UNIQUE,
    ADD COLUMN IF NOT EXISTS provider_type VARCHAR(32), -- 'github', 'google', 'oidc'
    ADD COLUMN IF NOT EXISTS allow_domains TEXT[], -- For Google workspace domain restriction
    ADD COLUMN IF NOT EXISTS github_org VARCHAR(255), -- GitHub organization restriction
    ADD COLUMN IF NOT EXISTS github_team VARCHAR(255), -- GitHub team restriction  
    ADD COLUMN IF NOT EXISTS groups_claim VARCHAR(64), -- OIDC groups claim name (e.g., 'groups')
    ADD COLUMN IF NOT EXISTS role_mappings JSONB; -- OIDC group -> role mappings

-- Step 2: Migrate existing data - set provider_key from provider column
-- Convert 'oidc' provider to provider_type='oidc', provider_key='oidc'
-- Convert any other provider to its own type and key
UPDATE sso_configs 
SET 
    provider_key = COALESCE(provider_key, provider),
    provider_type = COALESCE(provider_type, 
        CASE 
            WHEN provider = 'oidc' THEN 'oidc'
            WHEN provider = 'saml' THEN 'saml' -- Keep existing SAML configs
            ELSE 'oidc' -- Default existing configs to OIDC
        END
    );

-- Step 3: Make provider_key NOT NULL after migration
ALTER TABLE sso_configs ALTER COLUMN provider_key SET NOT NULL;

-- Step 4: Drop the old unique constraint on org_id (we now allow multiple providers per org)
-- First check if the constraint exists and drop it
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint 
        WHERE conname = 'sso_configs_org_unique' 
        AND conrelid = 'sso_configs'::regclass
    ) THEN
        ALTER TABLE sso_configs DROP CONSTRAINT sso_configs_org_unique;
    END IF;
END $$;

-- Step 5: Create new unique constraint on provider_key (used in URLs)
-- This was already added as UNIQUE in step 1, but ensure index exists
CREATE UNIQUE INDEX IF NOT EXISTS idx_sso_configs_provider_key ON sso_configs(provider_key);

-- Step 6: Create index for listing active providers
CREATE INDEX IF NOT EXISTS idx_sso_configs_active ON sso_configs(is_active, provider_key);

-- Step 7: Add constraint to validate provider_type values
ALTER TABLE sso_configs 
    ADD CONSTRAINT chk_provider_type 
    CHECK (provider_type IN ('github', 'google', 'oidc', 'saml'));

-- Step 8: Add constraint to ensure issuer_url is provided for google and oidc types
ALTER TABLE sso_configs 
    ADD CONSTRAINT chk_issuer_required 
    CHECK (
        (provider_type IN ('github', 'saml')) OR 
        (provider_type IN ('google', 'oidc') AND issuer_url IS NOT NULL AND issuer_url != '')
    );

-- Step 9: Ensure scopes defaults are appropriate
-- For GitHub: read:user user:email
-- For OIDC/Google: openid profile email
UPDATE sso_configs 
SET scopes = CASE provider_type
    WHEN 'github' THEN ARRAY['read:user', 'user:email']
    ELSE ARRAY['openid', 'profile', 'email']
END
WHERE scopes IS NULL OR array_length(scopes, 1) IS NULL;

-- Add comment documenting the table structure
COMMENT ON TABLE sso_configs IS 'SSO/OAuth2/OIDC provider configurations. Supports github, google, and generic oidc providers. provider_key is used in URL paths and must be URL-safe (a-z, 0-9, -)';
COMMENT ON COLUMN sso_configs.provider_key IS 'Unique URL-safe identifier used in OAuth callback URLs (e.g., "github", "google", "oidc-keycloak")';
COMMENT ON COLUMN sso_configs.provider_type IS 'Type of provider: github (OAuth2), google (OIDC), oidc (generic OIDC discovery), saml (legacy)';
COMMENT ON COLUMN sso_configs.allow_domains IS 'For Google: restrict to specific email domains (e.g., ["company.com"])';
COMMENT ON COLUMN sso_configs.github_org IS 'For GitHub: require membership in this organization';
COMMENT ON COLUMN sso_configs.github_team IS 'For GitHub: require membership in this team (within github_org)';
COMMENT ON COLUMN sso_configs.groups_claim IS 'For OIDC: claim name containing user groups/roles (e.g., "groups", "roles")';
COMMENT ON COLUMN sso_configs.role_mappings IS 'For OIDC: JSON mapping of provider groups to RustChat roles {"group-name": "member"}';
