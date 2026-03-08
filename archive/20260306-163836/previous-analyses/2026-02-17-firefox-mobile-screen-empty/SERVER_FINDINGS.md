- Endpoint or component: SFU local receiver tracks
- Source path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/calls_plugin/sfu/mod.rs`
- Source lines: 265-281
- Observed behavior: `video_track` and `screen_track` are created with `mime_type: "video/vp8"`.
- Notes: Sender media that negotiates a different codec can produce decode/interoperability failures downstream.

- Endpoint or component: Calls config contract
- Source path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/calls_plugin/mod.rs`
- Source lines: 507-519
- Observed behavior: Config response advertises `enable_simulcast: false` and `enable_av1: false`.
- Notes: Client offers should align with this policy for maximum compatibility.

- Endpoint or component: Web offer creation path
- Source path: `/Users/scolak/Projects/rustchat/frontend/src/stores/calls.ts`
- Source lines: 507-589
- Observed behavior (before this fix): offer SDP had simulcast stripping but no explicit video codec preference enforcement.
- Notes: Firefox desktop share can negotiate codecs less compatible with mobile decode path.
