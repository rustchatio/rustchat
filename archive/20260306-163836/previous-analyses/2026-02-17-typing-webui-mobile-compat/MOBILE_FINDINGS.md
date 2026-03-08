# Mobile Findings

## Mattermost Mobile websocket typing contract

- Screen, store, or service: websocket client typing sender
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/client/websocket/index.ts`
- Source lines: `419-423`
- Observed behavior: Mobile sends `action: 'user_typing'` with `channel_id` and `parent_id`.

- Screen, store, or service: websocket event dispatcher
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/actions/websocket/event.ts`
- Source lines: `147-148`
- Observed behavior: Typing is handled when event is `WebsocketEvents.TYPING` (`typing`).

- Screen, store, or service: typing event handler
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/actions/websocket/users.ts`
- Source lines: `105-108`
- Observed behavior: Handler uses `msg.broadcast.channel_id`, `msg.data.parent_id`, and `msg.data.user_id`.

## Compatibility implication

- If server emits `user_typing` instead of `typing`, mobile typing indicator handler is not triggered.
- If client code ignores `broadcast.channel_id`, typing cannot be associated with the active channel even when event is delivered.

