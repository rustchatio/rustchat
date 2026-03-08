- Endpoint or component: Post type contract for calls custom posts.
- Source path: `/Users/scolak/Projects/mattermost/webapp/channels/src/utils/constants.tsx`
- Source lines: `725-726`
- Observed behavior: Mattermost defines calls custom post types as `custom_calls` and `custom_calls_recording`; clients depend on type identity for rendering.
- Notes: This establishes canonical type naming expected by Mattermost clients.

- Endpoint or component: Rustchat call-post creation.
- Source path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/calls_plugin/mod.rs`
- Source lines: `780-793`, `821`
- Observed behavior: Rustchat creates a call thread post with props `{type: "custom_calls", start_at, end_at: 0}` and sends initial post with `post_type: "custom_calls"`.
- Notes: Initial create matches mobile expectations.

- Endpoint or component: Rustchat call end flow.
- Source path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/calls_plugin/mod.rs`
- Source lines: `3980-4004`
- Observed behavior: Call end removes call state and emits `custom_com.mattermost.calls_call_end`, but does not update call post props (`end_at`).
- Notes: This leaves historical call posts indistinguishable from active call posts for mobile renderer.

- Endpoint or component: MM post mapper in Rustchat.
- Source path: `/Users/scolak/Projects/rustchat/backend/src/mattermost_compat/mappers.rs`
- Source lines: `144`, `217`
- Observed behavior: `mm::Post.post_type` is emitted as empty string for mapped `Post`/`PostResponse` updates.
- Notes: Post update events for custom posts risk losing type identity unless mapper preserves `props.type`.
