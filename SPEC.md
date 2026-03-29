# SPEC: Categories Test Fix Without Relaxing Team Creation Permissions (2026-03-29)

## Problem Statement

`backend/tests/api_categories.rs` fails before any sidebar-category logic runs because the test setup tries to create a team through `POST /api/v1/teams` as a normal `member` user:
- [backend/tests/api_categories.rs:69](/Users/scolak/Projects/rustchat/backend/tests/api_categories.rs:69)
- [backend/tests/api_categories.rs:257](/Users/scolak/Projects/rustchat/backend/tests/api_categories.rs:257)

But the product rule is that regular `member` users should **not** be allowed to create teams. Team creation is intentionally restricted.

So the correct fix is not to relax backend permissions. The correct fix is to change the failing tests and any contradictory regression coverage so they set up teams using an allowed path.

## Goals

1. Preserve the existing restriction that `member` users cannot create teams.
2. Fix the category integration tests so they no longer rely on forbidden behavior.
3. Restore regression coverage so it matches the intended permission model.

## Non-Goals

1. No change to the live team-creation permission contract.
2. No RBAC redesign.
3. No change to sidebar category behavior unless test setup reveals a second real bug.
4. No frontend permission work in this pass.

## Scope and Contract Impact

### In Scope

1. Revert the local permission relaxation in:
   - [backend/src/api/teams.rs](/Users/scolak/Projects/rustchat/backend/src/api/teams.rs)
   - [backend/src/auth/policy.rs](/Users/scolak/Projects/rustchat/backend/src/auth/policy.rs)
2. Update [backend/tests/api_categories.rs](/Users/scolak/Projects/rustchat/backend/tests/api_categories.rs) to create teams via an allowed path.
3. Update [backend/tests/api_permissions.rs](/Users/scolak/Projects/rustchat/backend/tests/api_permissions.rs) so it asserts that members are blocked from team creation.

### Contract Impact

1. `POST /api/v1/teams` remains forbidden for `member`.
2. `POST /api/v1/teams` remains allowed only for the intended elevated role path.
3. Category tests stop depending on a forbidden bootstrap flow.

## Implementation Outline

1. Revert the local change that switched team creation from `TEAM_MANAGE` to `TEAM_CREATE`.
2. Revert the local `TEAM_CREATE` grants added to `member` and the related policy assertions.
3. Change category-test setup to one of these allowed paths:
   - register/login as `org_admin` when the test needs to create a team via API, or
   - insert the team and membership directly in the test database
4. Restore permission regression tests so:
   - `member` cannot create team
   - elevated role can create team
5. Re-run the focused suites.

## Verification Plan

Automated:
1. `cd /Users/scolak/Projects/rustchat/backend && cargo test --test api_categories -- --nocapture`
2. `cd /Users/scolak/Projects/rustchat/backend && cargo test --test api_permissions -- --nocapture`
3. `cd /Users/scolak/Projects/rustchat/backend && cargo check`
4. `cd /Users/scolak/Projects/rustchat/backend && cargo clippy --all-targets --all-features -- -D warnings`

Manual:
1. Attempt `POST /api/v1/teams` with a `member` token and confirm `403 Forbidden`.
2. Attempt the same with an elevated allowed role and confirm success.

## Risks and Notes

1. The current local workspace contains an uncommitted permission relaxation that should be reverted before this lands.
2. If many tests assume self-serve team creation, we may need a shared admin bootstrap helper to keep future tests honest.
3. Direct DB setup is acceptable for integration-test scaffolding if it keeps the API contract strict and the test intent focused.

# SPEC: Permission Guardrails Expansion for Team Settings and Create-Channel Flows (2026-03-29)

## Problem Statement

We already closed the first permission drift around team creation and channel-management affordances, but two adjacent areas still have the same failure mode:

1. Team settings are not consistently permission-gated.
   - [frontend/src/components/layout/ChannelSidebar.vue](/Users/scolak/Projects/rustchat/frontend/src/components/layout/ChannelSidebar.vue) shows `Team Settings` unconditionally from the team menu.
   - [frontend/src/components/modals/TeamSettingsModal.vue](/Users/scolak/Projects/rustchat/frontend/src/components/modals/TeamSettingsModal.vue) assumes full edit/delete/member-management access.
   - [backend/src/api/teams.rs](/Users/scolak/Projects/rustchat/backend/src/api/teams.rs) protects `PUT /teams/{id}` with team-admin or `TEAM_MANAGE`, but `DELETE /teams/{id}` currently has no permission check at all.
