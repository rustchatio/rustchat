# Gap Plan

## Items

### S1. User account menu parity (status dropdown screenshot)
- Rustchat target path: `frontend/src/components/layout/GlobalHeader.vue`, `frontend/src/components/modals/SetStatusModal.vue`
- Required behavior: Mattermost-style account menu order and wording; `Set custom status`; Online/Away/Do not disturb/Offline rows; DND row with secondary text and duration submenu; Profile and Log out rows.
- Current gap: Rustchat menu is custom card layout with 2x2 availability grid and different labels (`Busy`, `Invisible`) in `frontend/src/components/layout/GlobalHeader.vue:189-205`; custom status text is `Set a custom status` in `frontend/src/components/layout/GlobalHeader.vue:180`; DND duration lives in a separate modal with non-MM options (`4 hours`, `Today`) in `frontend/src/components/modals/SetStatusModal.vue:28-34`.
- Planned change: Replace current dropdown with menu-item components matching Mattermost order; change labels and separators; move DND durations into submenu-style interaction; keep current modal only for custom status text/emoji edit.
- Verification test: Playwright visual test for menu layout and labels; interaction test for DND submenu durations and selected-state checkmarks.
- Status: Planned

### S2. Settings shell and Display theme editor parity
- Rustchat target path: `frontend/src/components/settings/SettingsModal.vue`
- Required behavior: Settings shell with tabs `Notifications`, `Display`, `Sidebar`, `Advanced`; Display tab with collapsed rows and edit mode; theme editor with premade/custom modes and Save/Cancel.
- Current gap: Current tabs are `Settings`, `Status`, `Preferences` in `frontend/src/components/settings/SettingsModal.vue:50-54`; preferences content is custom theme/font/density layout in `frontend/src/components/settings/SettingsModal.vue:455-555`; no MM row-based edit pattern.
- Planned change: Rebuild modal into Mattermost tab model and create reusable `SettingItemMin`/`SettingItemMax` row components; implement Display rows in MM order; implement theme expanded editor with save/cancel transaction semantics.
- Verification test: Playwright screenshot tests for Display tab collapsed and expanded theme modes; store-level tests for save/cancel behavior.
- Status: Planned

### S3. Channel context menu parity (sidebar right-click/3-dot menu)
- Rustchat target path: `frontend/src/components/layout/ChannelSidebar.vue`, `frontend/src/stores/unreads.ts`, `frontend/src/api/channels.ts`, `frontend/src/api/preferences.ts`, `frontend/src/features/channels/repositories/channelRepository.ts`
- Required behavior: Channel menu entries in order: Mark as Read/Unread, Favorite, Mute Channel, Move to..., Copy Link, Add Members, Leave Channel.
- Current gap: Sidebar rows only provide inline mark-as-read hover action in `frontend/src/components/layout/ChannelSidebar.vue:293-300`; no context menu UI or move/favorite/mute actions; unread store uses custom `/channels/{id}/read` flow in `frontend/src/stores/unreads.ts:60-63`.
- Planned change: Add menu trigger per row and implement full MM item ordering/separators; wire actions to MM-compatible endpoints for notify props, preferences, category moves, and leave/add-members actions.
- Verification test: Component interaction tests for menu item visibility by channel type/permissions; backend integration tests for favorite/mute/move operations; E2E tests for `Move to...` submenu.
- Status: Planned

### S4. Calls settings plugin page parity
- Rustchat target path: `frontend/src/components/settings/SettingsModal.vue`, new `frontend/src/components/settings/calls/*`, `frontend/src/stores/calls.ts`
- Required behavior: Calls page under plugin preferences with `Audio devices` and `Video devices` rows and edit affordances.
- Current gap: No plugin-preferences section in settings nav (`frontend/src/components/settings/SettingsModal.vue:50-54`); no audio/video device settings UI found (`rg` only finds calls codec debug line at `frontend/src/stores/calls.ts:598`).
- Planned change: Add plugin preference group in left nav; add Calls settings screen with enumerated input/output devices and persisted selection; integrate with calls media pipeline.
- Verification test: Unit tests for device list and persisted selection; manual verification in browser permissions flow.
- Status: Planned

