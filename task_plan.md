# Task Plan

## 2026-03-28 Reaction Deduplication + Shared Presence/Status Layer

### Task
- Keep the reaction deduplication fix.
- Implement the approved Option B architecture from `SPEC.md`.
- Stop the UI from computing presence/status separately in each touched component.

### Architecture Locked In
- Add one shared frontend presence/status presenter for badge label, icon, and visual treatment.
- Add one shared user-summary source for direct-message and profile/detail surfaces.
- Move session cleanup and active presence usage fully onto `frontend/src/features/presence/stores/presenceStore.ts`.
- Treat `frontend/src/stores/presence.ts` as legacy and remove it from the active flow.

### Implementation Checklist
- [x] Keep reaction canonicalization centered in `frontend/src/utils/emoji.ts`, `frontend/src/stores/messages.ts`, and `frontend/src/components/channel/MessageItem.vue`.
- [x] Add the shared presence/status presenter and switch touched UI surfaces to use it.
- [x] Add one DM/user-summary helper so `ChannelInfoPanel` and `UserProfileModal` no longer shape user data independently, and centralize DM counterpart resolution in `frontend/src/utils/directMessage.ts`.
- [x] Update `frontend/src/stores/auth.ts` to clear the active feature presence store during logout/session expiry.
- [x] Remove or quarantine the legacy presence store so new code cannot accidentally keep using it.
- [x] Add regression tests for reaction alias dedupe and session presence cleanup.

### Manual Verification Commands
1. `cd frontend && npm run build`
2. `cd frontend && npm run dev`
3. In an authenticated channel, react with `👍` to a message that already has `+1` or `thumbsup`, then remove it again. Verify one reaction bucket is rendered throughout.
4. Open a direct message and compare the DM sidebar row, channel info panel, and user profile modal. Verify availability and custom status agree across all three surfaces.
5. Log out, then log back in as the same or another user. Verify no stale presence badges survive session cleanup.

### Parallelization
- Lane A: reaction normalization hardening
- Lane B: shared presence/status architecture
- Lane C: UI rollout on top of Lane B

### Readiness
- Ready for implementation once the branch follows the locked Option B architecture above.

### Outcome
- Implemented on branch `ui-improvements-3` and pushed at commit `f4cea640` (`feat: centralize presence status handling`).
- Automated verification completed:
  - `cd frontend && npm run test:unit`
  - `cd frontend && npm run build`
  - `cd frontend && npm run test:e2e`
- Result: PASS, `60 passed`.
- Follow-up gaps remain around cross-user custom status freshness, backend status contract consistency, and deeper reaction/status regression coverage. Those are tracked in the `2026-03-29 Presence/Status Baseline and Contract Closure` section below.

## 2026-03-29 Presence/Status Baseline and Contract Closure

### Task
- Freeze a baseline for the shipped reaction/presence/status work.
- Close the remaining UI and backend contract gaps without reopening the broader shell redesign.
- Make presence and custom status behavior consistent enough that DM surfaces, profile surfaces, and compatible clients stop drifting.

### Shipped Baseline
- Reaction identity is normalized in `frontend/src/utils/emoji.ts`, `frontend/src/stores/messages.ts`, and `frontend/src/components/channel/MessageItem.vue`.
- Shared status presentation exists in `frontend/src/features/presence/presencePresentation.ts`.
- Shared user summary hydration exists in `frontend/src/composables/useUserSummary.ts`.
- Session cleanup clears active presence state and summary cache in `frontend/src/stores/auth.ts`.
- Backend presence/status endpoints and websocket `status_change` flow are active in `backend/src/api/v4/status.rs`.

### Remaining Gaps
- Cross-user custom status text/emoji is not live-updated after initial fetch for DM/profile surfaces.
- `frontend/src/components/layout/ChannelSidebar.vue` still shapes DM counterpart data separately instead of consuming the shared user-summary path.
- Backend team-member presence payloads are inconsistent between `/api/v4/teams/{team_id}/members` and `/api/v4/users/.../teams/members`.
- Custom status expiry semantics need to be explicit and enforced, not just stored.
- Regression coverage is still missing for message-store reaction merge/remove behavior and backend custom-status propagation/expiry.

