# Gap Plan

## Work items

- Rustchat target path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/websocket.rs`
- Required behavior: Emit websocket typing events as `typing` and `stop_typing` with `broadcast.channel_id`.
- Current gap: Server emitted `user_typing` and `user_typing_stop`.
- Planned change: Map internal typing envelopes to Mattermost-standard event names.
- Verification test: `cargo check --lib` in `/Users/scolak/Projects/rustchat/backend` (pass).
- Status: Done.

- Rustchat target path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/websocket.rs`
- Required behavior: Accept typing actions from mobile/web variants (`user_typing`, `typing`, `typing_start`) and parse channel/thread IDs from multiple payload shapes.
- Current gap: Handler accepted only `action == user_typing` and only `data.channel_id`/`data.parent_id`.
- Planned change: Support additional typing action names and payload fallbacks (`channel_id`, `parent_id`, `thread_root_id` in `data` or top-level).
- Verification test: `cargo check --lib` in `/Users/scolak/Projects/rustchat/backend` (pass).
- Status: Done.

- Rustchat target path: `/Users/scolak/Projects/rustchat/backend/src/realtime/events.rs`
- Required behavior: Deserialize websocket command IDs in both UUID and Mattermost 26-char formats.
- Current gap: `ClientEnvelope.channel_id` and `TypingCommandData.thread_root_id` accepted UUID only.
- Planned change: Add compatibility deserializer using `parse_mm_or_uuid`.
- Verification test: `cargo check --lib` in `/Users/scolak/Projects/rustchat/backend` (pass).
- Status: Done.

- Rustchat target path: `/Users/scolak/Projects/rustchat/frontend/src/composables/useWebSocket.ts`
- Required behavior: WebUI typing sender should match Mattermost websocket command shape used by web/mobile clients.
- Current gap: WebUI sent custom envelope command `typing_start` only.
- Planned change: Send websocket `action: user_typing` with `data.channel_id` and `data.parent_id`.
- Verification test: `npm run build` in `/Users/scolak/Projects/rustchat/frontend` (pass).
- Status: Done.

- Rustchat target path: `/Users/scolak/Projects/rustchat/frontend/src/components/composer/MessageComposer.vue`
- Required behavior: Low-frequency typing activity updates (1-2 seconds) instead of per-keystroke spam.
- Current gap: Typing emit cadence was fixed at 3 seconds and not explicitly guarded for non-empty content.
- Planned change: Emit typing every 2 seconds only when content is non-empty.
- Verification test: `npm run build` in `/Users/scolak/Projects/rustchat/frontend` (pass).
- Status: Done.

- Rustchat target path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/config_client.rs`
- Required behavior: Mattermost mobile must receive typing-related config flags from `/api/v4/config/client?format=old`.
- Current gap: `EnableUserTypingMessages` and `TimeBetweenUserTypingUpdatesMilliseconds` were missing, causing mobile default `EnableUserTypingMessages=false` and no typing sends.
- Planned change: Add `EnableUserTypingMessages=true` and `TimeBetweenUserTypingUpdatesMilliseconds=2000` in legacy config payload.
- Verification test: `cargo check --lib` in `/Users/scolak/Projects/rustchat/backend` (pass).
- Status: Done.

## Completed compatibility checks

- Request action compatibility (`user_typing`): verified and aligned for WebUI sender.
- Event name compatibility (`typing`, `stop_typing`): fixed.
- Payload compatibility (`data.user_id`, `data.parent_id`, `broadcast.channel_id`): fixed and verified in code.
- 26-char ID compatibility for websocket typing command IDs: fixed for envelope command path.
- Mobile config compatibility for typing feature flags: fixed in legacy client config endpoint.

## Remaining risks

- Full backend `cargo test` currently blocked by unrelated pre-existing test compile error in `/Users/scolak/Projects/rustchat/backend/src/mattermost_compat/mappers.rs` (missing `notify_props` field in a test initializer).
- Existing mobile sessions may cache old config; a reconnect/relogin may be required for clients to pick up new typing flags.
- End-to-end runtime validation (mobile <-> mobile and mobile <-> web typing) still required after deploy/restart.

