# Server Findings

## Upstream evidence (`../mattermost`)

1. OpenAPI exposes required endpoints:
- `../mattermost/api/v4/source/system.yaml:24` (`/api/v4/system/ping`)
- `../mattermost/api/v4/source/system.yaml:444` (`/api/v4/config/client`)
- `../mattermost/api/v4/source/system.yaml:595` (`/api/v4/license/client`)

2. Server registers `system/ping` route in API init:
- `../mattermost/server/channels/api4/system.go:42`

3. `getSystemPing` response includes `status` and client bootstrap fields:
- `../mattermost/server/channels/api4/system.go:145-153`

## Rustchat current behavior evidence

1. Current active system router is empty:
- `backend/src/api/v4/system.rs:1-6`

2. Current plugins/groups/access_control/custom_profile/reports routers are empty stubs too:
- `backend/src/api/v4/plugins.rs:1-6`
- `backend/src/api/v4/groups.rs:1-6`
- `backend/src/api/v4/access_control.rs:1-6`
- `backend/src/api/v4/custom_profile.rs:1-6`
- `backend/src/api/v4/reports.rs:1-6`

3. Last commit added only stubs for these files:
- `git show --name-status 259f42d`

4. Prior commit moved real implementations to archive path:
- `git show --name-status 2f5076d` shows rename from `backend/src/api/v4/system.rs` to `archive/20260306-163915-remaining-dirty/backend/src/api/v4/system.rs` (and same pattern for related routers).

## Known good local evidence (archived files)

1. Archived system router includes required routes:
- `archive/20260306-163915-remaining-dirty/backend/src/api/v4/system.rs:18-20`

2. Archived plugins router includes plugin endpoints:
- `archive/20260306-163915-remaining-dirty/backend/src/api/v4/plugins.rs:25-46`

3. Archived calls plugin module contains full event/compat logic:
- `archive/20260306-163915-remaining-dirty/backend/src/api/v4/calls_plugin/mod.rs` (full module, 4239 lines)
