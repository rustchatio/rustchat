- Screen, store, or service: Mattermost Mobile websocket new-post handler
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/actions/websocket/posts.ts`
- Source lines: 38-49
- Observed behavior: Mobile parses incoming websocket post as `post = JSON.parse(msg.data.post)`.
- Notes: Matches Mattermost server contract (`data.post` JSON string).

- Screen, store, or service: Mattermost Mobile thread/new-post routing
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/actions/websocket/posts.ts`
- Source lines: 68-72, 95-101, 170-172
- Observed behavior: Mobile decides CRT/thread behavior using `post.root_id`; root post presence is verified/fetched when needed.
- Notes: Immediate thread visibility depends on `root_id` being present and correctly interpreted.

- Screen, store, or service: Mattermost Mobile post creation model
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/actions/remote/post.ts`
- Source lines: 89-96, 190-192, 257-259
- Observed behavior: Mobile post models use Mattermost-style fields (`create_at`, `update_at`, `root_id`) and thread logic uses `created.root_id`.
- Notes: Confirms mobile-originating posts/replies are naturally Mattermost-style across websocket flows.

- Screen, store, or service: Rustchat web client websocket handler (current gap)
- Source path: `/Users/scolak/Projects/rustchat/frontend/src/composables/useWebSocket.ts`
- Source lines: 133-136
- Observed behavior: Handles `message_created`, `post_created`, `thread_reply_created` but not `posted`; also casts `envelope.data` directly to `Post` (no `data.post` parsing or field normalization).
- Notes: Mattermost-compatible `posted` events and MM post field names are not handled end-to-end.

- Screen, store, or service: Rustchat web message store (current gap)
- Source path: `/Users/scolak/Projects/rustchat/frontend/src/stores/messages.ts`
- Source lines: 39, 45
- Observed behavior: Store mapping expects `created_at` and `root_post_id`.
- Notes: Incoming websocket payloads with `create_at` / `root_id` fail thread classification until refresh via REST.