2. Create-channel flows are not aligned to the explicit permission model.
   - [frontend/src/components/layout/ChannelSidebar.vue](/Users/scolak/Projects/rustchat/frontend/src/components/layout/ChannelSidebar.vue) exposes `Create channel` from category headers and the footer without a shared capability gate.
   - [frontend/src/components/modals/CreateChannelModal.vue](/Users/scolak/Projects/rustchat/frontend/src/components/modals/CreateChannelModal.vue) assumes the action is available if a team is selected.
   - [backend/src/api/channels.rs](/Users/scolak/Projects/rustchat/backend/src/api/channels.rs) currently allows standard channel creation for any team member, even though the policy model already defines `CHANNEL_CREATE`.

That means we still have permission drift:
- some actions are visible when they should not be
- some destructive operations are insufficiently enforced on the backend
- the test harness does not yet cover these neighboring flows

## Goals

1. Extend the shared permission/capability layer to cover team settings and create-channel flows.
2. Make team settings obey one rule everywhere:
   - global `TEAM_MANAGE`
   - or current-team membership role `admin` / `owner`
3. Make standard channel creation obey one rule everywhere:
   - explicit `CHANNEL_CREATE` permission
   - plus current-team membership
4. Add regression tests so these permission leaks become CI failures.
5. Preserve authorized flows for team admins, org admins, system admins, and regular members where the policy explicitly allows it.

## Non-Goals

1. No full RBAC redesign.
2. No change to direct-message creation semantics unless current research shows a concrete permission bug there.
3. No admin-console redesign.
4. No sweep across every permission-sensitive screen in the product.
5. No silent change to unrelated API v4 compatibility routes in this pass.

## Scope and Contract Impact

### In Scope

1. Team settings frontend gating:
   - team menu entry visibility
   - modal lock state if opened indirectly
   - member-management actions inside the modal
2. Team settings backend enforcement:
   - add permission checks to `DELETE /api/v1/teams/{id}`
   - keep delete/update semantics aligned
3. Create-channel frontend gating:
   - category header plus-buttons
   - footer `Create channel`
   - modal lock state if opened indirectly
4. Create-channel backend enforcement:
   - require `CHANNEL_CREATE` for standard public/private channel creation
   - keep current team-membership validation
5. Regression tests:
   - backend integration coverage for team delete and channel create permissions
   - frontend unit/UI coverage for hidden or locked team-settings and create-channel affordances

### Contract Impact

1. Unauthorized team deletion should return `403 Forbidden` instead of succeeding.
2. Standard channel creation should be explicitly gated by `CHANNEL_CREATE` instead of only team membership.
3. Unauthorized users should no longer see team-settings or create-channel controls as normal affordances.
4. Existing authorized users should keep the same successful behavior.

### Out of Scope

1. Team-join policy redesign.
2. Browse teams / browse channels access-model redesign.
3. Full admin teams-management permission normalization.
4. Direct-message or group-message creation redesign unless implementation uncovers a blocker.

## Current Findings and Proposed Rules

### Team Settings

Existing backend rule for update:
- allow if user has global `TEAM_MANAGE`
- otherwise allow if current-team membership role is `admin` or `owner`

Proposed rule for this pass:
- reuse that exact rule for:
  - frontend `Team Settings` menu visibility
  - frontend `TeamSettingsModal` edit/delete/member-management controls
  - backend `DELETE /teams/{id}`

### Create Channel

Existing policy model:
- `member` includes `CHANNEL_CREATE`
- `guest` does not

Existing native backend behavior:
- standard channel creation only checks team membership

Proposed rule for this pass:
- standard public/private channel creation requires:
  - current-team membership
  - `CHANNEL_CREATE`
- frontend create-channel affordances should follow the same rule
- direct-message creation is left unchanged unless implementation shows the same mismatch exists there too

## Implementation Outline

### Phase 1: Shared Capability Expansion

Extend [frontend/src/features/permissions/capabilities.ts](/Users/scolak/Projects/rustchat/frontend/src/features/permissions/capabilities.ts) with:

1. Team capability helpers:
   - `canManageTeam(...)`
   - `useCurrentTeamManagementPermission(...)`
2. Channel creation helpers:
   - `canCreateChannel(...)`
   - optional current-team-aware helper if the UI needs team-membership context

