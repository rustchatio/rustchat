# Gap Plan

- Rustchat target path: `backend/src/api/v4/config_client.rs`, `backend/src/api/v4/system.rs`, `backend/src/mattermost_compat/models.rs`
- Required behavior: Expose MM-compatible client config/license signals for LDAP and LDAPGroups.
- Current gap: `get_client_license` returns minimal unlicensed payload and compatibility `License` model has no LDAP/LDAPGroups feature fields (`backend/src/api/v4/config_client.rs:82-100`, `backend/src/mattermost_compat/models.rs:164-173`).
- Planned change: Extend compatibility license struct and serializer to include feature booleans/strings expected by clients (`LDAP`, `LDAPGroups`), keep `EnableLdap` in config map, and align `/api/v4/system/license` payload shape.
- Verification test: API tests for `/api/v4/license/client?format=old`, `/api/v4/license/client`, `/api/v4/license` asserting presence/type/value of `EnableLdap`, `LDAP`, `LDAPGroups`, `IsLicensed`.
- Status: pending

- Rustchat target path: `backend/src/models/team.rs`, `backend/src/models/channel.rs`, DB migrations, MM compatibility mappers
- Required behavior: Team/channel contracts include `group_constrained` and can be patched/read consistently.
- Current gap: No `group_constrained` field in Rustchat core/compat models (`backend/src/models/team.rs:10-23`, `backend/src/models/channel.rs:21-35`, `backend/src/mattermost_compat/models.rs:29-63`).
- Planned change: Add DB columns (`teams.group_constrained`, `channels.group_constrained`), wire through models and MM serializers, and patch handlers.
- Verification test: API tests for team/channel create/get/patch ensuring JSON has `group_constrained` and persistence works.
- Status: pending

- Rustchat target path: `backend/src/api/v4/ldap.rs` + new LDAP service layer
- Required behavior: Implement MM-like LDAP endpoints and contracts (groups list/link/unlink, diagnostics/test, user group sync memberships, migrate ID).
- Current gap: All LDAP handlers are enterprise stubs returning static 501 (`backend/src/api/v4/ldap.rs:11-21, 47-144`); unlink route missing.
- Planned change: Replace stubs with service-backed implementation; add DELETE `/ldap/groups/{remote_id}/link`; enforce permission + feature gating and MM-like status codes/payload shapes.
- Verification test: Endpoint-level tests for success/forbidden/not-implemented/not-found behavior and response schema parity.
- Status: pending

- Rustchat target path: `backend/src/api/v4/users.rs` + auth/account migration service
- Required behavior: Support `ldap_only` login semantics and functional `/users/migrate_auth/ldap` contract.
- Current gap: Login request does not model `ldap_only` and migration endpoint is placeholder status OK (`backend/src/api/v4/users.rs:225-232`, `2923-2926`).
- Planned change: Parse/pass `ldap_only` to auth strategy selection; implement migration validation (`from`, `force`, `match_field`) and execution path with correct permission/license checks.
- Verification test: Login tests covering mixed auth + LDAP-only behavior; migration contract tests for invalid params, authz failures, and success path.
- Status: pending

- Rustchat target path: `backend/src/api/v4/teams.rs`, `backend/src/api/v4/channels/compat.rs`, `backend/src/api/v4/users.rs`, groups service/repository
- Required behavior: Return real MM-compatible data for group association endpoints and minus-group-members/count endpoints.
- Current gap: Multiple endpoints return empty placeholders (`backend/src/api/v4/teams.rs:939-945, 1130-1148`; `backend/src/api/v4/channels/compat.rs:421-427, 440-456, 486-493`; `backend/src/api/v4/users.rs:2954-2966`).
- Planned change: Implement data access and MM response wrappers (`users + total_count`, `groups + total_group_count`), with permission-aware filtering and pagination.
- Verification test: API regression tests asserting response shape/order/pagination and non-empty results on prepared fixtures.
- Status: pending

- Rustchat target path: `backend/src/api/v4/groups.rs` + permission layer
- Required behavior: MM-compatible permission semantics for group read/write/link operations (system, team, channel scopes and allow_reference behavior).
- Current gap: Rustchat currently centralizes many operations to system-manage style checks (`backend/src/api/v4/groups.rs:247-269, 848, 891`).
- Planned change: Align permission checks to endpoint semantics similar to MM (`sysconsole read/write groups`, team/channel manage permissions), while preserving transitional feature flags where needed.
- Verification test: permission matrix tests for system admin, team admin, channel admin, regular member across link/get/list/update/delete endpoints.
- Status: pending

- Rustchat target path: `backend/src/services/membership_policies.rs`, `backend/src/services/membership_reconciliation.rs`, groups membership integration points
- Required behavior: Autosubscription source type `group` must evaluate actual group membership (LDAP-backed and custom syncable groups) in both join-time and background reconciliation.
- Current gap: Applicability currently hard-implements only `all_users`/`auth_service`; other source types are pass-through or skipped (`backend/src/services/membership_policies.rs:435-447`; `backend/src/services/membership_reconciliation.rs:95-126`).
- Planned change: Implement source resolvers for `group` (required), then `role` and `org`; join-time path should evaluate only policies applicable to joining user and team scope; background path should resolve concrete user sets per policy.
- Verification test: policy tests for source_type=`group` with users entering/leaving groups and asserting team/channel membership add/remove + audit entries.
- Status: pending

- Rustchat target path: group sync reconciliation + group-constrained cleanup integration
- Required behavior: Link/patch/unlink and LDAP user group-sync operations must consistently drive memberships and constrained cleanup.
- Current gap: Rustchat reconcile tracks syncable-added memberships but lacks constrained membership enforcement lifecycle.
- Planned change: Keep existing tracking-table reconcile, add constrained cleanup passes when `group_constrained` toggles or syncables change, and invoke from LDAP add-user sync endpoint.
- Verification test: integration tests for: link auto-add, unlink cleanup, constrained toggle cleanup, and re-add behavior based on policy/sync settings.
- Status: pending

- Rustchat target path: `backend/src/api/v4/websocket.rs` + group mutation callsites
- Required behavior: Emit MM-compatible group websocket events so mobile/web local caches converge.
- Current gap: MM websocket mapping in Rustchat has no group event cases (`backend/src/api/v4/websocket.rs:1226-1493`).
- Planned change: Add group event mapping and publication on group create/patch/delete/member add/remove/syncable associate/dissociate with expected payload keys (`group`, `group_member`, `group_team`, `group_channel` or equivalent compatibility payload).
- Verification test: websocket integration tests asserting event names/payloads and client-visible effects for each group mutation.
- Status: pending

- Rustchat target path: compatibility test suite (`backend/tests/` + smoke scripts)
- Required behavior: End-to-end compatibility confidence for LDAP/groups/autosubscription with MM-like contracts.
- Current gap: Current tests cover syncable link/reconcile but bypass LDAP API (insert LDAP group directly) (`backend/tests/api_v4_groups_syncables.rs:80-93`) and do not validate new license/login/group-constrained websocket contracts.
- Planned change: Add contract tests per endpoint/event and extend smoke checks for: LDAP flow, group association endpoints, constrained membership, and autosubscription from group source.
- Verification test: `cargo test` suites plus `./scripts/mm_compat_smoke.sh` and targeted mobile-compat smoke for group/ws flows.
- Status: pending
