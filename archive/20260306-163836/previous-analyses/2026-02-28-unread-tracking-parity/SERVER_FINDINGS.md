# Server Findings

## Mattermost server (reference behavior)

- Endpoint or component: Post unread route and request body contract
  - Source path: `../mattermost/server/channels/api4/post.go`
  - Source lines: `1189-1215`, `1195-1197`
  - Observed behavior: `setPostUnread` requires post/user IDs, parses body as boolean map, reads `collapsed_threads_supported`, checks permissions, calls `MarkChannelAsUnreadFromPost`, returns encoded unread state.
  - Notes: This is the canonical unread-anchor endpoint for channel unread behavior.

- Endpoint or component: Channel unread from post app logic
  - Source path: `../mattermost/server/channels/app/channel.go`
  - Source lines: `2911-2940`, `3042-3048`
  - Observed behavior: Marks channel unread from a specific post boundary. Handles CRT / non-CRT paths and computes mention/root/urgent counters.
  - Notes: Behavior is explicitly post-based, not channel-global.

- Endpoint or component: `post_unread` websocket payload
  - Source path: `../mattermost/server/channels/app/channel.go`
  - Source lines: `3051-3060`
  - Observed behavior: Emits websocket event with keys: `msg_count`, `msg_count_root`, `mention_count`, `mention_count_root`, `urgent_mention_count`, `last_viewed_at`, `post_id`.
  - Notes: Clients depend on this event to update unread state.

- Endpoint or component: SQL unread write semantics
  - Source path: `../mattermost/server/channels/store/sqlstore/channel_store.go`
  - Source lines: `2667-2742`
  - Observed behavior: Updates `ChannelMembers` read counters and mention counters from target post position; returns `ChannelUnreadAt` with full fields.
  - Notes: Uses channel total counters to derive read position (`MsgCount = TotalMsgCount - unread`).

- Endpoint or component: Model contract for unread/channel member
  - Source path: `../mattermost/server/public/model/channel_member.go`
  - Source lines: `30-52`, `54-69`
  - Observed behavior: `ChannelUnread`/`ChannelUnreadAt` include root and urgent fields; `ChannelMember` includes `mention_count_root`, `urgent_mention_count`, `msg_count_root`.
  - Notes: Root and urgent counters are first-class contract fields.

- Endpoint or component: Websocket event registry
  - Source path: `../mattermost/server/public/model/websocket_message.go`
  - Source lines: `16-21`
  - Observed behavior: Defines `WebsocketEventPostUnread = "post_unread"`.
  - Notes: Event is part of normal websocket contract.

- Endpoint or component: View channel API
  - Source path: `../mattermost/server/channels/api4/channel.go`
  - Source lines: `1669-1714`
  - Observed behavior: Validates request body and IDs; returns `ChannelViewResponse{status,last_viewed_at_times}`.
  - Notes: Invalid body/IDs produce explicit client errors.

- Endpoint or component: View channel invalid body test
  - Source path: `../mattermost/server/channels/api4/channel_test.go`
  - Source lines: `3661-3664`
  - Observed behavior: Posting `"garbage"` to `/channels/members/{id}/view` must return HTTP 400.
  - Notes: Confirms strict parsing contract.

- Endpoint or component: Posts around unread validation
  - Source path: `../mattermost/server/channels/api4/post.go`
  - Source lines: `348-431`, specifically `372-375`
  - Observed behavior: Rejects `limit_after == 0` with invalid URL parameter error.
  - Notes: Limit clamping is not contract-compatible here.

- Endpoint or component: Team unread APIs
  - Source path: `../mattermost/server/channels/api4/team.go`
  - Source lines: `595-624`, `1104-1129`
  - Observed behavior: `getTeamsUnreadForUser` and `getTeamUnread` return computed unread aggregates.
  - Notes: Includes collapsed-thread option and permission checks.

- Endpoint or component: Team unread model
  - Source path: `../mattermost/server/public/model/team_member.go`
  - Source lines: `47-56`
  - Observed behavior: Team unread includes `mention_count_root`, `msg_count_root`, `thread_count`, `thread_mention_count`, `thread_urgent_mention_count`.
  - Notes: Contract exceeds simple `{msg_count,mention_count}`.

## Rustchat server (current behavior)

- Endpoint or component: Channel-level unread route (non-MM parity)
  - Source path: `backend/src/api/v4/channels.rs`
  - Source lines: `81-89`, `1892-1980`
  - Observed behavior: Implements `POST /channels/{channel_id}/members/{user_id}/set_unread`; sets unread by moving `last_viewed_at` to oldest post (or epoch), sets `channel_reads.last_read_message_id = 0`, emits `channel_unread` with fixed `unread_count: 1`.
  - Notes: Mattermost server has post-based unread endpoint instead.

