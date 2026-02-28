-- Auto Membership Policies
-- Migration: Implements P1 from autosubscription gap plan

-- Main policy table
CREATE TABLE auto_membership_policies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    scope_type VARCHAR(20) NOT NULL CHECK (scope_type IN ('global', 'team')),
    team_id UUID REFERENCES teams(id) ON DELETE CASCADE,
    source_type VARCHAR(20) NOT NULL CHECK (source_type IN ('all_users', 'auth_service', 'group', 'role', 'org')),
    source_config JSONB DEFAULT '{}', -- additional filter config (e.g., auth_service type, group IDs, role names)
    enabled BOOLEAN NOT NULL DEFAULT true,
    priority INTEGER NOT NULL DEFAULT 0, -- higher number = higher priority
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Global policies must not have team_id; team policies must have team_id
    CONSTRAINT check_scope_team_consistency 
        CHECK ((scope_type = 'global' AND team_id IS NULL) OR (scope_type = 'team' AND team_id IS NOT NULL))
    
    -- Unique name per scope is enforced via expression index below.
    -- (Postgres table UNIQUE constraints cannot include expressions like COALESCE.)
);

-- Policy targets (teams/channels to auto-subscribe to)
CREATE TABLE auto_membership_policy_targets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    policy_id UUID NOT NULL REFERENCES auto_membership_policies(id) ON DELETE CASCADE,
    target_type VARCHAR(20) NOT NULL CHECK (target_type IN ('team', 'channel')),
    target_id UUID NOT NULL, -- references teams(id) or channels(id)
    role_mode VARCHAR(20) DEFAULT 'member' CHECK (role_mode IN ('member', 'admin')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Prevent duplicate targets per policy
    CONSTRAINT unique_target_per_policy UNIQUE (policy_id, target_type, target_id)
);

-- Audit log for policy runs
CREATE TABLE auto_membership_policy_audit (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    policy_id UUID REFERENCES auto_membership_policies(id) ON DELETE SET NULL,
    run_id UUID NOT NULL, -- groups operations from the same reconciliation run
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_type VARCHAR(20) NOT NULL CHECK (target_type IN ('team', 'channel')),
    target_id UUID NOT NULL,
    action VARCHAR(20) NOT NULL CHECK (action IN ('add', 'remove', 'skip')),
    status VARCHAR(20) NOT NULL CHECK (status IN ('success', 'failed', 'pending')),
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Track membership origin to distinguish manual vs policy-added memberships
-- Add origin column to existing tables via companion table (to avoid altering core tables)
CREATE TABLE membership_origins (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    membership_type VARCHAR(20) NOT NULL CHECK (membership_type IN ('team', 'channel')),
    membership_id UUID NOT NULL, -- references team_members or channel_members
    origin VARCHAR(20) NOT NULL DEFAULT 'manual' CHECK (origin IN ('manual', 'policy', 'invite', 'sync', 'default')),
    policy_id UUID REFERENCES auto_membership_policies(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- One origin record per membership
    CONSTRAINT unique_membership_origin UNIQUE (membership_type, membership_id)
);

-- Indexes for performance
CREATE INDEX idx_auto_membership_policies_scope ON auto_membership_policies(scope_type, team_id) WHERE enabled = true;
CREATE INDEX idx_auto_membership_policies_source ON auto_membership_policies(source_type, enabled);
CREATE UNIQUE INDEX idx_auto_membership_policies_unique_name_per_scope
    ON auto_membership_policies (name, scope_type, COALESCE(team_id, '00000000-0000-0000-0000-000000000000'::uuid));
CREATE INDEX idx_auto_membership_policy_targets_policy ON auto_membership_policy_targets(policy_id);
CREATE INDEX idx_auto_membership_policy_targets_lookup ON auto_membership_policy_targets(target_type, target_id);
CREATE INDEX idx_auto_membership_policy_audit_policy ON auto_membership_policy_audit(policy_id);
CREATE INDEX idx_auto_membership_policy_audit_run ON auto_membership_policy_audit(run_id);
CREATE INDEX idx_auto_membership_policy_audit_user ON auto_membership_policy_audit(user_id);
CREATE INDEX idx_auto_membership_policy_audit_created ON auto_membership_policy_audit(created_at);
CREATE INDEX idx_membership_origins_lookup ON membership_origins(membership_type, membership_id);
CREATE INDEX idx_membership_origins_policy ON membership_origins(policy_id);

-- Trigger to update updated_at timestamp (reuses existing function if available)
CREATE TRIGGER update_auto_membership_policies_updated_at
    BEFORE UPDATE ON auto_membership_policies
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_membership_origins_updated_at
    BEFORE UPDATE ON membership_origins
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
