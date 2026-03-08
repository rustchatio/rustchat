- Rustchat target path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/calls_plugin/sfu/mod.rs`
- Required behavior: Screen share should surface as a distinct remote stream identity compatible with mobile screen stream handling.
- Current gap: Outgoing screen track reuses the same stream id as regular video.
- Planned change: Use a dedicated screen stream id (contains `screen`) for `screen_track`.
- Verification test: Unit test asserting screen track stream id differs from video stream id and contains `screen` marker.
- Status: Planned

- Rustchat target path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/calls_plugin/mod.rs`
- Required behavior: Ring retries should notify clients that depend on `call_start` incoming-call processing.
- Current gap: `/ring` only emits `calls_ringing`.
- Planned change: Emit a compatibility `calls_call_start` payload in `/ring` (in addition to `calls_ringing`) using existing call state.
- Verification test: Unit/integration test for ring endpoint event fan-out includes `calls_ringing` and `calls_call_start`.
- Status: Planned

- Rustchat target path: `/Users/scolak/Projects/rustchat/previous-analyses/2026-02-14-mobile-screen-share-ringing-regressions/GAP_PLAN.md`
- Required behavior: Track completed compatibility checks, risks, and evidence.
- Current gap: Not updated yet.
- Planned change: Update with execution results after code/test changes.
- Verification test: N/A.
- Status: Planned
