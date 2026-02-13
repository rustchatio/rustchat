# Server Findings

- Endpoint or component: Mobile calls plugin backend source in Mattermost monorepo checkout.
  - Source path: `../mattermost/server`
  - Source lines: N/A (repo-level search)
  - Observed behavior: Calls plugin server implementation is not present in this checkout; direct server-source parity must be inferred from mobile client contracts and Rustchat behavior.
  - Notes: Used mobile code as the primary compatibility oracle.

- Endpoint or component: Rustchat call state serialization.
  - Source path: `backend/src/api/v4/calls_plugin/mod.rs`
  - Source lines: `651-749`
  - Observed behavior: call state payload includes sessions, participants, and now resolves missing `thread_id` via `ensure_call_thread_id` before serialization.
  - Notes: Prevents missing-thread key regressions in `calls_call_state` and REST state endpoints.

- Endpoint or component: Rustchat call-thread post creation.
  - Source path: `backend/src/api/v4/calls_plugin/mod.rs`
  - Source lines: `759-820`
  - Observed behavior: new helper inserts a `custom_calls` root post, broadcasts `message_created`, and increments unreads.
  - Notes: Establishes a concrete thread root for call thread navigation.

- Endpoint or component: Rustchat call-thread state persistence.
  - Source path: `backend/src/api/v4/calls_plugin/state.rs`
  - Source lines: `274-280`
  - Observed behavior: new `set_thread_id` mutator persists thread id to call state backend.
  - Notes: keeps thread id stable across subsequent state payloads.

- Endpoint or component: Rustchat REST call start flow.
  - Source path: `backend/src/api/v4/calls_plugin/mod.rs`
  - Source lines: `917-1082`
  - Observed behavior: start flow now ensures thread id before broadcasting `calls_call_start`, and includes `thread_id` in payload.
  - Notes: mobile receivers get a defined thread id at call start.

- Endpoint or component: Rustchat websocket join/reconnect flow.
  - Source path: `backend/src/api/v4/calls_plugin/mod.rs`
  - Source lines: `2635-2785`
  - Observed behavior: join flow now ensures thread id and emits `calls_call_state` after `calls_user_joined`.
  - Notes: improves state convergence for mobile clients on join.

- Endpoint or component: Rustchat SFU signaling websocket payload shape.
  - Source path: `backend/src/api/v4/calls_plugin/mod.rs`
  - Source lines: `3678-3733`
  - Observed behavior: signaling event now includes `connID`, `conn_id`, serialized `data`, and structured `signal`.
  - Notes: aligns with mobile websocket client filtering and peer signaling parser expectations.