- Endpoint or component: Channel read endpoint writes wrong `channel_reads` column names
  - Source path: `backend/src/api/v4/channels.rs`
  - Source lines: `1859-1871`
  - Observed behavior: SQL writes `channel_reads.last_viewed_at`, but migration defines `last_read_at` (`backend/migrations/20260124000002_unread_messages.sql:11-21`).
  - Notes: Call result is ignored (`let _ = ... .await`), masking runtime SQL errors.

- Endpoint or component: Channel unread endpoint also writes wrong `channel_reads` column
  - Source path: `backend/src/api/v4/channels.rs`
  - Source lines: `1948-1956`
  - Observed behavior: Updates `channel_reads.last_viewed_at` (non-existent per migration schema).
  - Notes: Result ignored; failures are silent.

- Endpoint or component: Post set unread incomplete contract
  - Source path: `backend/src/api/v4/posts.rs`
  - Source lines: `672-733`
  - Observed behavior: No request-body parse for `collapsed_threads_supported`; response struct omits MM fields (`user_id`, root counters, urgent counters); sets `msg_count` to `seq-1`; no websocket emission.
  - Notes: Route path matches MM, payload/side effects do not.

- Endpoint or component: Websocket mapping drops unread events
  - Source path: `backend/src/api/v4/websocket.rs`
  - Source lines: `404-406`, `1151-1393`
  - Observed behavior: Only mapped events are forwarded to MM websocket clients; unmapped events are dropped. No `post_unread` mapping case exists.
  - Notes: Even if backend emitted unread events, mapper currently suppresses them.

- Endpoint or component: Realtime event enum lacks `PostUnread`
  - Source path: `backend/src/realtime/events.rs`
  - Source lines: `34-104`
  - Observed behavior: Has `ChannelUnread` and `UnreadCountsUpdated`, but no `PostUnread`.
  - Notes: Prevents MM parity event emission path.

- Endpoint or component: View channel endpoint permissive behavior
  - Source path: `backend/src/api/v4/channels/view.rs`
  - Source lines: `25-33`, `84-90`
  - Observed behavior: Empty/invalid body returns `{"status":"OK"}`; parse errors are swallowed; response has no `last_viewed_at_times`.
  - Notes: Does not match MM validation and response contract.

- Endpoint or component: Team unread routes are placeholders
  - Source path: `backend/src/api/v4/users.rs`
  - Source lines: `910-923`, `925-938`
  - Observed behavior: `/users/me/teams/unread` and `/users/{id}/teams/unread` return `[]`; per-team unread returns static zeros.
  - Notes: No parity with MM team unread aggregates.

- Endpoint or component: Channel member counts zeroed in multiple responses
  - Source path: `backend/src/api/v4/channels.rs`
  - Source lines: `327-342`, `364-379`, `1035-1050`, `1150-1165`, `1207-1222`
  - Observed behavior: `msg_count` and `mention_count` are returned as `0` in channel member payloads.
  - Notes: Also replicated in mapper conversion path.

- Endpoint or component: Channel member mapper drops root/urgent fields
  - Source path: `backend/src/mattermost_compat/models.rs`, `backend/src/mattermost_compat/mappers.rs`
  - Source lines: models `109-121`; mapper `283-298`
  - Observed behavior: MM model and mapper only include `msg_count` and `mention_count`; root and urgent fields are absent from local compatibility structs.
  - Notes: Structural parity gap.

- Endpoint or component: Reconnect snapshot counter semantics mismatch
  - Source path: `backend/src/api/v4/websocket.rs`
  - Source lines: `811-839`, `852-856`
  - Observed behavior: Query computes unread count (`COUNT(posts after last_viewed_at)`) then maps that value into `channel_members.msg_count`.
  - Notes: In MM, `channel_members.msg_count` is read cursor count, not unread count.

- Endpoint or component: Redis unread service v1
  - Source path: `backend/src/services/unreads.rs`
  - Source lines: `1-326`
  - Observed behavior:
    - Keys: `rc:unread:{user}:{channel}`, `rc:unread_team:{user}:{team}`, `rc:channel:{channel}:last_msg_id`
    - Tracks only unread count (+ simple mentions elsewhere), no root/urgent/manual fields
    - `mark_all_as_read` uses Redis `KEYS rc:unread_team:{user}:*`
  - Notes: `KEYS` is non-scalable for large keyspaces and blocks Redis.

- Endpoint or component: API v1 posts read state source
  - Source path: `backend/src/api/posts.rs`
  - Source lines: `81-107`, `182-188`
  - Observed behavior: `/api/v1/channels/{channel}/posts` reads `channel_reads.last_read_message_id` and returns `read_state {last_read_message_id, first_unread_message_id}`.
  - Notes: Frontend message list divider relies on this `read_state`.

- Endpoint or component: API v1 channel read endpoint
  - Source path: `backend/src/api/channels.rs`
  - Source lines: `29`, `47-56`
  - Observed behavior: `/api/v1/channels/{id}/read` delegates to unread service `mark_channel_as_read` (optional `target_seq`).
  - Notes: This is separate from v4 read/view contract and currently drives the first-party frontend.