### Implementation Checklist
- [x] Extend the backend realtime status contract so other-user custom status updates can reach live clients.
- [x] Decide and implement custom-status expiry semantics server-side, including clear behavior after expiry.
- [x] Normalize `frontend/src/components/layout/ChannelSidebar.vue` onto the shared user-summary path for DM identity/status rendering.
- [x] Align backend team-member endpoints so presence is consistent across equivalent routes.
- [x] Add frontend regression tests for `frontend/src/stores/messages.ts` reaction merge/remove behavior.
- [x] Add backend regression tests for custom-status websocket propagation and expiry behavior.

### Manual Verification Commands
1. `cd /Users/scolak/Projects/rustchat/frontend && npm run dev`
2. In one session, open a DM with another user. In a second session, change that other user’s custom status text/emoji. Verify the DM sidebar row, channel info panel, and user profile modal all update without a full reload.
3. Set a timed custom status, wait past the expiry boundary, and verify the status is cleared in both REST reads and websocket-driven UI.
4. Compare `GET /api/v4/teams/{team_id}/members` with `GET /api/v4/users/me/teams/members` for the same user set and verify presence fields are consistent.

### Parallelization
- Lane A: backend status/custom-status contract and tests
- Lane B: frontend DM/sidebar/user-summary unification
- Lane C: frontend regression coverage for reaction/state behavior, after Lane B contract is fixed

### Readiness
- Ready for implementation only after the follow-up status contract in `SPEC.md` is approved.

### Progress Notes
- Custom-status expiry is now enforced two ways: lazily on REST reads and proactively by a backend worker that clears expired rows and broadcasts `status_change` updates to connected clients.
- The expiry worker poll interval is configurable via `RUSTCHAT_STATUS_EXPIRY_POLL_INTERVAL_SECONDS`; default is `5` seconds.

### Verification Status
1. `cd /Users/scolak/Projects/rustchat/frontend && npm run test:unit`
- Result: PASS

2. `cd /Users/scolak/Projects/rustchat/frontend && npm run build`
- Result: PASS

3. `cd /Users/scolak/Projects/rustchat/backend && cargo test --test api_v4_mobile_presence -- --nocapture`
- Result: PASS

4. `cd /Users/scolak/Projects/rustchat/frontend && npm run test:e2e`
- Result: PASS (`60 passed`)

## 2026-03-28 CI Required-Check Alignment for Frontend-Only PRs

### Task
- Fix the GitHub Actions/branch-protection mismatch that blocks frontend-only PRs even when all visible checks are green.
- Ensure the required `cargo check + test` status is always reported on PRs to `main`.
- Preserve full backend validation when backend-related files actually change.

### Implementation Status
- [x] Removed the top-level PR path filter from [backend-ci.yml](/Users/scolak/Projects/rustchat/.github/workflows/backend-ci.yml) so the workflow always starts for PRs targeting `main`.
- [x] Added an internal backend-change detection job that compares the current ref against the PR base or push predecessor SHA.
- [x] Split heavy backend validation into a separate conditional job that only runs when backend-related files change.
- [x] Added a lightweight always-reporting gate job named `cargo check + test` that passes with a clear skip message for frontend-only changes and fails if backend validation fails when required.

### Verification Status
1. `ruby -e 'require "yaml"; YAML.load_file(".github/workflows/backend-ci.yml"); puts "YAML ok"'`
- Result: PASS

### Manual Verification Commands
1. `gh pr checks 64`
2. Open the GitHub Actions run for a frontend-only PR and confirm:
- `cargo check + test` appears as a completed successful check
- the log shows `No backend-related changes detected; backend validation skipped.`
3. Open a backend-touching PR and confirm the `Backend validation` job runs full cargo validation before the `cargo check + test` gate concludes successfully.

### Readiness
- Ready for PR validation on GitHub Actions.
- Known limitation: local verification here is limited to YAML parsing; the full behavior depends on a live Actions run.

