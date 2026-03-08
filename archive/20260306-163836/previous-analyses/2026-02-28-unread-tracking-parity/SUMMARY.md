# Summary

- Topic: Unread tracking parity for channels/posts (manual unread, unread counters, read markers, websocket parity)
- Date: 2026-02-28
- Scope:
  - Rustchat backend (`/api/v1` and `/api/v4`) unread/read endpoints, websocket events, reconnect snapshot, Redis unread service
  - Rustchat frontend unread behavior (channel switching, scroll auto-read, per-message mark unread, channel menu)
  - Mattermost server and webapp contracts
  - Mattermost mobile contract (from `../mattermost-mobile` commit `0a3e701b`)

## Compatibility contract (evidence-backed)

### 1) Manual unread is post-anchored, not channel-global
- Mattermost server route is `POST /users/{user_id}/posts/{post_id}/set_unread`, not a channel-level set-unread route:
  - `../mattermost/server/channels/api4/post.go:1189-1215`
  - Route registration: `../mattermost/server/channels/api4/post.go:43` (found by search)
- Request body includes `collapsed_threads_supported`:
  - `../mattermost/server/channels/api4/post.go:1195-1197`
- Mark unread operation updates unread state from a specific post boundary (`post.create_at - 1`) and returns `ChannelUnreadAt` shape:
  - `../mattermost/server/channels/app/channel.go:2911-2940, 3042-3048`
  - `../mattermost/server/channels/store/sqlstore/channel_store.go:2667-2742`
  - Model fields: `team_id`, `user_id`, `channel_id`, `msg_count`, `mention_count`, `msg_count_root`, `mention_count_root`, `urgent_mention_count`, `last_viewed_at`
    - `../mattermost/server/public/model/channel_member.go:41-52`

### 2) Websocket contract includes `post_unread`
- Mattermost defines `post_unread` websocket event:
  - `../mattermost/server/public/model/websocket_message.go:20`
- Event payload includes: `msg_count`, `msg_count_root`, `mention_count`, `mention_count_root`, `urgent_mention_count`, `last_viewed_at`, `post_id`:
  - `../mattermost/server/channels/app/channel.go:3051-3060`

### 3) Manual unread suppresses auto mark-as-read
- Mattermost webapp sets manual unread state before API call and preserves behavior on success/failure:
  - `../mattermost/webapp/channels/src/packages/mattermost-redux/src/actions/posts.ts:388-424`
  - `../mattermost/webapp/channels/src/packages/mattermost-redux/src/reducers/entities/channels.ts:811-833`
- Auto mark-read paths explicitly skip channels marked manually unread:
  - `../mattermost/webapp/channels/src/actions/views/channel.ts:498-507`
  - `../mattermost/webapp/channels/src/actions/new_post.ts:109-126`

### 4) Channel “mark unread” UI action is still post-based
- Sidebar channel menu “Mark as Unread” calls mark-most-recent-post-as-unread:
  - `../mattermost/webapp/channels/src/components/sidebar/sidebar_channel/sidebar_channel_menu/sidebar_channel_menu.tsx:75-77`
  - `../mattermost/webapp/channels/src/actions/post_actions.ts:425-439`

### 5) View channel endpoint strictness and response contract
- Mattermost `/channels/members/{user_id}/view` validates JSON body and IDs; invalid JSON returns 400:
  - `../mattermost/server/channels/api4/channel.go:1669-1714`
  - `../mattermost/server/channels/api4/channel_test.go:3661-3664`
- Response includes `{"status":"OK","last_viewed_at_times":...}`:
  - `../mattermost/server/channels/api4/channel.go:1706-1710`

### 6) Posts-around-unread validation
- Mattermost rejects `limit_after == 0` as invalid URL param:
  - `../mattermost/server/channels/api4/post.go:372-375`

### 7) Team unread endpoints return real aggregated counts (including CRT fields)
- Route behavior:
  - `../mattermost/server/channels/api4/team.go:595-624`
  - `../mattermost/server/channels/api4/team.go:1104-1129`
- Model includes `mention_count_root`, `msg_count_root`, `thread_count`, `thread_mention_count`, `thread_urgent_mention_count`:
  - `../mattermost/server/public/model/team_member.go:47-56`

### 8) New message separator behavior
- Mattermost webapp inserts separator when post crosses `lastViewedAt` boundary:
  - `../mattermost/webapp/channels/src/packages/mattermost-redux/src/utils/post_list.ts:95-104`
- Mobile does same:
  - `../mattermost-mobile/app/utils/post_list/index.ts:248-257`

## Rustchat current behavior (gap summary)

- Rustchat UI uses channel-level set unread (`/channels/{id}/members/{user}/set_unread`) and message menu calls that channel endpoint:
  - `frontend/src/components/channel/MessageItem.vue:95-99`
  - `frontend/src/api/channels.ts:85-86`
  - `backend/src/api/v4/channels.rs:1892-1980`
- Rustchat v4 post set unread exists but is incomplete versus MM contract:
  - no parsed `collapsed_threads_supported`, partial response fields, no `post_unread` websocket emission
  - `backend/src/api/v4/posts.rs:672-733`
- Rustchat websocket mapper has no `post_unread` mapping; unmapped events are dropped:
  - `backend/src/api/v4/websocket.rs:404-406`
  - `backend/src/api/v4/websocket.rs:1151-1393`
- Rustchat event enum has no `PostUnread` variant:
  - `backend/src/realtime/events.rs:34-104`
- Rustchat view endpoint returns `{"status":"OK"}` even for invalid body and does not return `last_viewed_at_times`:
  - `backend/src/api/v4/channels/view.rs:25-33, 84-90`
- Rustchat team unread routes are stubs or fixed zeros:
  - `backend/src/api/v4/users.rs:910-938`
- Rustchat channel member payloads zero out `msg_count`/`mention_count` broadly:
  - `backend/src/api/v4/channels.rs:327-342, 364-379, 1035-1050, 1150-1165, 1207-1222`
  - `backend/src/mattermost_compat/mappers.rs:283-298`
- Rustchat reconnect snapshot computes unread counts then stores them into `channel_members.msg_count` field (semantic mismatch):
  - `backend/src/api/v4/websocket.rs:811-839`

## Redis + architecture decision summary (target)

- Source of truth: Postgres (`channel_members`, `channel_reads`, thread tables).  
  Redis is a cache/fan-out accelerator, not authoritative state.
- Required cached state per user/channel must include full MM unread tuple:
  - `msg_count`, `msg_count_root`, `mention_count`, `mention_count_root`, `urgent_mention_count`, `last_viewed_at`, `manually_unread`, `version`.
- Redis keys (proposed):
  - `rc:unread:v2:uc:{user_id}:{channel_id}` (hash)
  - `rc:unread:v2:ut:{user_id}:{team_id}` (hash aggregate)
  - `rc:unread:v2:dirty:{user_id}` (set of channels pending recompute)
  - `rc:unread:v2:chan_seq:{channel_id}` (last seq cursor cache)
- Write pattern:
  - DB transaction first
  - after commit: atomic Redis update + websocket event publication
  - periodic reconciliation worker recomputes dirty keys from DB
- Rollout:
  - dual-write (legacy + v2 keys), compare counts, then cut over reads
  - keep fallback recompute path from DB for cache miss/error

## Open questions

- None in current scope.
