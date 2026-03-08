- Endpoint or component: Mattermost origin client detection
- Source path: `/Users/scolak/Projects/mattermost/server/channels/web/handlers.go`
- Source lines: 456-470
- Observed behavior: `mobilev2=true` query explicitly classifies request origin as mobile.
- Notes: Rustchat must preserve compatibility for calls routes consumed by mobile/desktop clients that depend on this query contract.

- Endpoint or component: Rustchat SFU offer handling
- Source path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/calls_plugin/sfu/mod.rs`
- Source lines: 390-439
- Observed behavior: SFU applies remote description, creates answer, then waits 2 seconds before returning final local description.
- Notes: Repeated renegotiation cycles amplify timing/race sensitivity when client repeatedly adds/removes senders.

- Endpoint or component: Rustchat SFU video forwarding
- Source path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/calls_plugin/sfu/mod.rs`
- Source lines: 733-767
- Observed behavior: Video RTP is duplicated to both receiver `video_track` and `screen_track` as a race-safe fallback.
- Notes: This is resilient for screen-track routing, but it does not prevent client-side transceiver/SSRC churn from repeated add/remove sender flows.
