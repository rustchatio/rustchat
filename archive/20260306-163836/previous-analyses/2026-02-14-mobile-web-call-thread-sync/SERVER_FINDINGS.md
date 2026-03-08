- Endpoint or component: Mattermost server websocket post publishing
- Source path: `/Users/scolak/Projects/mattermost/server/public/model/websocket_message.go`
- Source lines: 17
- Observed behavior: Post creation events are named `posted`.
- Notes: This is the canonical event name expected by Mattermost web/mobile clients.

- Endpoint or component: Mattermost server websocket payload encoding
- Source path: `/Users/scolak/Projects/mattermost/server/channels/app/post.go`
- Source lines: 965-986
- Observed behavior: `publishWebsocketEventForPost` serializes the post with `post.ToJSON()` and inserts it as `message.Add("post", postJSON)` (string field inside `data`).
- Notes: Clients must parse `msg.data.post` JSON string.

- Endpoint or component: Mattermost webapp websocket event dispatch
- Source path: `/Users/scolak/Projects/mattermost/webapp/channels/src/actions/websocket_actions.ts`
- Source lines: 382-385, 785-788
- Observed behavior: `WebSocketEvents.Posted` is routed to new-post handler; handler parses `JSON.parse(msg.data.post)`.
- Notes: Confirms client contract: event is `posted`, payload is in `data.post` string.

- Endpoint or component: Mattermost webapp thread semantics
- Source path: `/Users/scolak/Projects/mattermost/webapp/channels/src/actions/new_post.ts`
- Source lines: 41-46, 65-66
- Observed behavior: Thread handling depends on `post.root_id`; replies trigger thread-aware flow immediately.
- Notes: If `root_id` is missing/unmapped, reply is treated as non-thread/root post.

- Endpoint or component: Rustchat websocket emit path (current)
- Source path: `/Users/scolak/Projects/rustchat/backend/src/services/posts.rs`
- Source lines: 139-147
- Observed behavior: Rustchat emits `EventType::MessageCreated` (`posted`) for root posts and `EventType::ThreadReplyCreated` for replies; payload is Mattermost-compat `mm::Post` (`root_id`, `create_at`).
- Notes: Payload shape differs from frontend store expectation (`root_post_id`, `created_at`).

- Endpoint or component: Rustchat call-thread root post emit path (current)
- Source path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/calls_plugin/mod.rs`
- Source lines: 810-829
- Observed behavior: Call thread root post is emitted as `EventType::MessageCreated` with `mm::Post` payload.
- Notes: Web client must support Mattermost post field names for these events too.

- Endpoint or component: Rustchat v4 websocket adapter gap (secondary)
- Source path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/websocket.rs`
- Source lines: 811-816
- Observed behavior: `posted` mapping only tries deserializing `env.data` as `PostResponse`, but runtime emits may already be `mm::Post`.
- Notes: Can drop/reject valid posted events on v4 path without fallback.