## 2026-03-28 Emoji Picker Overlay and Clickability Fix

### Task
- Fix shared emoji picker positioning so every trigger opens the overlay from the correct anchor.
- Stop the message reaction picker from becoming unclickable when moving the pointer from the message row into the teleported picker.
- Correct the Mattermost composer so the picker inserts emoji glyphs directly instead of treating them like `:shortcode:` autocomplete values.

### Implementation Status
- [x] Retokenized `frontend/src/components/atomic/EmojiPicker.vue` to the active design token system while keeping the teleported fixed-position overlay behavior.
- [x] Added anchored picker positioning and hover-safe close handling in `frontend/src/components/channel/MessageItem.vue` so reaction pickers stay clickable.
- [x] Added anchored picker positioning in `frontend/src/components/settings/StatusPicker.vue`.
- [x] Fixed `frontend/src/components/composer/MattermostComposer.vue` so picker selections insert glyphs at the cursor position and keep focus/selection behavior intact.

### Verification Status
1. `cd frontend && npm run build`
- Result: PASS
- Note: existing Vite warning about `frontend/src/stores/calls.ts` being both dynamically and statically imported still appears, but the build completes successfully.

### Manual Verification Commands
1. `cd frontend && npm run dev`
2. Open a channel, hover a message, click the reaction smile button, and move the pointer into the picker. Verify the picker stays open and every emoji remains clickable.
3. Open the status picker and confirm the emoji overlay opens aligned to the trigger and remains above the modal/backdrop.
4. Switch to the Mattermost composer, insert an emoji from the picker, and verify the actual emoji glyph is inserted at the cursor without breaking typing focus.

### Readiness
- Ready for user acceptance testing.
- Remaining follow-up: if we want emoji name search instead of glyph-only filtering, that should be a separate UX improvement pass.

## 2026-03-28 Settings, Profile, and Admin Normalization Pass

### Task
- Normalize the highest-traffic settings, profile, and first admin surfaces to the same design system as the redesigned app shell.
- Remove remaining hardcoded color drift from theme-sensitive settings views.
- Improve hierarchy and readability so theme choices remain trustworthy across settings and admin workflows.

### Implementation Status
- [x] Added stronger personal-context framing in `frontend/src/components/settings/SettingsModal.vue`.
- [x] Retokenized and restructured `frontend/src/components/settings/notifications/NotificationsTab.vue` with calmer rows, callouts, and action treatment.
- [x] Normalized `frontend/src/components/settings/profile/ProfileTab.vue` to the shared token system and improved section hierarchy.
- [x] Aligned the dedicated settings page in `frontend/src/views/settings/ProfileView.vue` with section cards for identity, appearance, and typography.
- [x] Redesigned `frontend/src/views/admin/AdminDashboard.vue` to use restrained token-driven stat cards and operational health panels.

### Verification Status
1. `cd frontend && npm run build`
- Result: PASS
- Note: existing Vite warning about `frontend/src/stores/calls.ts` being both dynamically and statically imported still appears, but the build completes successfully.

### Manual Verification Commands
1. `cd frontend && npm run dev`
2. Open `Settings -> Notifications` and verify expanded rows, badges, callouts, and actions remain readable in `Light`, `Dark`, `Futuristic`, and `High Contrast`.
3. Open `Settings -> Profile` and confirm the avatar controls, read-only email field, banners, and status section stay visually consistent with the shell.
4. Open `/settings/profile` and verify the identity, appearance, and typography cards remain readable on both desktop and mobile widths.
5. Open the admin dashboard and confirm stats, health indicators, and instance panels feel calmer while preserving clear service status cues.

### Readiness
- Ready for user acceptance testing.
- Remaining follow-up: broaden the same normalization approach to additional admin/settings surfaces if this direction feels right in live use.

## 2026-03-28 Design System and App Shell Redesign

### Task
- Analyze RustChat’s current product usability and visual language against the standard set by Slack and Mattermost.
- Establish a concrete design system and product direction for the web UI.
- Apply the first implementation pass to the authenticated app shell so the product feels more intentional, more usable, and less like generic SaaS.