### S5. Advanced settings parity
- Rustchat target path: `frontend/src/components/settings/SettingsModal.vue`, new `frontend/src/components/settings/advanced/*`, `frontend/src/stores/preferences.ts`, `backend/src/api/v4/users/preferences.rs`
- Required behavior: MM advanced rows: send-on-ctrl-enter, enable formatting, join/leave messages, performance debugging, unread scroll position, sync drafts.
- Current gap: No advanced tab; only basic profile/status/preferences content in `frontend/src/components/settings/SettingsModal.vue:338-555`.
- Planned change: Implement Advanced tab with row-based edit controls and preference mapping to MM categories/names in `mattermost_preferences`.
- Verification test: API contract tests for preference keys round-trip; UI tests per row save/cancel behavior.
- Status: Planned

### S6. Sidebar settings parity
- Rustchat target path: `frontend/src/components/settings/SettingsModal.vue`, new `frontend/src/components/settings/sidebar/*`, `frontend/src/stores/preferences.ts`
- Required behavior: Two rows only: `Group unread channels separately` and `Number of direct messages to show`.
- Current gap: No Sidebar tab exists in current settings modal (`frontend/src/components/settings/SettingsModal.vue:50-54`).
- Planned change: Add Sidebar tab and both rows with MM key names in stored preferences.
- Verification test: UI tests for row rendering and persisted values; regression test to ensure Direct Messages list respects configured limit.
- Status: Planned

### S7. Display settings long-list parity
- Rustchat target path: `frontend/src/components/settings/SettingsModal.vue`, new `frontend/src/components/settings/display/*`, `frontend/src/stores/theme.ts`, `frontend/src/stores/preferences.ts`
- Required behavior: Display row inventory including Theme, Threaded Discussions, Clock Display, Teammate Name Display, online availability badges, Share last active time, Timezone, Link/Image previews, Message Display, Click to open threads, Channel Display, Quick reactions, Render emoticons, Language.
- Current gap: Current Preferences tab provides custom controls unrelated to MM row inventory (`frontend/src/components/settings/SettingsModal.vue:455-545`).
- Planned change: Implement MM row inventory and map each row to compatible preference/category keys; keep existing theme store internals but present MM-compatible UX.
- Verification test: Snapshot test for row order/labels; API-level preference serialization test for each Display row.
- Status: Planned

### S8. Notifications settings parity
- Rustchat target path: `frontend/src/components/settings/SettingsModal.vue`, `frontend/src/components/settings/NotificationsPanel.vue`, new `frontend/src/components/settings/notifications/*`, `backend/src/api/v4/users/preferences.rs`
- Required behavior: Notifications page with desktop/mobile row + permission tag, desktop sounds, email notifications, keywords sections, auto-responder, troubleshooting card and actions.
- Current gap: Current notifications experience is simplified selects/toggles in `frontend/src/components/settings/NotificationsPanel.vue:70-233`; settings modal has only a single desktop notification enable row in `frontend/src/components/settings/SettingsModal.vue:547-554`.
- Planned change: Replace simplified panel with MM-like row structure; add permission status tag logic; add troubleshooting notice actions; wire each row to MM preference keys and server capabilities.
- Verification test: Playwright tests for notification rows and permission-tag states; backend preference integration tests for notify keys.
- Status: Planned

### S9. Composer formatting toggle parity
- Rustchat target path: `frontend/src/components/composer/MessageComposer.vue`, `frontend/src/components/composer/FormattingToolbar.vue`, new `frontend/src/components/composer/ToggleFormattingBar.vue`
- Required behavior: Bottom `Aa` toggle that hides/shows formatting bar; tooltip includes show/hide formatting shortcut; controls remain aligned with file/emoji/send actions.
- Current gap: Formatting toolbar is always visible at top in `frontend/src/components/composer/MessageComposer.vue:325-331`; no Aa toggle exists in `frontend/src/components/composer/FormattingToolbar.vue:24-51`.
- Planned change: Move formatting controls to hideable section controlled by new toggle component in footer action row; add shortcut handling (`Ctrl+Alt+T` / `Cmd+Opt+T`) and tooltip text.
- Verification test: Component keyboard-interaction tests; Playwright test for toggle and tooltip string.
- Status: Planned

