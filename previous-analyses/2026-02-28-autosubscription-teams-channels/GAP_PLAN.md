# Gap Plan

## Target behavior (parity-first)

- Rustchat target path: `backend/src/api/teams.rs`, `backend/src/api/v4/teams.rs`, `backend/src/api/v4/ldap.rs`, `backend/src/api/v4/groups.rs`, `frontend/src/views/admin/*`
- Required behavior:
  - Team-join auto-subscription should be policy-driven and deterministic.
  - Mattermost-compatible default channel behavior must be available.
  - LDAP/plugin group-based team/channel auto-membership must be supported in v4-compatible endpoints.
  - Admin console must expose configuration and remediation controls.
- Current gap:
  - API inconsistency (`v1/admin` add all public channels, `v4` adds none).
  - No default-channel bootstrap on team creation.
  - Group/LDAP sync endpoints are stubs or wrong HTTP method.
  - No policy UI.
- Planned change:
  - Implement parity baseline first, then optional richer policies.
- Verification test:
  - Cross-API contract tests (`v1`, `v4`) for team join/member add paths.
- Status: planned

## Design problems observed in Mattermost (must avoid when implementing)

- Rustchat target path: design principles for new module
- Required behavior: Avoid hidden or fragile policy behavior.
- Current gap: none yet (greenfield in rustchat).
- Planned change:
  - Do not silently ignore membership-application errors for mandatory policies.
  - Expose per-policy reconciliation status and last-run result in admin UI.
  - Keep per-team and per-segment scope; avoid purely global defaults as the only option.
  - Support non-LDAP sources for policy applicability in OSS mode.
- Verification test:
  - Fault-injection tests where one channel assignment fails; assert surfaced error/report.
- Status: planned

## Data model plan

### P1. Membership policy storage
- Rustchat target path: new migration + new repository/service module (`backend/src/services/membership_policies.rs`)
- Required behavior: Represent who should be auto-subscribed to what.
- Current gap: No policy entities.
- Planned change:
  - `auto_membership_policies`
    - `id`, `name`, `scope_type` (`global|team`), `team_id nullable`, `source_type` (`all_users|auth_service|group|role|org`), `enabled`, `created_at`, `updated_at`.
  - `auto_membership_policy_targets`
    - `policy_id`, `target_type` (`team|channel`), `target_id`, `role_mode` (`member|admin` optional).
  - `auto_membership_policy_audit`
    - `policy_id`, `run_id`, `user_id`, `action`, `status`, `error`, `created_at`.
  - `membership_origin`
    - extend team/channel membership rows (or companion table) with `origin` (`manual|policy|invite|sync`) to avoid destructive removals of manual membership.
- Verification test:
  - Migration tests + CRUD tests + uniqueness/foreign key constraints.
- Status: planned

### P2. Default channel bootstrap at team creation
- Rustchat target path: `backend/src/api/teams.rs` and equivalent service path used by v4/create-team flow
- Required behavior: New team gets canonical default channels.
- Current gap: No default channels created.
- Planned change:
  - Create `town-square` + `off-topic` public channels on team creation.
  - Mark canonical default channel metadata (`is_default=true` if schema extension is used).
- Verification test:
  - Team creation integration test asserting both channels exist.
- Status: planned

## API plan

### A1. Fix v4 LDAP sync-membership contract
- Rustchat target path: `backend/src/api/v4/ldap.rs`
- Required behavior: `POST /api/v4/ldap/users/{user_id}/group_sync_memberships` action endpoint.
- Current gap: route is GET and stubbed.
- Planned change:
  - Change route to POST.
  - Enforce auth-service and permission checks.
  - Trigger scoped policy/group reconciliation for target user.
- Verification test:
  - Contract tests for 200/400/403/501 semantics.
- Status: planned

### A2. Implement real group syncables API behavior
- Rustchat target path: `backend/src/api/v4/groups.rs`
- Required behavior: link/patch/unlink group syncables produce membership effects and retrievable state.
- Current gap: endpoints return stubs.
- Planned change:
  - Persist groups and syncables.
  - On link/patch/unlink schedule reconciliation job (async worker) and expose status.
- Verification test:
  - API tests for link->membership creation and unlink->constraint cleanup semantics.
- Status: planned

### A3. Unify team-member add/join behavior across API surfaces
- Rustchat target path: `backend/src/api/teams.rs`, `backend/src/api/admin.rs`, `backend/src/api/v4/teams.rs`
- Required behavior: all entry points share one policy application function.
- Current gap: inconsistent behavior between v1/admin and v4.
- Planned change:
  - Central service `apply_auto_membership_for_team_join(user_id, team_id, trigger)`.
  - Replace direct SQL fan-out to all public channels with policy-driven target selection.
- Verification test:
  - Parametric tests across each endpoint with same expected channel memberships.
- Status: planned

## Admin console plan

