# Frontend Solid Phase B Gap Register (2026-03-14)

This register tracks remaining Vue-to-Solid WebUI parity gaps after Phase A blockers were implemented.

## Scope
- Baseline reference: `archive/frontend-vue-backup/src`.
- Current target: `frontend-solid/src`.
- Focus: user-visible behavior parity (auth/navigation/admin/settings), not full backend/API redesign.

## Priority Legend
- `P0`: Blocks normal user/admin journeys.
- `P1`: High-impact UX parity gap.
- `P2`: Important but non-blocking.
- `P3`: Polish and long-tail parity.

## Gap Register

| ID | Priority | Area | Expected (Vue) | Current (Solid) | Status | Next Action |
|----|----------|------|----------------|------------------|--------|-------------|
| PB-001 | P0 | Auth error UX | Friendly auth errors even when backend returns non-JSON errors | Could surface JSON parse exceptions on login/register failures | Fixed | Keep regression covered in auth flow checks |
| PB-002 | P0 | Auth navigation discoverability | Login page provides stable sign-up path when allowed/default policy | Register link could disappear when auth policy fetch failed | Fixed | Validate against live backend policy variants |
| PB-003 | P0 | Admin discoverability | Admin Console is visible for admin users in primary navigation | Admin entry existed in menu/mobile only | Fixed | Keep role gating consistent across nav surfaces |
| PB-004 | P1 | Admin functional depth | Vue admin had many concrete subpages (users, teams, policies, health, etc.) | Solid now has functional overview, users, teams, server settings, security, compliance, and audit endpoint-backed workflows | In Progress | Continue expanding advanced admin workflows (membership policies/email/health drilldowns) |
| PB-005 | P1 | Settings parity depth | Vue settings modal includes richer tab behavior and plugin sections | Solid overlay now includes admin-only configuration section (`/settings/configuration`) with save/reload flow | In Progress | Continue missing advanced plugin/security controls |
| PB-006 | P1 | Notification behavior | Vue wired unread-driven notifications and indicator behavior | Solid header dropdown now renders unread/mention notifications from store data with channel navigation + read actions | Fixed | Add richer timeline/event metadata if required |
| PB-007 | P2 | Team/workspace switcher | Vue had richer multi-team behavior | Solid sidebar team selector now loads `/api/v1/teams`, shows team unread badges, and switches teams/channels | Fixed | Add create/join team flow wiring in a future slice |
| PB-008 | P2 | E2E parity baseline | Stable selectors and route expectations for auth/settings parity checks | Selector parity improved, but full auth/settings suite still blocked in local run without live auth backend | In Progress | Keep page objects aligned; rerun against live backend credentials |
| PB-009 | P0 | Admin role gating reliability | Admin surfaces should remain visible for multi-role admin principals | Exact role-string match could hide admin links/route for multi-role values | Fixed | Covered by unit role parser tests |
| PB-010 | P0 | Settings close loop | Closing settings should always exit overlay to app destination | History fallback could bounce through `/login?redirect=...` and reopen settings | Fixed | Keep deterministic close resolver and deep-link checks |
| PB-011 | P0 | Profile save persistence | Username/profile edits persist and rehydrate after save/reopen | Local form state could drift from refreshed auth profile after save | Fixed | Keep profile rehydrate behavior under regression coverage |
| PB-012 | P1 | Security password flow | Security tab should call API and validate current password | UI had placeholder-only success with no API call | Fixed | Maintain backend contract and UI error mapping |
| PB-013 | P1 | Thread follow parity | Thread follow toggle should read/write real follow state | Thread view hardcoded `isFollowing={false}` | Fixed | Keep API-backed follow/unfollow wiring in route |

## Phase B Slice Implemented in This Pass

1. `PB-001` fixed:
   - Added safe error-body parsing in:
     - `frontend-solid/src/stores/auth.ts`
     - `frontend-solid/src/routes/Register.tsx`

2. `PB-002` fixed:
   - Login register-link visibility now defaults to visible when policy is unavailable:
     - `frontend-solid/src/routes/Login.tsx`

3. `PB-003` fixed:
   - Added admin entry to sidebar for authorized users:
     - `frontend-solid/src/components/layout/Sidebar.tsx`

4. `PB-008` progressed:
   - Updated auth/settings Playwright page object selectors for current Solid UI:
     - `frontend-solid/e2e/pages/LoginPage.ts`
     - `frontend-solid/e2e/pages/RegisterPage.ts`
     - `frontend-solid/e2e/pages/SettingsPage.ts`

5. `PB-006` progressed:
   - Header and mobile badges now use unread store counts; header mark-all-read is wired:
     - `frontend-solid/src/components/layout/Header.tsx`
     - `frontend-solid/src/components/layout/MobileNav.tsx`

