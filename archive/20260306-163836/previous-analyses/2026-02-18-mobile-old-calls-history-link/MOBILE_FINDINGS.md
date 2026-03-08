- Screen, store, or service: Calls custom post wrapper.
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/components/calls_custom_message/index.ts`
- Source lines: `32-38`
- Observed behavior: If `post.props?.end_at` is set, component returns early and does not subscribe to active call/joining state streams.
- Notes: Ended posts are explicitly treated as non-active history cards.

- Screen, store, or service: Calls custom post renderer.
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/components/calls_custom_message/calls_custom_message.tsx`
- Source lines: `176-217`
- Observed behavior: Ended visual state (“Call ended”, ended time, duration) is shown only when `start_at > 0 && end_at > 0`.
- Notes: Ended status is data-driven from post props.

- Screen, store, or service: Calls custom post renderer.
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/components/calls_custom_message/calls_custom_message.tsx`
- Source lines: `219-251` and `284`
- Observed behavior: When not ended, renderer shows action button (`Leave` if already in same channel call, otherwise `Join`).
- Notes: Any stale old post with missing `end_at` remains join/leave-capable in UI.

- Screen, store, or service: Calls websocket handlers.
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/products/calls/connection/websocket_event_handlers.ts`
- Source lines: `101-108`
- Observed behavior: `calls_call_end` handler updates call state; it does not mutate call post props.
- Notes: Mobile relies on server post updates (`post_edited`) to transition old call posts to ended state.
