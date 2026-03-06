# Architecture Gaps

1. Split state architecture is causing unread-indicator desync.
- `GlobalHeader.vue` uses `../../features/unreads` store id `unreadStore`.
- Messaging/websocket/sidebar flow uses `../../stores/unreads` store id `unreads`.
- This creates independent unread totals for bell-dot vs actual channel read flow.

2. Message list rendering is missing day grouping abstraction.
- `MessageList.vue` renders direct `v-for` over messages.
- No pre-processed list items (`date` + `message`) like upstream web/mobile pipeline.

3. Edit behavior lacks a single contract source.
- Backend accepts edits without configured global time policy.
- Config client payload does not expose edit-limit values used by clients.
- Frontend has no `edited` marker path bound to `edited_at`/`edit_at`.

4. Edit immediacy chain is incomplete in current WebUI.
- `MessageItem` emits `update` event.
- Parent `MessageList`/`ChannelView` chain does not forward or handle this event.
