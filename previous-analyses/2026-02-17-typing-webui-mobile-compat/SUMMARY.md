# Typing WebUI-Mobile Compatibility Summary

- Topic: WebUI <-> Mattermost Mobile typing indicators over websocket
- Date: 2026-02-17
- Scope: Websocket command/event compatibility for typing start/stop and channel routing

## Compatibility contract

1. Outbound client command must be `action: "user_typing"` with `data.channel_id` and optional `data.parent_id`.
2. Server-to-client typing event must be `event: "typing"` (not `user_typing`).
3. Server-to-client stop-typing event must be `event: "stop_typing"` (not `user_typing_stop`).
4. Event payload must include `data.user_id` and optional `data.parent_id`.
5. Channel targeting must be in `broadcast.channel_id`; clients should resolve channel from broadcast when top-level `channel_id` is absent.

## Evidence

- Mattermost webapp sends `user_typing`: `/Users/scolak/Projects/mattermost/webapp/platform/client/src/websocket.ts:618`
- Mattermost webapp consumes `typing` and reads `msg.broadcast.channel_id`: `/Users/scolak/Projects/mattermost/webapp/channels/src/components/msg_typing/msg_typing.tsx:23`
- Mattermost mobile sends `user_typing`: `/Users/scolak/Projects/mattermost-mobile/app/client/websocket/index.ts:419`
- Mattermost mobile dispatches typing on `WebsocketEvents.TYPING`: `/Users/scolak/Projects/mattermost-mobile/app/actions/websocket/event.ts:147`
- Mattermost mobile reads `msg.broadcast.channel_id`, `msg.data.parent_id`, `msg.data.user_id`: `/Users/scolak/Projects/mattermost-mobile/app/actions/websocket/users.ts:106`

## Open questions

- None for typing contract. Additional websocket events should be audited separately.
