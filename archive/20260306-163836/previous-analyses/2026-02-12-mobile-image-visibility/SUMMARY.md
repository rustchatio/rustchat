# Summary

- Topic: Mobile image attachments disappear from message history after re-login
- Date: 2026-02-12
- Scope: Mattermost mobile compatibility for post-list payloads (`/channels/{id}/posts`, pinned, unread)

## Compatibility contract

- Channel post list APIs used by mobile (`GET /api/v4/channels/{channel_id}/posts`, `GET .../pinned`, `GET /api/v4/users/{user_id}/channels/{channel_id}/posts/unread`) must return post metadata containing file infos under `metadata.files` whenever `post.file_ids` is non-empty.
  - Evidence: `/Users/scolak/Projects/mattermost-mobile/app/client/rest/posts.ts:92-137`
  - Evidence: `/Users/scolak/Projects/mattermost-mobile/app/database/operator/server_data_operator/handlers/post.ts:327-330`

- Mobile processes files from `metadata.files` and separately tracks `file_ids`; missing `metadata.files` causes file models not to be created/updated during sync, so attachments disappear on re-login/history reload.
  - Evidence: `/Users/scolak/Projects/mattermost-mobile/app/database/operator/server_data_operator/handlers/post.ts:327-330`
  - Evidence: `/Users/scolak/Projects/mattermost-mobile/app/database/operator/server_data_operator/handlers/post.ts:396-406`

- Reactions and files must coexist in metadata; adding reactions cannot replace/erase existing file metadata.
  - Evidence: `/Users/scolak/Projects/mattermost-mobile/app/database/operator/server_data_operator/handlers/post.ts:315-330`

- Upstream Mattermost prepares and sanitizes post lists through metadata-aware pipeline before response encoding.
  - Evidence: `/Users/scolak/Projects/mattermost/server/channels/api4/post.go:321-333`
  - Evidence: `/Users/scolak/Projects/mattermost/server/channels/api4/post.go:402-408`
  - Evidence: `/Users/scolak/Projects/mattermost/server/channels/api4/channel.go:836-844`

## Open questions

- `GET /api/v4/posts/{post_id}/files/info` in RustChat still sources `post_id` from the `files` table, and uploaded files may have empty DB `post_id` until explicitly updated. Current mobile history flow is fixed via `metadata.files`, but this endpoint’s linking semantics may need follow-up hardening.
