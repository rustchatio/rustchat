# Architecture Gaps

## 1) API routing model

- Upstream: strongly typed gorilla/mux subrouter tree with explicit per-resource initialization and local-router mirrors (`api.go:177-495`).
- RustChat: Axum router composition via `.merge(...)` modules and fallback `501` envelope (`backend/src/api/v4/mod.rs:87-210`).

Assessment:
- Architectural pattern differs, but this is acceptable if wire contracts remain exact.
- Main risk is route drift when adding endpoints because subrouter constraints and method tables are not centrally generated from a contract.

## 2) Websocket model

- Upstream websocket endpoint setup with connection_id/sequence handling and platform hub wiring (`websocket.go:52-125`).
- RustChat websocket includes resume semantics, replay queueing, reconnect snapshots, and calls action dispatch (`backend/src/api/v4/websocket.rs:54-113`, `:712-760`).

Assessment:
- Semantically close for reconnect and calls actions.
- Residual risk remains around edge-case parity of action/error envelopes without exhaustive replay-contract tests.

## 3) Calls architecture

- Mobile expects plugin REST + websocket custom events (`calls/client/rest.ts`, `calls/connection/websocket_client.ts`).
- RustChat has first-class calls plugin routes and event handling (`backend/src/api/v4/calls_plugin/mod.rs:48-175`, websocket action hooks at `backend/src/api/v4/websocket.rs:551-611`).

Assessment:
- Good functional coverage of mobile calls flows.
- Need sustained compatibility tests to avoid regressions in event payload details.

## 4) Verification architecture gap

- Current local environment does not provide a reliable parity CI signal:
  - integration tests fail on DB auth,
  - smoke scripts default to port 3000 where a different stack is active.

Assessment:
- This is a production-readiness blocker independent of endpoint coverage.

## Architectural actions

1. Add contract-diff CI gate for method/path and schema deltas.
2. Add websocket golden-event fixtures for high-traffic mobile events.
3. Define and enforce compatibility profiles (core/mobile vs full/enterprise).
4. Standardize local verification stack bootstrap and port isolation.
