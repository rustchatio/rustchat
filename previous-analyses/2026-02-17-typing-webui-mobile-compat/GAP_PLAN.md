# Gap Plan

## Work items

- Rustchat target path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/websocket.rs`
- Required behavior: Emit websocket typing events as `typing` and `stop_typing` with `broadcast.channel_id`.
- Current gap: Server emitted `user_typing` and `user_typing_stop`.
- Planned change: Map internal typing envelopes to Mattermost-standard event names.
- Verification test: `cargo check --lib` in `/Users/scolak/Projects/rustchat/backend` (pass).
- Status: Done.

- Rustchat target path: `/Users/scolak/Projects/rustchat/frontend/src/composables/useWebSocket.ts`
- Required behavior: Resolve channel id from `broadcast.channel_id` for typing events and handle `stop_typing` events.
- Current gap: WebUI only used top-level `channel_id`/`data.channel_id` and did not handle `stop_typing` event name.
- Planned change: Normalize `broadcast` fields and route typing/stop_typing through broadcast-aware channel resolution.
- Verification test: `npm run build` in `/Users/scolak/Projects/rustchat/frontend` (pass).
- Status: Done.

## Completed compatibility checks

- Request action compatibility (`user_typing`): verified.
- Event name compatibility (`typing`, `stop_typing`): fixed.
- Payload compatibility (`data.user_id`, `data.parent_id`, `broadcast.channel_id`): fixed and verified in code.

## Remaining risks

- No dedicated automated unit test yet for typing event name mapping in `map_envelope_to_mm`.
- End-to-end runtime validation (WebUI <-> mobile) still needed in a live environment.

