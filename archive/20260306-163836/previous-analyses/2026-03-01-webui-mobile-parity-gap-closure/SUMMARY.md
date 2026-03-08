# Summary

- Topic: WebUI parity + mobile compatibility gap closure (status, categories, notify props, settings/menu/composer alignment)
- Date: 2026-03-01
- Status: Backend/mobile wire-contract gaps are closed and verified; Notifications/Calls settings content and shell parity pass is implemented and compiled.
- Scope: `backend/src/api/v4/*`, websocket status broadcast semantics, and frontend parity-sensitive consumers (`frontend/src/components/*`, `frontend/src/api/*`, `frontend/src/stores/*`)

## Closed contract gaps

- `POST /api/v4/users/status/ids` accepts raw array payload from mobile (`../rustchat-mobile/app/client/rest/users.ts:406`) and returns status array semantics consumed by mobile reducers (`../rustchat-mobile/app/actions/remote/user.ts:370`).
- Custom status routes support `me` path usage (`/users/me/status/custom*`) from mobile.
- `PUT /channels/{channel_id}/members/{user_id}/notify_props` returns `{status: "OK"}` semantics.
- Categories update accepts raw categories array; category order read route exists (`GET /users/{user_id}/teams/{team_id}/channels/categories/order`).
- `status_change` websocket events are fan-out compatible for presence synchronization.

## Verification evidence

- Backend tests:
  - `cargo test --test api_v4_mobile_presence --test api_v4_channel_member_routes --test api_categories` (pass)
- Frontend build:
  - `npm run build` in `frontend/` (pass)
- Settings parity screenshots:
  - `npm run test:e2e:settings-parity` (pass, screenshot diff against committed baselines)
  - Artifacts:
    - `frontend/e2e/settings_parity.spec.ts-snapshots/settings-notifications-chromium-darwin.png`
    - `frontend/e2e/settings_parity.spec.ts-snapshots/settings-display-chromium-darwin.png`
    - `frontend/e2e/settings_parity.spec.ts-snapshots/settings-sidebar-chromium-darwin.png`
    - `frontend/e2e/settings_parity.spec.ts-snapshots/settings-advanced-chromium-darwin.png`
    - `frontend/e2e/settings_parity.spec.ts-snapshots/settings-calls-chromium-darwin.png`
- Compatibility smoke:
  - `BASE=http://localhost:3000 ./scripts/mm_mobile_smoke.sh` (pass)
  - `BASE=http://localhost:3000 LOGIN_ID=compat_smoke_1772369282 PASSWORD=Password123! ./scripts/mm_compat_smoke.sh` (pass)
- CI gate:
  - `.github/workflows/frontend-ci.yml` includes `settings-visual-parity` job (`macos-latest`) running `npm run test:e2e:settings-parity`.

## Remaining gap (non-contract)

- Snapshot baseline is currently darwin/chromium scoped; if a Linux visual gate is needed, a linux baseline set must be generated and added.
