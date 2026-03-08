- Endpoint or component: Rustchat SFU outgoing track construction
- Source path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/calls_plugin/sfu/mod.rs`
- Source lines: 257-273
- Observed behavior: `video_track` and `screen_track` are both created with the same stream id (`format!("stream-{}", session_id)`), differing only by track id.
- Notes: This makes screen media indistinguishable at stream identity level for consumers expecting a dedicated screen stream lifecycle.

- Endpoint or component: Rustchat screen-share toggle path
- Source path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/calls_plugin/mod.rs`
- Source lines: 1631-1665
- Observed behavior: Toggle updates state and emits screen on/off events; no call-state rebroadcast happens in this handler.
- Notes: Clients missing the event rely on API refresh timing.

- Endpoint or component: Rustchat ringing endpoint
- Source path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/calls_plugin/mod.rs`
- Source lines: 2205-2223
- Observed behavior: `/ring` currently emits only `custom_com.mattermost.calls_ringing`.
- Notes: No compatibility fallback event for clients that derive incoming notifications from `call_start` processing.

- Endpoint or component: Rustchat call-start endpoint
- Source path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/calls_plugin/mod.rs`
- Source lines: 1057-1074, 1092-1101
- Observed behavior: Start call emits `calls_call_start` and (for DM/GM) `calls_ringing`.
- Notes: Initial ring path exists, but explicit `/ring` retries do not reuse `call_start` semantics.
