# Mobile Findings

## Mattermost mobile contract

Repository used: `../mattermost-mobile` at commit `0a3e701b`.

- Screen, store, or service: REST client `markPostAsUnread`
  - Source path: `../mattermost-mobile/app/client/rest/posts.ts`
  - Source lines: `149-156`
  - Observed behavior: Calls `POST /users/{user}/posts/{post}/set_unread` with body `{"collapsed_threads_supported": true}`.
  - Notes: Mobile always sends CRT-support flag.

- Screen, store, or service: REST client `viewMyChannel`
  - Source path: `../mattermost-mobile/app/client/rest/channels.ts`
  - Source lines: `311-317`
  - Observed behavior: Calls `POST /channels/members/me/view` with `channel_id`, `prev_channel_id`, and `collapsed_threads_supported: true`.
  - Notes: Relies on view endpoint body parsing and response consistency.

- Screen, store, or service: Post options “Mark as Unread”
  - Source path: `../mattermost-mobile/app/screens/post_options/options/mark_unread_option/mark_unread_option.tsx`
  - Source lines: `35-42`
  - Observed behavior: In channel context calls `markPostAsUnread`; in CRT thread context calls thread unread endpoint.
  - Notes: User intent is anchored to a selected post, not a channel-global toggle.

- Screen, store, or service: Thread options “Mark as Unread”
  - Source path: `../mattermost-mobile/app/screens/thread_options/options/mark_as_unread_option.tsx`
  - Source lines: `35-41`
  - Observed behavior: Toggles thread read/unread using thread endpoints.
  - Notes: Thread unread is separate from channel unread.

- Screen, store, or service: Websocket event registration
  - Source path: `../mattermost-mobile/app/constants/websocket.ts`
  - Source lines: `7-13`
  - Observed behavior: `POST_UNREAD` is a first-class websocket event constant (`'post_unread'`).
  - Notes: Client expects this event name directly.

- Screen, store, or service: Websocket dispatcher
  - Source path: `../mattermost-mobile/app/actions/websocket/event.ts`
  - Source lines: `45-47`
  - Observed behavior: Routes `POST_UNREAD` to `posts.handlePostUnread`.
  - Notes: Event dispatch depends on exact event name and payload fields.

- Screen, store, or service: Websocket unread handler
  - Source path: `../mattermost-mobile/app/actions/websocket/posts.ts`
  - Source lines: `335-369`
  - Observed behavior: Reads `msg_count`, `msg_count_root`, `mention_count`, `mention_count_root`, `last_viewed_at`, team/channel IDs; applies CRT-aware counters; skips auto update when `myChannel.manuallyUnread` is true.
  - Notes: Manual unread flag is a behavioral gate.

- Screen, store, or service: Local model for manual unread
  - Source path: `../mattermost-mobile/app/database/models/server/my_channel.ts`
  - Source lines: `38-40`
  - Observed behavior: Persists `manually_unread` on my-channel model.
  - Notes: Auto-read suppression is stateful across events/views.

- Screen, store, or service: New messages separator
  - Source path: `../mattermost-mobile/app/utils/post_list/index.ts`
  - Source lines: `248-257`
  - Observed behavior: Adds start-of-new-messages marker when post timestamp crosses `lastViewedAt`.
  - Notes: Separator logic is timestamp-based and per-channel view state.

## Rustchat frontend behavior affecting parity

- Screen, store, or service: Channel switch/read behavior
  - Source path: `frontend/src/views/main/ChannelView.vue`
  - Source lines: `56-61`
  - Observed behavior: Immediately calls `markAsRead` on channel change.
  - Notes: No manual-unread gate before auto read.

- Screen, store, or service: Scroll-to-bottom auto-read behavior
  - Source path: `frontend/src/components/channel/MessageList.vue`
  - Source lines: `54-57`
  - Observed behavior: When at bottom and unread > 0, marks channel read.
  - Notes: Also has no manual-unread gate.

- Screen, store, or service: Per-message “Mark as unread”
  - Source path: `frontend/src/components/channel/MessageItem.vue`
  - Source lines: `95-99`
  - Observed behavior: Calls channel-level `unreadStore.markAsUnread(channelId)`.
  - Notes: Does not anchor to specific post.

- Screen, store, or service: Channel context menu “Mark as unread”
  - Source path: `frontend/src/components/channels/ChannelContextMenu.vue`
  - Source lines: `155-163`, `243-249`
  - Observed behavior: Toggles channel read/unread using channel-level endpoints.
  - Notes: Differs from Mattermost’s mark-most-recent-post approach.
