# UX Journeys

## Journey scorecard

| Journey | Status | Evidence | Key gaps |
| :--- | :--- | :--- | :--- |
| Login/session | Mostly compatible | `users.ts:122-220`, RustChat `users.rs` login routes | environment verification instability |
| Team/channel discovery & switching | Mostly compatible | `teams.ts:64-125`, `channels.ts:196-272`, RustChat channels/users routes | missing `GET /channels` admin path |
| Posting/editing/reacting/threading | Partial | `posts.ts:100-246`, RustChat `posts.rs` routes | missing `PUT /posts/{id}`, reveal/burn verb drift |
| Notifications/read state | Mostly compatible | `posts.ts:220-238`, `websocket/index.ts:150-157`, RustChat read/unread routes | needs full reconnect contract tests |
| File upload/preview/download | Mostly compatible | `files.ts:55-103`, RustChat files/posts routes | no critical route gap found in sampled set |
| Calls entry/join/leave/host controls | Mostly compatible | `calls/client/rest.ts:41-170`, `calls/websocket_*` handlers, RustChat calls plugin routes/events | regressions possible without stable integration env |

## Ease-of-use observations

- Upstream mobile relies on aggressive local batching and reconnect rehydration; backend contract consistency matters more than raw endpoint count.
- RustChat already exposes most high-traffic paths needed by mobile flows.
- User-perceived break risk is currently concentrated in edge/feature-flagged post routes and in operational instability (if deployment wiring mirrors current local issues).

## Immediate UX hardening priorities

1. Close post verb/method parity gaps.
2. Add reconnect-flow contract tests for unread/thread/calls states.
3. Make smoke scripts environment-aware (`BASE` must point to RustChat backend).