### Implementation Status
- [x] Wrote the product design system and design direction to `DESIGN.md`.
- [x] Added `CLAUDE.md` guidance so future UI work is expected to follow `DESIGN.md`.
- [x] Redesigned the app frame and shell layering in `frontend/src/components/layout/AppShell.vue`.
- [x] Rebalanced brand, search, and user-utility hierarchy in `frontend/src/components/layout/GlobalHeader.vue`.
- [x] Strengthened team rail and channel sidebar scan hierarchy in `frontend/src/components/layout/TeamRail.vue` and `frontend/src/components/layout/ChannelSidebar.vue`.
- [x] Improved channel context emphasis in `frontend/src/components/channel/ChannelHeader.vue`.
- [x] Reworked loading and empty-channel states in `frontend/src/components/channel/MessageList.vue` so quiet channels feel intentional instead of abandoned.

### Verification Status
1. `cd frontend && npm run build`
- Result: PASS

### Manual Verification Commands
1. `cd frontend && npm run dev`
2. Open the authenticated app shell and verify:
   - global header brand/context feel stronger than search chrome
   - team rail and channel sidebar are faster to scan
   - selected channel state is clear without feeling loud
   - channel header communicates current context cleanly
   - empty or quiet channels feel intentional, not broken
3. Check responsive behavior:
   - mobile left drawer
   - desktop shell with RHS open
   - channel with and without topic text

### Readiness
- Ready for user acceptance testing.
- Remaining follow-up: carry the same system into broader settings, modal, and admin surfaces over time.
## 2026-03-28 Theme Source-of-Truth Fix + Brand/Typography Second Pass

### Task
- Strengthen RustChat’s typography and brand presence so the authenticated app shell feels less generic.
- Fix theme-driven contrast issues where filled buttons and text could become unreadable on certain presets.
- Remove hardcoded neutral colors from the main profile and display settings surfaces so theme selection actually propagates.

### Implementation Status
- [x] Warmed the default light/dark palette and switched the default chat font family toward `IBM Plex Sans` in `frontend/src/style.css`.
- [x] Added a `brand-foreground` token so filled controls can keep readable text across all shipped theme presets.
- [x] Updated theme metadata and defaults in `frontend/src/stores/theme.ts` to align the UI selector with the active runtime theme system.
- [x] Replaced the broken faux-custom theme editor with a preset-only, token-backed editor in `frontend/src/components/settings/display/ThemeEditor.vue`.
- [x] Retokenized the display settings rows in `frontend/src/components/settings/display/DisplayTab.vue` and the profile settings page in `frontend/src/views/settings/ProfileView.vue`.
- [x] Applied contrast-safe filled-button text to shared shell controls in the composer, headers, rails, and related settings surfaces.

### Verification Status
1. `cd frontend && npm run build`
- Result: PASS

### Manual Verification Commands
1. `cd frontend && npm run dev`
2. Login, open `Settings -> Display`, and switch between `Light`, `Dark`, `Futuristic`, and `High Contrast`.
3. Verify the theme preview cards, save button, composer send button, team rail selection, and profile/settings actions keep readable label contrast.
4. Open `/settings/profile` and confirm font, font size, and theme changes apply live without gray overrides on the page chrome or form controls.

### Readiness
- Ready for user acceptance testing.
- Remaining known limitation: `frontend/src/features/theme/*` still exists as an inactive parallel theme stack; runtime behavior now stays on `frontend/src/stores/theme.ts`.

## 2026-03-13 WebSocket Token Expiry Enforcement

### Task
- Stop realtime websocket message delivery after JWT token expiration, even without page refresh.
- Force web UI to clear authenticated screen state and navigate to login immediately when JWT expires.

