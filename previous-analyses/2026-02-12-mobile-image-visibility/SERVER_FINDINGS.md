# Server Findings

- Endpoint or component: Upstream channel post list response assembly
- Source path: `/Users/scolak/Projects/mattermost/server/channels/api4/post.go`
- Source lines: `321-333`, `402-408`
- Observed behavior: Mattermost runs `PreparePostListForClient` and metadata sanitization before encoding post lists.
- Notes: Metadata preparation path is centralized, not ad-hoc per handler.

- Endpoint or component: Upstream pinned posts response assembly
- Source path: `/Users/scolak/Projects/mattermost/server/channels/api4/channel.go`
- Source lines: `836-844`
- Observed behavior: Pinned post lists also pass through `PreparePostListForClient` + metadata sanitization.
- Notes: Confirms pinned route should preserve file metadata too.

- Endpoint or component: RustChat channel posts list (`GET /channels/{channel_id}/posts`)
- Source path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/channels.rs`
- Source lines: `1506-1550`
- Observed behavior: Handler now populates files via `populate_files` and merges reactions into existing metadata object.
- Notes: Fix prevents reaction metadata from overwriting file metadata.

- Endpoint or component: RustChat pinned posts list (`GET /channels/{channel_id}/pinned`)
- Source path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/channels.rs`
- Source lines: `843-888`
- Observed behavior: Handler now populates files before mapping posts and merges reactions into existing metadata.
- Notes: Keeps pinned-post attachment behavior aligned with standard post list.

- Endpoint or component: RustChat unread-around-last-read list
- Source path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/posts/unread.rs`
- Source lines: `79-145`
- Observed behavior: Handler now populates files and merges reactions without replacing metadata.
- Notes: Addresses unread-sync path used by mobile.
