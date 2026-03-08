# Gap Plan

## Closeout status

- Contract closure: Complete
- Mobile compatibility blockers: Closed
- Remaining parity scope: optional cross-platform (Linux) screenshot baseline expansion

## Completed checks

- Rustchat target path: `backend/src/api/v4/status.rs`
- Required behavior: `status/ids` accepts raw array and returns status array; custom-status routes accept `me`.
- Current gap: Previously required wrapped `user_ids` payload and map response; custom-status handlers rejected `me`.
- Planned change: Implemented dual-input parsing (`raw` + wrapped), strict invalid/empty validation, status array response, and `me` resolution helper for custom status endpoints.
- Verification test: `backend/tests/api_v4_mobile_presence.rs` (`status_ids_accepts_raw_and_wrapped_payloads`, `custom_status_me_routes_are_supported_and_scoped`).
- Status: Done

- Rustchat target path: `backend/src/api/v4/channels.rs`
- Required behavior: notify_props update returns status-only OK payload.
- Current gap: Returned full channel-member object.
- Planned change: Return `{"status":"OK"}` after update.
- Verification test: `backend/tests/api_v4_channel_member_routes.rs` (`mm_channel_member_routes`).
- Status: Done

- Rustchat target path: `backend/src/api/v4/categories.rs`, `backend/src/api/v4/users/sidebar_categories.rs`, `backend/src/api/v4/users.rs`
- Required behavior: categories PUT accepts raw array; GET order route available.
- Current gap: Wrapped payload only in primary route and no GET order route.
- Planned change: Added untagged payload compatibility (`raw` + wrapped), added `get_category_order_internal`, mounted GET on `/channels/categories/order`.
- Verification test: `backend/tests/api_categories.rs` (`test_sidebar_categories` covers raw + wrapped update and GET order).
- Status: Done

- Rustchat target path: `backend/src/api/v4/status.rs`
- Required behavior: status websocket event reaches other listeners.
- Current gap: Event used `broadcast.user_id` direct-target semantics, suppressing fan-out.
- Planned change: Emit `status_change` with `broadcast: None` for broad delivery.
- Verification test: `backend/tests/api_v4_mobile_presence.rs` (`status_change_event_reaches_other_connected_users`).
- Status: Done

- Rustchat target path: `frontend/src/api/users.ts`, `frontend/src/stores/auth.ts`, `frontend/src/components/layout/GlobalHeader.vue`
- Required behavior: use `status` payload semantics and working DND duration updates.
- Current gap: UI sent `presence` and non-effective `duration` for DND.
- Planned change: Normalize outgoing payload to `status`, emit `dnd_end_time`, align DND options.
- Verification test: Frontend build success (`npm run build`), backend DND/status integration tests pass.
- Status: Done
- Follow-up patch: Fixed DND submenu mapping bug (`Tomorrow` now maps to `tomorrow` key in `GlobalHeader.vue`).

- Rustchat target path: `frontend/src/api/channels.ts`, `frontend/src/features/channels/repositories/channelRepository.ts`
- Required behavior: frontend matches changed notify_props response and categories update body shape.
- Current gap: Notify props typed as `ChannelMember` response and categories updates posted wrapped object.
- Planned change: Notify props API typed as status response; repository no longer parses member response; categories updates post raw array.
- Verification test: Frontend build success (`npm run build`).
- Status: Done

- Rustchat target path: `frontend/src/components/settings/SettingsModal.vue`, `frontend/src/stores/ui.ts`, `frontend/src/components/settings/SettingItemMin.vue`
- Required behavior: settings tab state bound to store + plugin grouping; notifications row slot rendering works.
- Current gap: Modal ignored `ui.settingsTab`, lacked plugin grouping structure, and `SettingItemMin` had no named slots used by Notifications tab.
- Planned change: Bound modal open/tab selection to store, added `PLUGIN PREFERENCES` section for Calls, exported `settingsTab` from UI store, added `icon`/`extra` slot support.
- Verification test: Frontend build success (`npm run build`).
- Status: Done