### U1. Policy management UI
- Rustchat target path: new admin views (e.g. `frontend/src/views/admin/AutoMembershipPolicies.vue`)
- Required behavior:
  - Create/edit/disable policies.
  - Select targets (teams/channels).
  - Select source filter (all users, LDAP, SAML, role, group).
  - Preview impact before save.
- Current gap: no policy UI.
- Planned change:
  - Add admin navigation entry and CRUD forms.
  - Add dry-run endpoint returning candidate users and resulting memberships.
- Verification test:
  - Frontend component tests + E2E policy create/edit/delete.
- Status: planned

### U2. User-level re-sync action
- Rustchat target path: user admin actions panel (equivalent to Mattermost "Re-sync user via LDAP groups")
- Required behavior: one-click reconciliation for a selected user.
- Current gap: missing.
- Planned change:
  - Add action button gated by permissions/source type.
  - Call corrected POST LDAP sync-memberships endpoint.
  - Display operation result and failures.
- Verification test:
  - UI + API integration test for successful and rejected flows.
- Status: planned

## Execution model and reliability

### R1. Reconciliation worker
- Rustchat target path: new background job module
- Required behavior: idempotent async membership convergence after policy changes and group updates.
- Current gap: no worker.
- Planned change:
  - Queue tasks by `(policy_id,user_id)` or `(syncable_id,scope)`.
  - Idempotent upserts for membership add/remove based on `membership_origin`.
  - Store job result in `auto_membership_policy_audit`.
- Verification test:
  - Retry/idempotency tests and crash-recovery tests.
- Status: planned

### R2. Failure visibility
- Rustchat target path: admin API + UI status widgets
- Required behavior: admins can detect partial failures quickly.
- Current gap: no status/audit surface.
- Planned change:
  - Add endpoints for last run status and failed operations.
  - Add UI warning badges and downloadable error list.
- Verification test:
  - Synthetic failure injection test + UI rendering checks.
- Status: planned

## External benchmark-informed decisions

- Rustchat target path: policy product requirements
- Required behavior: combine parity with pragmatic admin UX.
- Current gap: no formal requirements.
- Planned change:
  - From Zulip: keep simple org-wide defaults for fast onboarding.
  - From Slack/Rocket.Chat: allow explicit default channel list for new users.
  - From Mattermost: keep group-sync model for enterprise identity systems.
  - From Teams: allow dynamic-membership style group-driven assignment where identity data exists.
- Verification test:
  - Product acceptance checklist mapped to benchmark capabilities.
- Status: planned

## Delivery phases

### Phase 1 (parity baseline)
- Implement team default channel bootstrap and unified join/add semantics.
- Fix v4 LDAP route method + minimal action behavior.
- Add regression tests for new-user/team-join auto-membership.

### Phase 2 (group syncable parity)
- Implement real groups/syncables persistence and link/unlink behaviors.
- Add user-level re-sync endpoint + admin action.

### Phase 3 (full policy UI and reliability)
- Introduce policy CRUD UI, dry-run preview, reconciliation/audit dashboards.
- Add origin-aware removal logic and failure surfacing.

## Verification matrix

- Backend:
  - `cargo test` for new membership policy service and API contract tests.
  - Endpoint parity tests for `v1`, `v4`, admin flows.
- Frontend:
  - component tests for policy editor and user re-sync action.
  - E2E for policy create -> user join -> expected memberships.
- Compatibility:
  - smoke checks for `/api/v4/ldap/users/{user_id}/group_sync_memberships` and group syncable routes.

## Implementation status update (2026-02-28)

- Completed checks:
  - `P2` default channel bootstrap is implemented in team creation path via `ensure_default_channels_for_team`.
  - `A1` LDAP sync-membership route method is corrected to `POST`.
  - `A2` group syncables persistence + link/patch/unlink membership effects are implemented (see dedicated iteration: `previous-analyses/2026-02-28-groups-syncables-a2/`).
  - `A3` team-join/member-add paths use shared default-channel application service with soft-fail parity behavior.
  - v4 invite flow now supports DB-backed `invite_id`, one-time `token`, and invite-id regeneration.
- Test evidence:
  - `backend/tests/api_team_autosubscription.rs`:
    - `add_team_member_succeeds_when_default_channel_autojoin_fails`
    - `v4_add_team_member_succeeds_when_default_channel_autojoin_fails`
    - `v4_add_team_member_by_invite_succeeds_when_default_channel_autojoin_fails`
    - `v4_add_team_member_by_token_uses_one_time_token`
    - `create_team_bootstraps_fallback_default_channels_and_joins_creator`
    - `create_team_bootstraps_configured_default_channels_and_joins_creator`
  - `backend/tests/api_v4_groups_syncables.rs`:
    - `v4_team_syncable_link_patch_and_retrieve`
    - `v4_channel_syncable_link_and_unlink_cleans_memberships`
- Remaining risks:
  - `U2`, `R1`, and `R2` are still open (user-level re-sync UI/action, durable reconciliation worker, and failure/audit surfacing).
  - `U1` is partially implemented: baseline admin configuration for default channels is now exposed, but full policy CRUD/preview is still pending.
