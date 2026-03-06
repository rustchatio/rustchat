# Summary

- Topic: RustChat vs Mattermost parity and production-readiness gap analysis
- Date: 2026-03-06
- Scope: Server architecture, API v4 endpoint coverage, websocket/calls behavior, and Mattermost mobile core user journeys
- Upstream baseline:
  - `../mattermost` commit: `56f51d7df254ef0dea727976ca3b4b2175ba2fab`
  - `../mattermost-mobile` commit: `0a3e701ba5d536174a7e5b1be0adcfc1b8959165`
  - `rustchat` commit: `47cf28137992fb543285e8009294503760931dfc`

## Compatibility contract (current state)

- API v4 method+path coverage against upstream OpenAPI baseline: `438 / 570` (`76.8%`), `132` missing, `117` RustChat-only extras.
- Core mobile journey route coverage (selected 74 high-traffic routes): direct route gap closed for `PUT /api/v4/posts/{post_id}` in this iteration; sampled set now covered at route level.
- Calls plugin surface required by mobile app is implemented under `/api/v4/plugins/com.mattermost.calls/*` and corresponding websocket custom events are handled.

## Headline findings

1. Core chat + team/channel + calls mobile paths are mostly present, but full Mattermost parity is not yet met.
2. A high volume of uncovered endpoints sits in plugin/enterprise/admin surfaces (`plugins`, `groups`, `access_control_policies`, `custom_profile_attributes`, `license`, `reports`).
3. Post route contract mismatches identified for mobile/core flows in this iteration are closed (`PUT /posts/{post_id}`, `GET /posts/{post_id}/reveal`, `DELETE /posts/{post_id}/burn`), with temporary POST shims retained for burn-on-read routes.
4. Verification environment now has deterministic integration bootstrap (`docker-compose.integration.yml` + `RUSTCHAT_TEST_*`), and smoke scripts now require explicit `BASE` with compatibility preflight.

## Production decision

- Status: **Not ready for production parity claim**.
- Reason: parity backlog (remaining P2 contract/coverage gaps) and incomplete full-stack smoke evidence against a dedicated live RustChat target in this iteration.

## Open questions

- Which parity target profile is release-blocking: "mobile core only" or "full upstream v4 including enterprise/plugin surfaces"?
- Should RustChat explicitly defer enterprise-only routes in a published compatibility profile, or implement stubs with exact upstream semantics?
