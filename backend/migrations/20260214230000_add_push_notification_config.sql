-- Migration: Add FCM push notification configuration
-- Adds columns for Firebase Cloud Messaging configuration

ALTER TABLE server_config
ADD COLUMN IF NOT EXISTS fcm_project_id TEXT,
ADD COLUMN IF NOT EXISTS fcm_access_token TEXT,
ADD COLUMN IF NOT EXISTS apns_key_id TEXT,
ADD COLUMN IF NOT EXISTS apns_team_id TEXT,
ADD COLUMN IF NOT EXISTS apns_bundle_id TEXT,
ADD COLUMN IF NOT EXISTS apns_private_key TEXT;

-- Add comments explaining the columns
COMMENT ON COLUMN server_config.fcm_project_id IS 'Firebase Cloud Messaging project ID for Android push notifications';
COMMENT ON COLUMN server_config.fcm_access_token IS 'Firebase Cloud Messaging service account access token';
COMMENT ON COLUMN server_config.apns_key_id IS 'Apple Push Notification Service key ID';
COMMENT ON COLUMN server_config.apns_team_id IS 'Apple Developer Team ID';
COMMENT ON COLUMN server_config.apns_bundle_id IS 'iOS app bundle identifier';
COMMENT ON COLUMN server_config.apns_private_key IS 'APNS private key for authentication';
