# Server Findings

## Upstream Mattermost (server/web protocol usage)

- Endpoint or component: Webapp websocket client typing command
- Source path: `/Users/scolak/Projects/mattermost/webapp/platform/client/src/websocket.ts`
- Source lines: `618-623`
- Observed behavior: Web client emits typing as websocket action `user_typing` with `channel_id` and `parent_id`.

- Endpoint or component: Webapp typing event consumer
- Source path: `/Users/scolak/Projects/mattermost/webapp/channels/src/components/msg_typing/msg_typing.tsx`
- Source lines: `23-26`
- Observed behavior: Typing UI listens to event `typing`, then reads `broadcast.channel_id`, `data.parent_id`, and `data.user_id`.

- Endpoint or component: Webapp websocket event constants
- Source path: `/Users/scolak/Projects/mattermost/webapp/platform/client/src/websocket_events.ts`
- Source lines: `4-6`
- Observed behavior: Typing event constant is `Typing = 'typing'`.

## Rustchat state before patch

- Target path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/websocket.rs`
- Prior behavior: mapped internal typing envelopes to outgoing events `user_typing` and `user_typing_stop` in `map_envelope_to_mm`, diverging from Mattermost's `typing`/`stop_typing`.

## Rustchat state after patch

- Target path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/websocket.rs:841`
- Observed behavior: outgoing event names changed to `typing` and `stop_typing` while preserving `data.user_id`, `data.parent_id` (`thread_root_id` mirrored), and `broadcast.channel_id`.