### Implementation Status
- [x] Added shared websocket auth context (`user_id` + `expires_at`) parsing in `backend/src/api/websocket_core.rs`.
- [x] Enforced token-expiry runtime disconnect in `/api/v4/websocket` loop (`backend/src/api/v4/websocket.rs`).
- [x] Enforced token-expiry runtime disconnect in legacy `/ws` loop (`backend/src/api/ws.rs`).
- [x] Added regression tests for claims-to-websocket-auth expiry mapping.
- [x] Added JWT expiry timer in frontend auth lifecycle to trigger forced logout at token `exp` (`frontend/src/stores/auth.ts`).
- [x] Added centralized frontend session cleanup on logout (messages/channels/unreads/presence/teams/preferences/calls/UI) before login redirect.
- [x] Updated websocket client close handling to treat auth-expiry close as forced logout and suppress reconnect loops (`frontend/src/composables/useWebSocket.ts`).

### Verification Status
1. `cd backend && cargo check`
- Result: PASS

2. `cd backend && cargo clippy --all-targets --all-features -- -D warnings`
- Result: PASS

3. `cd backend && cargo test claims_to_websocket_auth -- --nocapture`
- Result: PASS

4. `cd frontend && npm run build`
- Result: PASS

### Manual Verification Commands
1. Start backend with a short token lifetime:
   - `cd backend && RUSTCHAT_JWT_EXPIRY_HOURS=1 cargo run`
2. Connect to websocket with a valid token and observe forced close after expiry boundary:
   - `npx wscat -c ws://127.0.0.1:3000/api/v4/websocket -s "<JWT_TOKEN>"`
3. Web UI check:
   - Login, keep a channel open past token expiration, verify UI redirects to `/login` and channel/message UI state is cleared.
   - Confirm websocket reconnect attempts stop while token is expired.

### Readiness
- Ready for user acceptance testing.
- Expected behavior: when JWT expires, active websocket session is closed and realtime events stop.

## Task
Small compatibility-aligned messaging fixes:
- Show date separators for older messages (not only time) in WebUI message list.
- Enforce global message edit policy (`disabled`, `enabled`, or time-limited like 30 minutes) and keep edited UX consistent.
- Make reaction click behavior toggle correctly for the current user (add on first click, remove on second).
- Clear WebUI top-ring notification indicator when unseen messages are viewed/read.

## Implementation Status
- [x] Added timeline date separators in WebUI message list rendering (`frontend/src/components/channel/MessageList.vue`).
- [x] Exposed optional post edit timestamps in frontend post API typings (`frontend/src/api/posts.ts`) for edited-state handling.
- [x] Aligned unread indicator source in global header to shared unread store (`frontend/src/components/layout/GlobalHeader.vue`).
- [x] Added server setting form field for global post edit window (`frontend/src/views/admin/ServerSettings.vue`).
- [x] Included `post_edit_time_limit_seconds` in frontend config defaults (`frontend/src/features/config/stores/configStore.ts`).
- [x] Applied formatting-consistent updates in backend post update handlers (`backend/src/api/posts.rs`, `backend/src/api/v4/posts.rs`) after policy logic integration.

## Verification Status

### Automated
1. `cd backend && cargo check`
- Result: PASS

2. `cd frontend && npm run build`
- Result: PASS

3. `cd backend && cargo clippy --all-targets --all-features -- -D warnings`
- Result: PASS (all clippy warnings fixed across the codebase)

4. `cd backend && cargo test --no-fail-fast -- --nocapture`
- Result: PARTIAL
  - Unit tests in `src/lib.rs`: PASS (`125 passed, 0 failed`)
  - Many integration targets: FAIL due missing test DB bootstrap (`RUSTCHAT_TEST_DATABASE_URL` candidates unavailable/auth failure)

5. `BASE=http://localhost:3000 ./scripts/mm_compat_smoke.sh`
- Result: FAIL (target unavailable; preflight ping connection refused)

6. `BASE=http://localhost:3000 ./scripts/mm_mobile_smoke.sh`
- Result: FAIL (no `X-MM-COMPAT: 1` header observed because local target was unavailable)

## Manual Verification Commands
1. `./scripts/clippy_check.sh` - Run clippy validation
2. `BASE=<your-running-rustchat-url> ./scripts/mm_compat_smoke.sh`
3. `BASE=<your-running-rustchat-url> ./scripts/mm_mobile_smoke.sh`
4. `curl -si <your-running-rustchat-url>/api/v4/config/client | rg -n "AllowEditPost|PostEditTimeLimit"`