- Rustchat target path: `frontend/src/components/settings/notifications/NotificationsTab.vue`, `frontend/src/components/settings/calls/CallsTab.vue`, `frontend/src/components/settings/SettingsModal.vue`
- Required behavior: notifications/calls settings content and layout align closer to Mattermost row naming, descriptions, and troubleshooting/actions.
- Current gap: previous tabs diverged in section naming, row labels, card actions, and calls row structure.
- Planned change: Reworked Notifications rows (`Desktop and mobile notifications`, sounds, email, keywords, highlighted keywords, auto-replies), added `Learn more` and troubleshooting actions (including `/api/v4/notifications/test`), refactored Calls tab to two-row `Audio devices`/`Video devices` structure with Mattermost wording, and refactored `SettingsModal` shell to top-level `Settings` header layout closer to Mattermost geometry.
- Verification test: Frontend build success (`npm run build`).
- Status: Done (content + shell parity pass)

- Rustchat target path: `frontend/src/components/channels/ChannelContextMenu.vue`, `frontend/src/components/layout/ChannelSidebar.vue`, `frontend/src/components/composer/MessageComposer.vue`, `frontend/src/components/layout/AppShell.vue`
- Required behavior: closer web parity for context menu ordering, right-click opening, formatting toggle affordance, and shell spacing.
- Current gap: Extra menu items/order divergence, click-only trigger, icon-only formatting toggle, custom rounded/gapped shell.
- Planned change: Removed non-parity menu items (`Channel Details`, `Delete Channel`), enabled row right-click, changed formatting toggle to `Aa` + chevron, reduced shell rounding/gap.
- Verification test: Frontend build success (`npm run build`).
- Status: Done (functional parity pass)

## Remaining risks

- Screenshot-diff gating is wired in CI on `macos-latest`; linux screenshot baselines are not yet tracked.
- Compatibility smoke scripts are currently environment-sensitive (expected server version and auth token extraction path) and should stay aligned when server defaults change.

## Test evidence

- Backend tests executed:
  - `cargo test --test api_v4_mobile_presence --test api_v4_channel_member_routes --test api_categories` (pass)
- Frontend validation executed:
  - `npm run build` in `frontend/` (pass)
- UI parity screenshots executed:
  - `npm run test:e2e:settings-parity:update` (generated baseline snapshots)
  - `npm run test:e2e:settings-parity` (pass, diff gate)
  - Baselines committed:
    - `frontend/e2e/settings_parity.spec.ts-snapshots/settings-notifications-chromium-darwin.png`
    - `frontend/e2e/settings_parity.spec.ts-snapshots/settings-display-chromium-darwin.png`
    - `frontend/e2e/settings_parity.spec.ts-snapshots/settings-sidebar-chromium-darwin.png`
    - `frontend/e2e/settings_parity.spec.ts-snapshots/settings-advanced-chromium-darwin.png`
    - `frontend/e2e/settings_parity.spec.ts-snapshots/settings-calls-chromium-darwin.png`
- CI coverage:
  - `.github/workflows/frontend-ci.yml` includes `settings-visual-parity` job that installs Chromium and runs `npm run test:e2e:settings-parity`.
- Compatibility smoke executed:
  - `BASE=http://localhost:3000 ./scripts/mm_mobile_smoke.sh` (pass)
  - `BASE=http://localhost:3000 LOGIN_ID=compat_smoke_1772369282 PASSWORD=Password123! ./scripts/mm_compat_smoke.sh` (pass, including authenticated checks)
- Verification harness updates:
  - `scripts/mm_mobile_smoke.sh`: expected version check made configurable via `EXPECTED_MM_VERSION` (default `10.11.10`).
  - `scripts/mm_compat_smoke.sh`: default `BASE` set to `http://localhost:3000`, version check parameterized, config check aligned to `?format=old`, and login token parsing made case-insensitive with JSON fallback.
