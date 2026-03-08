- Endpoint or component: Rustchat Calls config response
- Source path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/calls_plugin/mod.rs`
- Source lines: 507-519
- Observed behavior: Calls config is returned with `enable_simulcast: false`.
- Notes: Client offers should not force simulcast attributes when this policy is disabled.

- Endpoint or component: Rustchat SFU forwarding behavior
- Source path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/calls_plugin/sfu/mod.rs`
- Source lines: 733-760
- Observed behavior: SFU forwards incoming video RTP to receiver video and screen tracks as a fallback strategy.
- Notes: This does not normalize browser offer SDP; malformed/unsupported simulcast signaling from sender can still break track mapping.

- Endpoint or component: Runtime symptom from reported logs
- Source path: User-provided runtime logs
- Source lines: N/A
- Observed behavior: Repeated warnings/errors including `Incoming unhandled RTP ... mid RTP Extensions required for Simulcast` during screen-share renegotiation.
- Notes: Symptom is consistent with unsupported/undesired simulcast signaling on sender offers.
