# Gap Plan

- Rustchat target path: `backend/src/api/auth.rs`, `backend/src/api/v4/users.rs`, `backend/src/api/oauth.rs`
- Required behavior: Strict SSO-only server enforcement and robust account linking for OIDC users.
- Current gap: Password login still works server-side when `require_sso` is true; OIDC existing-user matching is email-first and does not re-sync roles/groups.
- Planned change:
  - Add server guards for SSO-only mode in v1/v4 login endpoints.
  - Resolve users by `(auth_provider, auth_provider_id)` before email fallback.
  - Add optional per-login role re-sync from Keycloak group mappings.
- Verification test:
  - API tests for v1/v4 login rejection under `require_sso=true`.
  - OAuth callback tests for provider-ID linking and role re-sync behavior.
- Status: pending

- Rustchat target path: new Keycloak group sync service (suggested `backend/src/services/keycloak_sync.rs`) + wiring in startup and admin trigger route
- Required behavior: Import Keycloak groups and memberships into Rustchat `groups/group_members` as authoritative identity-source data.
- Current gap: No Keycloak/Admin API importer exists; no OIDC->groups persistence.
- Planned change:
  - Upsert groups as `source='plugin_keycloak'` (syncable-compatible), `remote_id=<keycloak_group_id>`.
  - Upsert `group_members` by Keycloak user external ID mapping.
  - Trigger existing syncable reconciliation for changed groups.
- Verification test:
  - Integration tests: create/update/delete membership deltas and resulting channel/team membership convergence.
- Status: pending

- Rustchat target path: `backend/src/api/v4/groups.rs` and new internal mapping logic
- Required behavior: Keycloak groups define channel/team auto-subscriptions deterministically.
- Current gap: No mapping from Keycloak group metadata/path to `group_syncables` links.
- Planned change:
  - Define mapping contract (recommended: Keycloak group attributes, fallback: naming convention).
  - Build idempotent linker that maps each Keycloak group to team/channel syncables (`auto_add`, `scheme_admin`).
- Verification test:
  - Contract tests for mapping parser and syncable link/update/unlink behavior.
- Status: pending

- Rustchat target path: `backend/src/api/v4/users.rs`, `backend/src/api/v4/teams.rs`, `backend/src/api/v4/channels/compat.rs`, `backend/src/api/v4/websocket.rs`
- Required behavior: Functional Mattermost-compatible group association + event endpoints for mobile/web clients.
- Current gap: User/team/channel groups endpoints are stubs; group websocket events are incomplete; SSO code-exchange endpoint unsupported.
- Planned change:
  - Implement DB-backed association endpoints with MM-compatible payload and filtering.
  - Implement `/users/login/sso/code-exchange` on top of existing exchange-code service.
  - Emit group-related websocket events on group/syncable/member mutations.
- Verification test:
  - Endpoint schema tests + websocket integration tests + targeted mobile smoke test.
- Status: pending

- Rustchat target path: `backend/src/services/membership_policies.rs`, `backend/src/services/membership_reconciliation.rs`, migrations
- Required behavior: Policy applicability for `group|role|org` and schema-consistent identity attributes.
- Current gap: Queries use `users.auth_service` and `users.props` while schema/migrations clearly track `auth_provider/auth_provider_id`; non-`all_users` group semantics are incomplete.
- Planned change:
  - Align policy queries to actual schema (`auth_provider` or add explicit migration if `auth_service` is intended).
  - Implement real applicability logic for `group|role|org` using existing tables.
- Verification test:
  - Unit/integration tests for each source type and reconciliation outcomes.
- Status: pending

- Rustchat target path: `backend/src/api/v4/channels.rs` (DM/GM creation path)
- Required behavior: Optional policy that restricts who can open direct/group conversations based on Keycloak-derived ACL rules.
- Current gap: Direct/group channel creation is not group-policy constrained.
- Planned change:
  - Add optional enforcement hook (configurable) before direct/group channel creation.
- Verification test:
  - API tests for allowed/denied DM/GM creation under configured constraints.
- Status: pending
