# Gap Plan

## Implemented

1. End-call deadlock recovery for stale host websocket session
- Change: allow participant end-call path when host session is inactive and caller is still in call.
- Code: `backend/src/api/v4/calls_plugin/mod.rs:342-351`, `backend/src/api/v4/calls_plugin/mod.rs:1373-1379`

2. Calls reaction payload parity for mobile reducers
- Change: include `session_id`, `reaction`, `timestamp`, and `emoji` object in reaction events.
- Code: `backend/src/api/v4/calls_plugin/mod.rs:1558-1599`, `backend/src/api/v4/calls_plugin/mod.rs:3231-3275`

3. Regression tests added
- Host transfer after host leaves: `backend/tests/api_calls_signaling.rs:736-807`
- System admin can end non-host call: `backend/tests/api_calls_signaling.rs:809-863`
- Reaction websocket payload fields: `backend/tests/api_calls_signaling.rs:865-930`

## Verification

- Build/compile gate passed:
  - `cargo test --test api_calls_signaling --no-run` (passes)

- Full integration test run is blocked in this environment by missing Postgres test service:
  - `cargo test --test api_calls_signaling` fails with `Connection refused` to Postgres in `tests/common/mod.rs` setup.

## Remaining Risks

1. Multi-device host presence edge cases should be validated with manual E2E (same host user on multiple sessions).
2. Media viewer issue reported separately still needs dedicated endpoint-level investigation (`/api/v4/files/*` flow) after calls regressions are validated in a rebuilt backend.