6. `PB-004` progressed:
   - Replaced admin scaffold with functional data-backed sections:
     - `frontend-solid/src/routes/Admin.tsx`
   - Implemented:
     - Overview cards and health summary (`/api/v1/admin/stats`, `/api/v1/admin/health`)
     - Users management list with status filter/search and deactivate/reactivate actions
     - Teams section with team detail drilldown (channels + members)
     - Server settings quick-edit flow (`/api/v1/admin/config/*`)

7. `PB-005` progressed:
   - Added admin-only settings overlay section:
     - `frontend-solid/src/routes/Settings.tsx`
   - Route `/settings/configuration` now renders inside the modal overlay and supports save/reload for key server settings fields.

8. Runtime compatibility hardening:
   - Added startup `crypto.randomUUID` polyfill for older runtime environments:
     - `frontend-solid/src/utils/cryptoPolyfills.ts`
     - `frontend-solid/src/index.tsx`

9. `PB-007` fixed:
   - Replaced static sidebar team selector with API-backed switching flow:
     - `frontend-solid/src/components/layout/Sidebar.tsx`
   - Implemented:
     - Team list load from `/api/v1/teams`
     - Current-team detection from selected channel
     - Team switch action (`fetchChannels` + route navigation)
     - Team unread badges from unread store

10. `PB-006` fixed:
    - Replaced mock notification dropdown entries with unread-store-driven channel notifications:
      - `frontend-solid/src/components/layout/Header.tsx`
    - Implemented:
      - Dynamic mention/unread notification items
      - Open-notification navigates to target channel
      - Read-state update via `markAsRead`

11. `PB-004` progressed:
    - Replaced remaining `security/compliance/audit` placeholders with endpoint-backed sections:
      - `frontend-solid/src/routes/Admin.tsx`
    - Implemented:
      - Security: SSO + permission inventory (`/api/v1/admin/sso`, `/api/v1/admin/permissions`, role permissions)
      - Compliance: retention config edit/save (`/api/v1/admin/config/compliance`)
      - Audit: recent audit log table (`/api/v1/admin/audit`)

12. `PB-009` fixed:
    - Updated admin role helper to parse multi-role strings (`space`/`comma` separated):
      - `frontend-solid/src/utils/roles.ts`
    - Added unit coverage for multi-role cases:
      - `frontend-solid/tests/auth/authRedirect.test.ts`

13. `PB-010` fixed:
    - Reworked settings close behavior to deterministic fallback order (stored non-settings return target -> current/default channel -> `/`) and removed history-back auth-loop behavior:
      - `frontend-solid/src/routes/Settings.tsx`

14. `PB-011` fixed:
    - Added profile form rehydrate sync from latest auth user when not editing; keeps saved values consistent after reopen:
      - `frontend-solid/src/routes/Settings.tsx`

15. `PB-012` fixed:
    - Implemented password change API call in security settings:
      - `frontend-solid/src/routes/Settings.tsx`
      - `frontend-solid/src/stores/user.ts`
    - Enforced required `current_password` in v1 backend contract:
      - `backend/src/models/user.rs`
      - `backend/src/api/users.rs`

16. `PB-013` fixed:
    - Replaced hardcoded thread follow state with API-backed load/follow/unfollow flow:
      - `frontend-solid/src/routes/Thread.tsx`

17. `PB-008` progressed:
    - Updated login/register/settings Playwright selectors and settings behaviors to current UI contract:
      - `frontend-solid/e2e/pages/LoginPage.ts`
      - `frontend-solid/e2e/pages/RegisterPage.ts`
      - `frontend-solid/e2e/pages/SettingsPage.ts`
      - `frontend-solid/e2e/tests/settings.spec.ts`
      - `frontend-solid/e2e/tests/auth.spec.ts`

## Verification Snapshot

- `cd frontend-solid && npm run build` -> PASS
- `cd frontend-solid && npm run test -- tests/auth/authRedirect.test.ts` -> PASS
- `cd frontend-solid && PLAYWRIGHT_WEB_SERVER=1 npm run test:e2e -- e2e/tests/auth.spec.ts --project=chromium --grep "redirect to login when accessing (protected route|settings) unauthenticated"` -> PASS (2/2)
- `cd frontend-solid && npm run build` -> PASS after team-switcher/notification/admin-section updates
- `cd backend && cargo check` -> PASS after v1 password contract update (warnings only in unrelated files)
- `cd backend && cargo test --lib change_password_ -- --nocapture` -> PASS (2/2)
- `cd frontend-solid && PLAYWRIGHT_WEB_SERVER=1 npm run test:e2e -- e2e/tests/auth.spec.ts e2e/tests/settings.spec.ts --project=chromium` -> FAIL in local-only mode (no successful live auth login; selector drift reduced, auth backend data still required)

## Recommended Next Slice (Phase B.3)

1. Add richer admin workflows for membership policies, email settings, and health diagnostics parity.
2. Expand settings overlay parity for plugin/security controls beyond the current configuration subset.
3. Add Playwright coverage for admin section rendering and team-switcher behavior.
