- Rustchat target path: `backend/src/api/ws.rs`
- Required behavior: On websocket connect, subscribe user to all member channels so `posted` events arrive immediately across channels.
- Current gap: v1 websocket initializes with `subscribe_channels=false`, unlike v4 path and Mattermost membership-scoped delivery.
- Planned change: Switched v1 websocket initialization to `subscribe_channels=true`.
- Verification test:
  - Attempted: `cargo test websocket_core -- --nocapture` (blocked by existing unrelated compile error in `src/mattermost_compat/mappers.rs:299`, missing `notify_props` in test initializer).
  - Attempted: `cargo test api::v4::websocket -- --nocapture` (blocked by same unrelated compile error).
  - Completed: `cargo check` passed for backend crate after patch.
- Status: Completed (code change merged), verification partially blocked by pre-existing test compile failure.

- Rustchat target path: `frontend/src/composables/useWebSocket.ts` (no code change expected)
- Required behavior: Continue handling `posted` events into message store for immediate UI updates.
- Current gap: None in event handler path for posted; issue is upstream subscription scope.
- Planned change: No change unless regression found.
- Verification test: Manual smoke with two channels/users to confirm off-channel immediate updates and unread count changes.
- Status: No change required in this iteration.

Compatibility checklist snapshot:
- API Contract: N/A (no REST schema change)
- Realtime Contract: Implemented on Rustchat v1 websocket path (`subscribe_channels=true` at connect)
- Data Semantics: N/A
- Auth and Permissions: Preserved (subscriptions still derived from `channel_members`)
- Client Expectations: Preserved (`posted` handling unchanged on WebUI; mobile-style realtime contract aligned at subscription scope)
- Verification: Backend build check passed; websocket test run blocked by unrelated pre-existing test compile error; manual smoke still required.
