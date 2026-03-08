## Gap 1: Manual flag not persisted in status lifecycle
- Required behavior: status endpoints and websocket lifecycle should produce consistent `manual` semantics.
- Rustchat before fix:
  - Presence persistence updated only `users.presence` + `last_login_at`.
  - `GET /api/v4/users/*/status` mostly returned `manual: false` regardless of last persisted status write source.
- Implementation:
  - Added migration `backend/migrations/20260217000001_add_presence_manual.sql`.
  - Updated websocket/core presence persistence to write `presence_manual`.
  - Updated `/api/v4/users/status/ids`, `/api/v4/users/{id}/status`, and `/api/v4/users/me/status` to read and return `presence_manual`.
- Status: Implemented.

## Gap 2: Status update write paths need consistent manual derivation
- Required behavior: manual should be computed from status intent for user-issued updates.
- Implementation:
  - Added helper `status_is_manual(status)` in `backend/src/api/websocket_core.rs`.
  - Websocket `presence` command now derives manual from status value.
  - `/api/v4/users/{id}/status` now derives and persists manual based on status value.
  - Preferences status update path writes `presence_manual` whenever `presence` is updated.
- Status: Implemented.

## Gap 3: Regression guard for requested mobile lifecycle
- Required behavior: after websocket close, status transitions to `offline` non-manual; after reconnect, transitions to `online` non-manual.
- Implementation:
  - Added integration test `backend/tests/api_v4_mobile_presence.rs`.
- Verification run:
  - `cargo test --test api_v4_mobile_presence -- --nocapture`
  - Result: test logic compiles, but execution blocked in this environment due missing local Postgres (`Connection refused` from `tests/common/mod.rs`).
- Status: Implemented with environment-blocked execution.

## Gap 4: Background websocket churn flips users offline too quickly
- Required behavior: short mobile background intervals should not immediately force offline presence.
- Upstream evidence:
  - Mobile app schedules websocket close after 15 seconds in background (`../mattermost-mobile/app/managers/websocket_manager.ts:23`, `:281-287`).
  - Server marks users offline when websocket is gone (`../mattermost/server/channels/app/platform/web_hub.go:629`).
- Rustchat mitigation:
  - Added configurable disconnect grace lookup in `backend/src/api/websocket_core.rs` (`get_presence_offline_grace_seconds`).
  - Updated `set_offline_if_last_connection` to delay offline write and re-check active connection count before applying offline.
  - Config key: `server_config.site.mobile_presence_disconnect_grace_seconds` (default `300`, `0` keeps immediate offline behavior).
- Verification run:
  - `cargo test --test api_v4_mobile_presence --no-run` -> passed.
  - `cargo test --test api_v4_mobile_presence -- --nocapture` -> blocked by missing local Postgres in this environment.
- Remaining risk:
  - This does not keep a real websocket alive in background; it only prevents immediate offline status flips during short background intervals.