## Readiness
- Requested behavior changes are implemented in code.
- Final acceptance still depends on running smoke checks against a live compatible server endpoint and DB-backed integration test environment.

---

## UIX refinement: reaction identity + shell/status alignment

### Scope completed
- [x] Normalized reaction identity so alias/name/glyph variants collapse into one reaction bucket (`frontend/src/utils/emoji.ts`, `frontend/src/stores/messages.ts`, `frontend/src/components/channel/MessageItem.vue`).
- [x] Extended reaction updates to cover loaded thread replies as well as top-level channel messages (`frontend/src/stores/messages.ts`).
- [x] Compressed the top-left brand lockup and simplified header search to a single clear action (`frontend/src/components/layout/GlobalHeader.vue`).
- [x] Theme-aligned the channel identity headline so it no longer reads as hard black against the shell (`frontend/src/components/channel/ChannelHeader.vue`).
- [x] Upgraded DM rows from abstract status dots to avatar-led identity with proper presence badges (`frontend/src/components/layout/ChannelSidebar.vue`, `frontend/src/components/ui/RcAvatar.vue`).
- [x] Redesigned member list and profile modal presence treatment with explicit status labels/icons and token-safe surfaces (`frontend/src/components/channel/ChannelMembersPanel.vue`, `frontend/src/components/modals/UserProfileModal.vue`).
- [x] Removed the admin/slate modal theme leak from team, channel, and direct-message pickers so collaboration flows use the same token-driven shell styling (`frontend/src/components/modals/BrowseTeamsModal.vue`, `frontend/src/components/modals/BrowseChannelsModal.vue`, `frontend/src/components/modals/DirectMessageModal.vue`).
- [x] Filled the direct-message details gap in the right sidebar so DM conversations show the other user’s avatar, presence, and custom status instead of only generic channel metadata (`frontend/src/components/channel/ChannelInfoPanel.vue`).

### Verification Status

#### Automated
1. `cd frontend && npm run build`
- Result: PASS
- Note: existing Vite warning remains for `frontend/src/stores/calls.ts` being both dynamically and statically imported.

### Manual Verification Commands
1. `cd /Users/scolak/Projects/rustchat/frontend && npm run dev`
2. Open a message with existing reactions, add `👍`, and confirm the same visible emoji does not render twice when websocket updates land.
3. Open a thread reply and repeat the same reaction toggle check there.
4. Review the authenticated shell header and confirm:
   - the site title uses the brand accent instead of reading as hard black
   - the brand block is less crowded
   - the center search treatment is a single-line “Search” action
5. Open the members panel and a user profile modal and confirm presence states render with consistent labels/icons for `Online`, `Away`, `Do not disturb`, and `Offline`.
6. Open `Browse Teams`, `Browse Channels`, and `Direct Messages` and confirm they inherit the regular collaboration theme instead of the old dark/admin palette.
7. Open a direct message, then open `Channel Info` in the right sidebar and confirm the other user’s avatar, presence badge, and custom status are shown.

### Readiness
- The approved UIX spec items for reaction deduplication, header/search compression, and presence/status credibility are implemented in code.
- Remaining acceptance should come from a quick live browser pass against the authenticated app to confirm the final feel matches the intended shell direction.

---

## 001-platform-foundation: RustChat Platform Foundation

### Specification
**Status**: ✅ Approved (2026-03-13)  
**Location**: `specs/001-platform-foundation/spec.md`  
**User Stories**: 8 (4 P1, 4 P2)  
**Stages**: 4-stage SaaS maturity (Ad-Hoc → Reactive → Proactive → Strategic)

### Implementation Plan
**Status**: ✅ Created (2026-03-13)  
**Location**: `specs/001-platform-foundation/plan.md`  
**Phases**: 5 (Foundation, Auth/State, Security/Compliance, AI/APIs, Frontend)

