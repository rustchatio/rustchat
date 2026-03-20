-- Seed data for integration tests
-- Minimal dataset for entity registration and authentication flows

-- Test team
INSERT INTO teams (id, name, display_name, description, type) VALUES
('test_team_001', 'test-team', 'Test Team', 'Test team for integration tests', 'O')
ON CONFLICT (id) DO NOTHING;

-- Test users
INSERT INTO users (id, username, email, password, first_name, last_name, locale, timezone) VALUES
('test_user_001', 'testuser1', 'testuser1@example.com', '$2b$10$abcdefghijklmnopqrstuv', 'Test', 'User1', 'en', 'UTC'),
('test_user_002', 'testuser2', 'testuser2@example.com', '$2b$10$abcdefghijklmnopqrstuv', 'Test', 'User2', 'en', 'UTC'),
('test_admin_001', 'testadmin', 'testadmin@example.com', '$2b$10$abcdefghijklmnopqrstuv', 'Test', 'Admin', 'en', 'UTC')
ON CONFLICT (id) DO NOTHING;

-- Make test_admin_001 a system admin
UPDATE users SET roles = 'system_admin' WHERE id = 'test_admin_001';

-- Add users to team
INSERT INTO team_members (team_id, user_id, roles) VALUES
('test_team_001', 'test_user_001', 'team_user'),
('test_team_001', 'test_user_002', 'team_user'),
('test_team_001', 'test_admin_001', 'team_admin')
ON CONFLICT (team_id, user_id) DO NOTHING;

-- Test channel
INSERT INTO channels (id, team_id, type, display_name, name, creator_id) VALUES
('test_channel_001', 'test_team_001', 'O', 'Test Channel', 'test-channel', 'test_user_001')
ON CONFLICT (id) DO NOTHING;

-- Add members to channel
INSERT INTO channel_members (channel_id, user_id) VALUES
('test_channel_001', 'test_user_001'),
('test_channel_001', 'test_user_002'),
('test_channel_001', 'test_admin_001')
ON CONFLICT (channel_id, user_id) DO NOTHING;

-- Test entities for API key testing
INSERT INTO entities (id, entity_type, name, description, owner_id, is_active) VALUES
('test_entity_001', 'bot', 'Test Bot', 'Bot for testing', 'test_admin_001', true),
('test_entity_002', 'integration', 'Test Integration', 'Integration for testing', 'test_admin_001', true)
ON CONFLICT (id) DO NOTHING;

-- Test API keys (hashed placeholder values)
INSERT INTO api_keys (id, entity_id, key_hash, description, expires_at, created_by) VALUES
('test_apikey_001', 'test_entity_001', '$argon2id$v=19$m=19456,t=2,p=1$placeholder', 'Test key 1', NOW() + INTERVAL '365 days', 'test_admin_001'),
('test_apikey_002', 'test_entity_002', '$argon2id$v=19$m=19456,t=2,p=1$placeholder', 'Test key 2', NOW() + INTERVAL '365 days', 'test_admin_001')
ON CONFLICT (id) DO NOTHING;