### S10. Main channel workspace shell parity
- Rustchat target path: `frontend/src/components/layout/AppShell.vue`, `frontend/src/components/layout/GlobalHeader.vue`, `frontend/src/components/channel/ChannelHeader.vue`
- Required behavior: MM-like shell proportions and header affordances so settings/menu/context interactions appear in expected placements.
- Current gap: Rustchat shell uses custom spacing/rounded panels and does not mirror MM default chrome in `frontend/src/components/layout/AppShell.vue:14-54`; global header controls differ in icon set and spacing in `frontend/src/components/layout/GlobalHeader.vue:88-137`.
- Planned change: Keep existing functional layout but adjust shell spacing and control placements where needed to support parity for settings/menu behavior; defer full visual restyle until functional parity is complete.
- Verification test: Reference screenshot diff checks on desktop width breakpoints.
- Status: Planned

### S11. API contract alignment needed by menu/settings features
- Rustchat target path: `backend/src/api/v4/users/preferences.rs`, `backend/src/api/v4/users.rs`, `backend/src/api/v4/status.rs`, `backend/src/api/v4/channels.rs`, `backend/src/api/v4/posts.rs`, `backend/src/api/v4/categories.rs`
- Required behavior: MM payload/response/error semantics for preferences, status, notify_props, set_unread, and category move/order endpoints.
- Current gap: Duplicate status route implementations exist in `backend/src/api/v4/users.rs:114-117` and `backend/src/api/v4/status.rs:24-29`; custom status endpoints in users router are currently placeholder (`backend/src/api/v4/users.rs:2491-2526`); `set_post_unread` does not read MM request body (`collapsed_threads_supported`) and returns local shape tied to `last_read_id` in `backend/src/api/v4/posts.rs:671-731`.
- Planned change: Consolidate/normalize status handler contract; implement full custom-status persistence and retrieval semantics; align set-unread request parsing and response schema with MM state object; keep canonical categories endpoints under `/users/{user_id}/teams/{team_id}/channels/categories`.
- Verification test: Extend `backend/tests/api_v4_mobile_presence.rs`, `backend/tests/api_v4_posts.rs`, `backend/tests/api_categories.rs`, and add new contract tests for custom status and notify props.
- Status: Planned

## Implementation phases

- Rustchat target path: `frontend/src/components/settings/*`, `frontend/src/components/layout/*`, `frontend/src/components/composer/*`, `backend/src/api/v4/*`, `backend/tests/*`, `frontend/e2e/*`
- Required behavior: Deliver parity incrementally without breaking existing flows.
- Current gap: Many screenshot surfaces require coordinated frontend and backend updates.
- Planned change: Phase 1: status menu + status API cleanup; Phase 2: settings shell and shared row components; Phase 3: Display/Sidebar/Advanced tabs; Phase 4: Notifications + troubleshooting block; Phase 5: channel context menu + move/mute/favorite wiring; Phase 6: composer formatting toggle; Phase 7: calls settings plugin panel; Phase 8: final visual regression plus mobile contract gate (`preferences`, `notify_props`, `set_unread`, categories, websocket events).
- Verification test: Run `cargo test` for v4 suites and targeted new tests; run `frontend test:e2e` with parity snapshots.
- Status: Planned

## Risks and blockers

- Rustchat target path: `../rustchat-mobile`, `backend/src/api/v4/*`, `backend/tests/*`
- Required behavior: Keep backend contracts fully compatible with mobile consumers while implementing web screenshot parity.
- Current gap: Mobile UI patterns differ from desktop, but mobile hard-depends on shared contracts:
  - Preferences arrays + category/key semantics (`display_settings`, `sidebar_settings`, `advanced_settings`, `theme`).
  - Status/custom-status routes and legacy custom-status-recent delete endpoint.
  - Channel `notify_props` mutation behavior for mute/unmute.
  - Post `set_unread` request body (`collapsed_threads_supported`) and response semantics.
  - Team-scoped channel category routes and websocket category events.
- Planned change: Treat S11 as a cross-cutting dependency for S1-S8; add explicit mobile contract tests for each endpoint/event before merging final parity.
- Verification test: Extend API test suites with mobile-shaped request/response fixtures and websocket event handling assertions.
- Status: Open (no external blocker)