### Constitution Compliance
| Principle | Status |
|-----------|--------|
| I. Contract Compatibility | ✅ |
| II. Plan-First Validation | ✅ |
| III. Evidence-Backed Analysis | ⏳ (upstream analysis pending) |
| IV. Test Gates | ✅ |
| V. Security Discipline | ✅ |
| VI. Mobile-First | ✅ |
| VII. Product Contract | ✅ |
| VIII. Feature Safety | ✅ |
| IX. Test Coverage | ✅ |
| X. Data Sovereignty | ✅ |
| XI. Stateless Tier | ✅ |
| XII. Memory Safety | ✅ |
| XIII. Zero-Trust | ✅ |
| XIV. Federated Identity | ✅ |
| XV. Strict RBAC | ✅ |
| XVI. Audit Logging | ✅ |
| XVII. Privacy by Design | ✅ |
| XVIII. Accessibility | ✅ |
| XIX. DevSecOps | ✅ |

### Current Status
- ✅ Specification approved
- ✅ Implementation plan created
- ✅ Tasks generated (196 tasks across 11 phases)
- ⏳ **READY FOR**: Implementation → Validation

### Next Actions
1. Begin Phase 0: Foundation & Tooling
   - `cd backend && cargo init`
   - Setup Docker, CI/CD
2. Checkpoint at each stage gate
3. Run `/speckit.validate` after Phase 11

### Task Summary
| Phase | Stage | Tasks | Focus Area |
|-------|-------|-------|------------|
| 0 | Ad-Hoc | T001-T017 | Containerization, CI/CD |
| 1 | Reactive | T018-T042 | Redis, OIDC/SAML, Metrics |
| 2 | Core | T043-T062 | Messaging, Mattermost API |
| 3 | Proactive | T063-T080 | RBAC, Audit Logging |
| 4 | Proactive | T081-T093 | GDPR, Data Lifecycle |
| 5 | Proactive | T094-T110 | OpenSearch, Kafka, HA |
| 6 | Strategic | T111-T123 | MCP Protocol |
| 7 | Strategic | T124-T134 | A2A Protocol |
| 8 | Strategic | T135-T150 | OpenAPI, Webhooks, SDKs |
| 9 | Frontend | T151-T169 | Solid.js, Accessibility |
| 10 | DevSecOps | T170-T184 | Security, K8s, Monitoring |
| 11 | Validation | T185-T197 | Load Testing, Chaos |

**Total Tasks**: 196
**Estimated Duration**: 16-20 weeks (2-3 devs parallel)

### Success Criteria (from spec)
- SC-001: Platform sustains 200,000 concurrent users with p99 latency < 150ms
- SC-002: Morning auth spike (30/sec, 150k/90min) completes with <0.1% failure rate
- SC-003: Kafka fan-out achieves <5ms latency for 10,000-member channel broadcasts
- SC-004: MCP/A2A agent integration passes security audit (zero critical findings)
- SC-005: OpenAPI 3.1.1 spec generates functional SDKs for TypeScript, Python, Rust
- SC-006: Webhook delivery achieves 99.9% success rate with Standard Webhooks compliance
- SC-007: GDPR Right to Erasure completes within 30 days, cryptographically verified
- SC-008: Accessibility audit passes WCAG 2.1 AA and BITV 2.0 with zero violations
- SC-009: `cargo audit` shows zero high/critical CVEs at release
- SC-010: Zero-downtime deployment achieved via blue-green strategy

## GSTACK REVIEW REPORT

| Review | Trigger | Why | Runs | Status | Findings |
|--------|---------|-----|------|--------|----------|
| CEO Review | `/plan-ceo-review` | Scope & strategy | 0 | — | — |
| Codex Review | `/codex review` | Independent 2nd opinion | 0 | — | — |
| Eng Review | `/plan-eng-review` | Architecture & tests (required) | 1 | ISSUES OPEN | 16 issues, 2 critical gaps |
| Design Review | `/plan-design-review` | UI/UX gaps | 0 | — | — |

**UNRESOLVED:** 2
**VERDICT:** ENG REVIEW NOT CLEAR — eng review required before implementation continues