Data sources:
- global role from `authStore.user?.role`
- current team membership from `teamStore.members` when already loaded
- team-members API fallback only if needed

### Phase 2: Frontend Affordance Guardrails

Update:
- [frontend/src/components/layout/ChannelSidebar.vue](/Users/scolak/Projects/rustchat/frontend/src/components/layout/ChannelSidebar.vue)
- [frontend/src/components/modals/TeamSettingsModal.vue](/Users/scolak/Projects/rustchat/frontend/src/components/modals/TeamSettingsModal.vue)
- [frontend/src/components/modals/CreateChannelModal.vue](/Users/scolak/Projects/rustchat/frontend/src/components/modals/CreateChannelModal.vue)

Behavior:
- hide `Team Settings` unless the user can manage the current team
- if the modal is opened indirectly without permission, show a locked/access-denied state instead of editable controls
- hide `Create channel` affordances unless the user can create channels in the current team
- if the create-channel modal is opened indirectly without permission, show a locked/access-denied state

### Phase 3: Backend Enforcement

Update [backend/src/api/teams.rs](/Users/scolak/Projects/rustchat/backend/src/api/teams.rs):
- add the same team-admin / `TEAM_MANAGE` check to `delete_team`

Update [backend/src/api/channels.rs](/Users/scolak/Projects/rustchat/backend/src/api/channels.rs):
- add explicit `CHANNEL_CREATE` enforcement for standard public/private channel creation
- keep the existing team-membership check
- leave direct-message creation unchanged unless a concrete mismatch is found during implementation

### Phase 4: Regression Harness Expansion

Backend:
- extend [backend/tests/api_permissions.rs](/Users/scolak/Projects/rustchat/backend/tests/api_permissions.rs) with:
  - unauthorized team delete returns `403`
  - authorized team admin delete succeeds
  - guest or non-privileged user cannot create a standard channel
  - member with `CHANNEL_CREATE` can still create a standard channel

Frontend:
- extend [frontend/src/features/permissions/capabilities.test.ts](/Users/scolak/Projects/rustchat/frontend/src/features/permissions/capabilities.test.ts)
- extend [frontend/src/features/permissions/permissionsUi.test.ts](/Users/scolak/Projects/rustchat/frontend/src/features/permissions/permissionsUi.test.ts)

UI cases to pin:
- `Team Settings` hidden for unauthorized users
- `Create channel` hidden for unauthorized users
- `TeamSettingsModal` locked state
- `CreateChannelModal` locked state

## Verification Plan

Automated:
1. `cd /Users/scolak/Projects/rustchat/frontend && npm run test:unit`
2. `cd /Users/scolak/Projects/rustchat/frontend && npm run build`
3. `cd /Users/scolak/Projects/rustchat/backend && cargo test --test api_permissions -- --nocapture`

Manual:
1. Log in as a low-privilege user and verify the sidebar no longer shows:
   - `Team Settings`
   - `Create channel`
2. Attempt team deletion via API as a low-privilege user and verify `403`.
3. Attempt standard channel creation via API as a low-privilege user lacking `CHANNEL_CREATE` and verify `403`.
4. Log in as an allowed user and verify:
   - team settings still opens and saves
   - standard channel creation still works

Suggested manual verification commands:
1. `curl -i -X DELETE http://127.0.0.1:3000/api/v1/teams/<team-id> -H "Authorization: Bearer $TOKEN"`
2. `curl -i -X POST http://127.0.0.1:3000/api/v1/channels -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d '{"team_id":"<team-id>","name":"gated-channel","display_name":"Gated Channel","channel_type":"public"}'`

## Risks and Notes

1. `member` currently has `CHANNEL_CREATE` in the policy engine, so this pass should not accidentally remove valid member channel creation if that is intentional.
2. Team-management frontend logic needs current-team membership data; we should reuse `teamStore.members` where possible instead of creating another parallel fetch path.
3. Backend enforcement and frontend hiding must land together, otherwise the app either looks broken or remains unsafe.
4. `DELETE /teams/{id}` is the highest-risk backend gap in this scope because it is destructive and currently unguarded.

## Success Criteria

1. Unauthorized users cannot delete teams.
2. Unauthorized users do not see or meaningfully use team-settings controls.
3. Standard channel creation follows explicit permission rules on both frontend and backend.
4. The regression harness covers these flows in both API and UI layers.
5. The new scope remains narrow and does not reopen unrelated permission work.
