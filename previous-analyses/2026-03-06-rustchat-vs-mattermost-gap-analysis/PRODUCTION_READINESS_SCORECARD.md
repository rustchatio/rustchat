# Production Readiness Scorecard

Date: 2026-03-06

## Gates

| Gate | Result | Notes |
| :--- | :--- | :--- |
| Build health (`cargo check`, frontend build) | PASS | both commands succeed |
| Backend automated tests | PARTIAL | targeted parity suites (`api_v4_post_routes`, `api_v4_channels_all`, `api_v4_plugins_dialogs`) pass with deterministic integration profile; full backend suite not rerun in this iteration |
| Compatibility smoke scripts | PARTIAL | scripts now enforce explicit `BASE` + `X-MM-COMPAT` preflight; full smoke against a live RustChat app target still pending in this iteration |
| Core mobile route compatibility (sampled) | PASS/PARTIAL | direct sampled gap closed (`PUT /posts/{post_id}`); broader parity backlog remains |
| Full upstream v4 parity | FAIL | 438/570 method+path matches |
| Calls mobile surface | PASS/PARTIAL | route/event surface present; environment prevents full e2e confirmation |

## Release recommendation

- Decision: **NO-GO** for claiming "almost all Mattermost features" parity.
- Conditional GO (limited profile): possible for controlled internal/mobile-core pilots with current P1 closures and documented P2 limitations.

## Blocking items

1. `G-005` large endpoint coverage delta on plugin/admin/enterprise surfaces.
2. Full compatibility smoke evidence against dedicated running RustChat app target not captured in this iteration.

## Suggested acceptance thresholds

- P0 = 0, P1 = 0 before production launch.
- P2 allowed only with explicit documented compatibility profile and user-visible limitations.
- Mandatory green checks:
  - backend integration suite in CI profile,
  - mobile and compat smoke scripts against dedicated RustChat stack,
  - endpoint diff gate with explicit allowlist.
