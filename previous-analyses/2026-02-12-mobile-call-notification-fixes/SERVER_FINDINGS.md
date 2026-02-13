# Server Findings

- Endpoint or component: Calls plugin implementation availability in upstream repo
- Source path: `../mattermost/server/Makefile`
- Source lines: `153-161`
- Observed behavior: Calls ships as prepackaged `mattermost-plugin-calls` artifact.
- Notes: Endpoint internals are plugin-owned and not directly visible in `../mattermost/server`.

- Endpoint or component: RustChat start-call behavior
- Source path: `backend/src/api/v4/calls_plugin/mod.rs`
- Source lines: `777-949`
- Observed behavior: Emits `calls_call_start` and `calls_user_joined`; does not auto-emit `calls_ringing` for DM/GM.
- Notes: P0 gap for automatic ringing notifications.

- Endpoint or component: RustChat dismissed notifications in call-state response
- Source path: `backend/src/api/v4/calls_plugin/mod.rs`
- Source lines: `621-713`
- Observed behavior: `dismissed_notification` is always `Some(HashMap::new())`.
- Notes: P0 gap; dismissal is not persisted per user.

- Endpoint or component: RustChat dismiss-notification endpoint
- Source path: `backend/src/api/v4/calls_plugin/mod.rs`
- Source lines: `2032-2064`
- Observed behavior: Broadcasts dismissed event but does not mutate persisted call state.
- Notes: Requires call-state mutation + persistence.

- Endpoint or component: RustChat calls config payload
- Source path: `backend/src/api/v4/calls_plugin/mod.rs`
- Source lines: `203-221`, `445-476`
- Observed behavior: Returns a subset of fields; missing several mobile-checked fields.
- Notes: P1 gap for feature gating parity.

- Endpoint or component: RustChat call state backend
- Source path: `backend/src/api/v4/calls_plugin/state.rs`
- Source lines: `16-26`, `334-360`, `404-442`
- Observed behavior: `CallState` persists through memory/redis serialization, so new fields can be persisted by extending struct.
- Notes: Safe place to store dismissed users.
