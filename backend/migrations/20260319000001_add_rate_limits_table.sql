-- Rate limit configuration table.
-- Two row types share this table:
--   Limit rows:  window_secs > 0, limit_value = max requests per window
--   Flag rows:   window_secs = 0, limit_value = 1 (enabled) or 0 (disabled)
CREATE TABLE rate_limits (
    key TEXT PRIMARY KEY,
    limit_value INTEGER NOT NULL,
    window_secs INTEGER NOT NULL DEFAULT 60,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

COMMENT ON TABLE rate_limits IS
  'Rate limit config. Rows with window_secs=0 are enabled flags (limit_value 1=on, 0=off). '
  'Rows with window_secs>0 are request limits (limit_value = max requests per window_secs).';

-- Limit rows
INSERT INTO rate_limits (key, limit_value, window_secs) VALUES
    ('auth_ip_per_minute',           20, 60),
    ('auth_user_per_minute',         10, 60),
    ('register_ip_per_minute',       10, 60),
    ('password_reset_ip_per_minute',  5, 60),
    ('websocket_ip_per_minute',      30, 60);

-- Enabled flag rows (window_secs = 0 marks these as flags)
INSERT INTO rate_limits (key, limit_value, window_secs) VALUES
    ('auth_ip_enabled',           1, 0),
    ('auth_user_enabled',         1, 0),
    ('register_ip_enabled',       1, 0),
    ('password_reset_ip_enabled', 1, 0),
    ('websocket_ip_enabled',      1, 0);
