# SPEC: Permission Guardrails and Regression Harness (2026-03-29)

## Problem Statement

RustChat currently has permission drift between backend enforcement and frontend affordances.

Concrete findings from the current codebase:

1. Team creation is effectively unguarded on the native backend route.
   - [backend/src/api/teams.rs](/Users/scolak/Projects/rustchat/backend/src/api/teams.rs#L74) creates a team without checking `TEAM_CREATE`, `TEAM_MANAGE`, or any equivalent permission.
   - This matches the bug report that a regular member can create a team when they should not.
2. Team-creation UI is exposed unconditionally.
   - [frontend/src/components/layout/TeamRail.vue](/Users/scolak/Projects/rustchat/frontend/src/components/layout/TeamRail.vue#L67) always shows the `Create Team` affordance.
   - [frontend/src/components/modals/CreateTeamModal.vue](/Users/scolak/Projects/rustchat/frontend/src/components/modals/CreateTeamModal.vue#L1) has no permission gate and assumes the action is available.
3. Channel-management affordances are visible even when the action may be forbidden.
   - [frontend/src/components/channel/ChannelInfoPanel.vue](/Users/scolak/Projects/rustchat/frontend/src/components/channel/ChannelInfoPanel.vue#L282) shows the `Edit` action unconditionally.
   - [frontend/src/components/modals/ChannelSettingsModal.vue](/Users/scolak/Projects/rustchat/frontend/src/components/modals/ChannelSettingsModal.vue#L1) assumes the user is allowed to update membership/settings and relies on API failure rather than capability gating.
4. The frontend currently uses scattered role checks instead of one shared capability model.
   - Example: [frontend/src/components/layout/ChannelSidebar.vue](/Users/scolak/Projects/rustchat/frontend/src/components/layout/ChannelSidebar.vue#L176) uses coarse role checks like `system_admin` / `org_admin`.
   - This increases the chance that one surface hides an action while another exposes it.

This is more than one bug. It is a missing permission-safety mechanism.

## Goals

1. Enforce team-creation permissions on the backend.
2. Hide team/channel-management affordances in the frontend when the current user lacks permission.
3. Centralize permission/capability checks in one shared frontend layer instead of ad hoc role checks.
4. Add regression tests that fail when privileged actions become available to unprivileged users.
5. Make permission bugs cheap to detect before landing.

## Non-Goals

1. No full RBAC redesign or permission-schema migration in this pass.
2. No dynamic per-org custom role editor redesign.
3. No large admin-console overhaul.
4. No attempt to solve every permission edge in the product at once.
5. No silent compatibility change to unrelated Mattermost routes.

## Scope and Contract Impact

### In Scope

1. Native backend team-creation enforcement.
2. Shared frontend capability helpers for the currently affected surfaces.
3. Hiding or disabling:
   - `Create Team`
   - channel settings / rename-edit entry points
   - channel member-management entry points where the user cannot manage the channel
4. Regression tests at:
   - backend permission-enforcement level
   - frontend capability-rendering level
   - one focused end-to-end or smoke path for a low-privilege user

### Contract Impact

1. Unauthorized team creation should return `403 Forbidden` instead of succeeding.
2. Unauthorized UI actions should no longer be visible as normal controls.
3. Existing authorized flows should remain unchanged.

### Out of Scope for This Pass

1. Full audit of every admin/system endpoint.
2. Dynamic permission sync from backend role tables into the frontend.
3. Team-settings permissions redesign beyond the affected create/manage surfaces.

## Proposed Approach

### Phase 1: Lock Backend Enforcement

Target files:
- [backend/src/api/teams.rs](/Users/scolak/Projects/rustchat/backend/src/api/teams.rs)
- backend tests covering native team creation

Work:
1. Add an explicit permission check to `create_team`.
2. Use the existing policy model rather than inventing a new rule.
3. Choose and document the exact permission used:
   - preferred: `TEAM_CREATE`
   - fallback if current policy semantics require it: `TEAM_MANAGE`
4. Add backend regression tests proving:
   - `member` cannot create a team
   - authorized admin role can create a team

### Phase 2: Introduce a Frontend Capability Layer

Target files:
- `frontend/src/features/auth/...` or a new focused capability helper under `frontend/src/features/permissions/`
- [frontend/src/components/layout/TeamRail.vue](/Users/scolak/Projects/rustchat/frontend/src/components/layout/TeamRail.vue)
- [frontend/src/components/modals/CreateTeamModal.vue](/Users/scolak/Projects/rustchat/frontend/src/components/modals/CreateTeamModal.vue)
- [frontend/src/components/channel/ChannelInfoPanel.vue](/Users/scolak/Projects/rustchat/frontend/src/components/channel/ChannelInfoPanel.vue)
- [frontend/src/components/modals/ChannelSettingsModal.vue](/Users/scolak/Projects/rustchat/frontend/src/components/modals/ChannelSettingsModal.vue)
- [frontend/src/components/layout/ChannelSidebar.vue](/Users/scolak/Projects/rustchat/frontend/src/components/layout/ChannelSidebar.vue)

Work:
1. Create one shared capability source for the current user, based on:
   - global role
   - current channel membership role where needed
2. Add helpers like:
   - `canCreateTeam`
   - `canManageChannel(channel, membership)`
   - `canEditChannelSettings(channel, membership)`
3. Replace ad hoc role-string checks in touched surfaces with these helpers.
4. Hide privileged actions entirely by default unless product needs disabled-state discoverability.

### Phase 3: Build the Regression Harness

#### Backend

Add integration tests for permission-sensitive endpoints using low-privilege and admin users.

First table to cover:
1. `POST /api/v1/teams`
2. `PUT /api/v1/channels/{id}`
3. `DELETE /api/v1/channels/{id}` or equivalent privileged channel action
4. add/remove channel member routes where admin-only behavior is intended

Goal:
- a small permission matrix test file that makes privilege regressions obvious

#### Frontend

Add component/unit tests that assert affordance visibility for:
1. member user
2. org/system admin
3. channel admin vs non-admin member

Goal:
- if a forbidden button reappears in the UI, CI fails before manual QA

#### Optional E2E Smoke

Add one focused Playwright scenario:
1. log in as low-privilege user
2. verify `Create Team` is absent
3. open a normal channel
4. verify channel edit/settings affordance is absent

This is not the primary safety net, but it gives a high-signal user-path check.

## Test Strategy

### Required

1. Backend integration tests for permission enforcement.
2. Frontend unit/component tests for hidden affordances.
3. Existing relevant test suites still pass.

### Recommended Manual Verification

1. Log in as `member` and verify no `Create Team` action exists in the team rail.
2. Attempt `POST /api/v1/teams` as `member` and verify `403`.
3. Open a channel as a non-admin member and verify no edit/settings affordance is visible.
4. Attempt the corresponding update API as a non-admin and verify `403`.
5. Repeat with an authorized admin user and verify the actions are visible and succeed.

## Risks and Edge Cases

1. Permission-name mismatch:
   - the policy model defines `TEAM_CREATE`, but the current app may have historically used `TEAM_MANAGE` as the real gate.
   - We must choose one rule and apply it consistently.
2. Channel membership role vs global role:
   - a user may be allowed globally but not be channel admin locally, or vice versa.
   - Frontend helpers must model both layers.
3. UI hiding without backend enforcement is insufficient.
4. Backend enforcement without UI cleanup creates noisy broken affordances.

## Success Criteria

1. A member who lacks the intended privilege cannot create a team.
2. A member who lacks channel-management privilege does not see edit/manage affordances in the touched surfaces.
3. Authorized admins still see and can use these actions.
4. Regression tests cover both API enforcement and UI affordance visibility.
5. Permission regressions in these areas become CI failures instead of manual surprises.
