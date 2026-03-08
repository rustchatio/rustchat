# Gap Plan

- Rustchat target path:
  - `frontend/src/components/channel/MessageList.vue`
  - `frontend/src/components/channel/MessageItem.vue`
  - `frontend/src/views/main/ChannelView.vue`
  - `frontend/src/components/layout/GlobalHeader.vue`
  - `frontend/src/stores/messages.ts`
  - `frontend/src/stores/unreads.ts`
  - `frontend/src/composables/useWebSocket.ts`
  - `backend/src/api/v4/posts.rs`
  - `backend/src/api/v4/config_client.rs`
  - `backend/src/api/v4/system.rs`
  - `backend/src/api/posts.rs` (if v1 edit path remains used by current WebUI)

- Required behavior:
  - Day separators for message lists (web parity with Mattermost behavior).
  - Global post-edit policy support: disabled, unlimited enabled, or time-limited edit window.
  - Edited marker shown when post has been edited and update reflected immediately.
  - Reactions toggle correctly (second click from same user removes reaction and decrements count).
  - Bell/unread dot clears after viewing unread messages.

- Current gap:
  - WebUI currently renders only time stamps in message rows and has no day separators.
  - Edit path has no configurable time-limit enforcement and no edited indicator in message row.
  - `MessageItem` emits `update` but parent chain does not handle it.
  - Header uses a different unread store (`features/unreads`) than the rest of app (`stores/unreads`).

- Planned change:
  - Implement date separator rendering in message list.
  - Add edited metadata into message model/state and render edited badge.
  - Wire immediate local update flow for edits in parent/store.
  - Unify header unread source with the active unread store.
  - Implement PostEditTimeLimit contract on backend edit endpoints and surface config/client keys.

- Verification test:
  - Frontend build: `cd frontend && npm run build`
  - Backend quality gates: `cd backend && cargo clippy --all-targets --all-features -- -D warnings && cargo test --no-fail-fast -- --nocapture`
  - Manual contract checks:
    - `curl -si -X PUT "$BASE/api/v4/posts/$POST_ID" -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d '{"id":"'$POST_ID'","message":"edited"}'`
    - `curl -s "$BASE/api/v4/config/client?format=old" -H "Authorization: Bearer $TOKEN" | jq '.PostEditTimeLimit,.AllowEditPost'`

- Status:
  - Planned (awaiting SPEC approval gate)
