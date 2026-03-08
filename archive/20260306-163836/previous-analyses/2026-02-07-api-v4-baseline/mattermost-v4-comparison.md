# Mattermost API v4 Comparison

This document summarizes RustChat's current `/api/v4` compatibility position relative to Mattermost v4 and avoids stale per-endpoint claims that drift quickly.

## Last Verified

- Date: **2026-02-07**
- Verified artifacts:
  - `backend/src/api/v4/mod.rs`
  - `docs/api_v4_compatibility_report.md`
  - `tools/mm-compat/output/endpoints_baseline.json`
  - `tools/mm-compat/output/endpoints_final.json`
  - `tools/mm-compat/output/priority_report.md`

## Comparison Scope

- Mattermost OpenAPI baseline set (`endpoints_baseline.json`): **570** method+path entries.
- Combined priority/discovery set (`endpoints_final.json`): **632** entries.
- Static mobile-reference extraction (`endpoints_static.json`): **87** entries.

`endpoints_final.json` is a prioritization/discovery artifact, not proof of full semantic parity.

## Current Parity Statement

- RustChat exposes a broad `/api/v4` route surface across users/teams/channels/posts/files/system/plugins/threads and calls plugin namespaces.
- Compatibility for core client flows (login, bootstrap, channels, posts, websocket, calls handshake) is implemented.
- Many enterprise/admin surfaces are present but still partial or placeholder.
- Unsupported flows should return explicit Mattermost-style `501` rather than silent success stubs.

## High-Impact Endpoint Status (Verified)

| Endpoint | Status |
| --- | --- |
| `POST /api/v4/users/login` | Implemented |
| `GET /api/v4/users/me` | Implemented |
| `GET /api/v4/users/me/teams` | Implemented |
| `GET /api/v4/users/me/teams/{team_id}/channels` | Implemented |
| `GET /api/v4/channels/{channel_id}/posts` | Implemented |
| `POST /api/v4/posts` | Implemented |
| `GET /api/v4/config/client?format=old` | Implemented |
| `GET /api/v4/license/client` | Implemented |
| `GET /api/v4/websocket` | Implemented |
| `GET /api/v4/plugins` | Implemented (calls plugin state-aware response) |
| `GET /api/v4/plugins/{plugin_id}` | Implemented for `com.mattermost.calls` |
| `GET /api/v4/plugins/statuses` | Implemented |
| `GET /api/v4/plugins/webapp` | Implemented when calls enabled |
| `POST /api/v4/plugins` | Explicit `501` |
| `POST /api/v4/plugins/install_from_url` | Explicit `501` |
| `DELETE /api/v4/plugins/{plugin_id}` | Explicit `501` |
| `POST /api/v4/plugins/{plugin_id}/enable` | Explicit `501` |
| `POST /api/v4/plugins/{plugin_id}/disable` | Explicit `501` |
| `POST /api/v4/actions/dialogs/open` | Explicit `501` |
| `POST /api/v4/actions/dialogs/submit` | Explicit `501` |
| `POST /api/v4/actions/dialogs/lookup` | Explicit `501` |

## Version and Fallback Behavior

- Compatibility version reported by system endpoints: `10.11.10`.
- `/api/v4` router fallback returns Mattermost-style `501 Not Implemented`.
- `/api/v4/*` responses include `X-MM-COMPAT: 1`.

## Detailed Follow-Up Reference

For detailed stub/partial endpoint inventory and test coverage notes, use:

- `docs/api_v4_compatibility_report.md`
