# GAP_PLAN

- Rustchat target path: `backend/src/api/v4/system.rs:136`
- Required behavior: `/api/v4/notifications/test` should surface failure as an API error (non-200) when push test cannot be sent, matching Mattermost semantics.
- Implemented: Endpoint now returns API errors (non-200) for `Err(...)`, zero devices, and zero-sent cases with registered devices; includes diagnostic logs for registered devices/tokens/platforms and push config hints (`has_push_proxy_url`, `has_fcm_db_config`, `has_fcm_env_config`).
- Remaining risk: `send_push_to_user` still collapses some per-device failures into `sent_count=0`; endpoint now surfaces that as `ExternalService`, but exact per-device cause still depends on backend/push-proxy logs.
- Verification test: Manual mobile validation still required. Unit tests added for zero-device vs delivery-unavailable classification in `backend/src/api/v4/system.rs`.
- Status: Implemented (behavioral gap closed)

- Rustchat target path: `backend/src/api/v4/system.rs:524`
- Required behavior: `/api/v4/system/ping` should honor `device_id` query and return `CanReceiveNotifications` like Mattermost.
- Implemented: `device_id` query is now accepted and `CanReceiveNotifications` is returned on both normal and `format=old` ping responses. Value is inferred from configuration (`unknown` if push path appears configured, `false` otherwise) and logged.
- Remaining risk: This is not a full upstream-equivalent live proxy send; it does not return `true` because ping currently does not execute a real test push.
- Verification test: Unit tests added for field omission/no-device-id and configured/unconfigured value mapping in `backend/src/api/v4/system.rs`.
- Status: Implemented (partial parity; diagnostic-compatible)

- Rustchat target path: `backend/src/api/v4/users.rs:975`
- Required behavior: Device registration failures must be observable so push debugging is possible.
- Implemented: `attach_device` now logs parsed mobile fields (`device_notification_disabled`, `mobile_version`) and logs DB insert/update success or SQL errors instead of silently discarding the result.
- Remaining risk: Endpoint still returns `{"status":"OK"}` on DB write failure for compatibility; logs must be checked to diagnose failures.
- Verification test: Compile/test pass confirms handler changes build; runtime log validation needed with real mobile registration.
- Status: Implemented (logging/debug gap closed)

- Rustchat target path: Runtime configuration (`.env`, `docker-compose.yml`, `push-proxy`)
- Required behavior: At least one push delivery path must be configured (push-proxy FCM/APNS or backend direct FCM fallback) for test notifications and real pushes to work.
- Current gap: Local `.env` only sets `RUSTCHAT_PUSH_PROXY_URL`; no `FIREBASE_PROJECT_ID`/`FIREBASE_KEY_PATH` and no APNS keys are configured, so `push-proxy` starts with FCM/APNS disabled and Rustchat fallback is also unconfigured.
- Planned change: Configure Firebase service account (Android) and APNS keys (iOS) in env/secrets, then validate with proxy health + test push.
- Verification test: `POST /api/v4/notifications/test` delivers to a registered device; push-proxy logs show FCM/APNS client initialized and successful send.
- Status: Still open (highest priority environmental blocker)

## Completed checks (2026-02-22)

- `/api/v4/notifications/test` failure semantics now return non-200 errors instead of HTTP 200 with embedded error JSON.
- `/api/v4/system/ping` now emits `CanReceiveNotifications` when `device_id` is provided.
- Device registration path logs DB write success/failure and mobile metadata for debugging.
- Push send loop logs explicit `NotConfigured` cause before returning `sent_count=0`.

## Test evidence

- Command: `cargo test --manifest-path backend/Cargo.toml api::v4::system::tests::`
- Result: `6 passed; 0 failed` for `api::v4::system::tests` (filter also executed integration test binaries with 0 matching tests).
