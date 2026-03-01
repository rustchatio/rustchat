# Gap Plan

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

- Rustchat target path: `frontend/src/components/channels/ChannelContextMenu.vue`, `frontend/src/components/layout/ChannelSidebar.vue`, `frontend/src/components/composer/MessageComposer.vue`, `frontend/src/components/layout/AppShell.vue`
- Required behavior: closer web parity for context menu ordering, right-click opening, formatting toggle affordance, and shell spacing.
- Current gap: Extra menu items/order divergence, click-only trigger, icon-only formatting toggle, custom rounded/gapped shell.
- Planned change: Removed non-parity menu items (`Channel Details`, `Delete Channel`), enabled row right-click, changed formatting toggle to `Aa` + chevron, reduced shell rounding/gap.
- Verification test: Frontend build success (`npm run build`).
- Status: Done (functional parity pass)

## Remaining risks

- Notifications and Calls settings content is improved but still not fully pixel/row identical to latest Mattermost desktop in all sub-rows and microcopy.
- Full parity screenshot regression suite is not yet wired in CI for these surfaces.
- `./scripts/mm_mobile_smoke.sh` currently fails locally because the target server is not running with expected compatibility header (`X-MM-COMPAT: 1`).

## Test evidence

- Backend tests executed:
  - `cargo test --test api_v4_mobile_presence --test api_v4_channel_member_routes --test api_categories` (pass)
- Frontend validation executed:
  - `npm run build` in `frontend/` (pass)
- Compatibility smoke executed:
  - `./scripts/mm_mobile_smoke.sh` (failed in current environment: missing compatibility header from `http://localhost:3000/api/v4/system/ping`)
