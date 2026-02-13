# Mobile Findings

- Screen, store, or service: Calls websocket signal gate.
  - Source path: `../mattermost-mobile/app/products/calls/connection/websocket_client.ts`
  - Source lines: `108-110`
  - Observed behavior: signal/join/error messages are ignored unless `msg.data.connID` matches current/original connection id.
  - Notes: server-emitted calls signals without `connID` are dropped.

- Screen, store, or service: Calls peer signaling parser.
  - Source path: `../mattermost-mobile/app/products/calls/connection/connection.ts`
  - Source lines: `450-457`
  - Observed behavior: handler expects `data` string and runs `JSON.parse(data)` before `peer.signal(...)`.
  - Notes: server payload must provide serialized `data` field; structured-only payload is insufficient.

- Screen, store, or service: Call thread navigation action.
  - Source path: `../mattermost-mobile/app/products/calls/screens/call_screen/call_screen.tsx`
  - Source lines: `485-497`
  - Observed behavior: thread screen navigation passes `rootId: currentCall.threadId` directly without fallback.
  - Notes: undefined thread id propagates to thread query path.

- Screen, store, or service: Call state mapping from server payload.
  - Source path: `../mattermost-mobile/app/products/calls/actions/calls.ts`
  - Source lines: `143-170`
  - Observed behavior: `threadId` is set from `call.thread_id`; absent key results in undefined `threadId`.
  - Notes: backend must include a valid `thread_id` in call state payload.

- Screen, store, or service: Follow-thread behavior after call start/join.
  - Source path: `../mattermost-mobile/app/products/calls/state/actions.ts`
  - Source lines: `500-511`, `563-579`
  - Observed behavior: call flow queries/follows thread using `call.threadId` and separately tracks speaking state from voice events.
  - Notes: missing thread id and dropped signaling both directly impact reported regressions.

- Screen, store, or service: Thread query implementation.
  - Source path: `../mattermost-mobile/app/queries/servers/thread.ts`
  - Source lines: `65-68`
  - Observed behavior: thread observers construct WatermelonDB `Q.where('id', threadId)` queries.
  - Notes: undefined `threadId` leads to query comparator errors consistent with reported crash text.
