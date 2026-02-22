# GAP_PLAN

- Rustchat target path: `backend/src/api/v4/system.rs:136`
- Required behavior: `/api/v4/notifications/test` should surface failure as an API error (non-200) when push test cannot be sent, matching Mattermost semantics.
- Current gap: Rustchat returns HTTP 200 with `{status:"OK", error:"..."}` even on push failure/no config, causing mobile UI to show success because it only checks transport errors.
- Planned change: Return `AppError`/HTTP error when `send_push_to_user` returns `Err(...)`; optionally return a non-OK response when zero devices are registered.
- Verification test: Mobile test-notification button shows error state when push pipeline is not configured; API contract test asserts non-200 on test failure.
- Status: High priority

- Rustchat target path: `backend/src/api/v4/system.rs:524`
- Required behavior: `/api/v4/system/ping` should honor `device_id` query and return `CanReceiveNotifications` like Mattermost.
- Current gap: Rustchat ignores `device_id` and never returns `CanReceiveNotifications`, reducing mobile push verification compatibility and diagnostics.
- Planned change: Add `device_id` query handling and return a compatibility value (`true/verified/not_available/unknown` equivalent used by mobile) based on push proxy configuration/test.
- Verification test: Ping with `device_id` includes `CanReceiveNotifications` field; mobile push proxy warning behavior matches Mattermost expectations.
- Status: Medium priority

- Rustchat target path: `backend/src/api/v4/users.rs:975`
- Required behavior: Device registration failures must be observable so push debugging is possible.
- Current gap: `attach_device` swallows DB write errors (`let _ = ...execute(...).await`) and still returns `{"status":"OK"}`.
- Planned change: Log DB errors explicitly and/or return an error in debug mode; add metrics/logging for token registration success/failure counts.
- Verification test: Simulated DB failure is visible in logs/API; successful attach writes `user_devices` row.
- Status: Medium priority

- Rustchat target path: Runtime configuration (`.env`, `docker-compose.yml`, `push-proxy`)
- Required behavior: At least one push delivery path must be configured (push-proxy FCM/APNS or backend direct FCM fallback) for test notifications and real pushes to work.
- Current gap: Local `.env` only sets `RUSTCHAT_PUSH_PROXY_URL`; no `FIREBASE_PROJECT_ID`/`FIREBASE_KEY_PATH` and no APNS keys are configured, so `push-proxy` starts with FCM/APNS disabled and Rustchat fallback is also unconfigured.
- Planned change: Configure Firebase service account (Android) and APNS keys (iOS) in env/secrets, then validate with proxy health + test push.
- Verification test: `POST /api/v4/notifications/test` delivers to a registered device; push-proxy logs show FCM/APNS client initialized and successful send.
- Status: Highest priority (environmental blocker)
