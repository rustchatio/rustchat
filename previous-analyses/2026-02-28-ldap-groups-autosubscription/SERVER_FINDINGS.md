# Server Findings

- Endpoint or component: LDAP route surface and feature gating
- Source path: `../mattermost/server/channels/api4/ldap.go`
- Source lines: 22-46, 48-51, 69-71, 87-90, 112-115, 158-160
- Observed behavior: Mattermost exposes full LDAP endpoints (`/sync`, `/test`, `/test_connection`, `/test_diagnostics`, `/groups`, `/groups/{remote_id}/link`, `/users/{user_id}/group_sync_memberships`) and gates LDAP with license checks (`LDAP` and `LDAPGroups`). Disabled features return `http.StatusNotImplemented`.
- Notes: Rustchat currently exposes similar paths but every LDAP handler returns static 501 enterprise-required JSON (`backend/src/api/v4/ldap.rs:11-21, 47-144`).

- Endpoint or component: LDAP groups list response contract
- Source path: `../mattermost/server/channels/api4/ldap.go`
- Source lines: 152-204
- Observed behavior: `GET /api/v4/ldap/groups` returns JSON object `{ "count": number, "groups": [...] }`, where each group item can include `mattermost_group_id`, `name`, `primary_key`, and `has_syncables`.
- Notes: Rustchat `GET /api/v4/ldap/groups` is currently stubbed 501 (`backend/src/api/v4/ldap.rs:79-85`).

- Endpoint or component: LDAP group link/unlink semantics
- Source path: `../mattermost/server/channels/api4/ldap.go`
- Source lines: 206-306, 308-347
- Observed behavior: `POST /ldap/groups/{remote_id}/link` resolves LDAP group, creates/undeletes mapped MM group, returns `201` for create and `200` for already-existing/undeleted. `DELETE /ldap/groups/{remote_id}/link` soft-deletes the mapped group and returns status OK.
- Notes: Rustchat only defines POST link route; DELETE unlink route is missing (`backend/src/api/v4/ldap.rs:31, 87-94`).

- Endpoint or component: Add user to group syncables behavior
- Source path: `../mattermost/server/channels/api4/ldap.go`
- Source lines: 484-515, 498-507
- Observed behavior: `POST /ldap/users/{user_id}/group_sync_memberships` requires LDAP-backed user (or SAML with sync-with-ldap) and executes `CreateDefaultMemberships` scoped to that user.
- Notes: Rustchat route exists but is stubbed 501 (`backend/src/api/v4/ldap.rs:42-44, 137-144`).

- Endpoint or component: Groups + syncables API contracts
- Source path: `../mattermost/server/channels/api4/group.go`
- Source lines: 22-105, 375-395, 590-644
- Observed behavior: Full route surface for groups and syncables exists. Link returns `201`, patch/get/list return `200`, unlink returns status OK.
- Notes: Rustchat has broad groups/syncables implementation and persistence (`backend/src/api/v4/groups.rs:160-193`, `backend/migrations/20260228183000_groups_syncables.sql:3-78`).

- Endpoint or component: Link/patch side effects and unlink cleanup
- Source path: `../mattermost/server/channels/api4/group.go`, `../mattermost/server/channels/app/syncables.go`
- Source lines: group.go 390-392, 575-577, 637-639; syncables.go 271-310, 313-324
- Observed behavior: Link/patch trigger async `SyncRolesAndMembership`; unlink triggers async `RemoveMembershipsFromUnlinkedSyncable`, which also enforces group-constrained cleanup.
- Notes: Rustchat also runs async reconcile/cleanup (`backend/src/api/v4/groups.rs:527-544, 546-575, 1225, 1268-1282, 1394`) but does not integrate group-constrained semantics.

- Endpoint or component: Group-constrained team/channel enforcement
- Source path: `../mattermost/server/channels/api4/team.go`, `../mattermost/server/channels/api4/channel.go`, `../mattermost/server/channels/app/syncables.go`
- Source lines: team.go 341-347, 1090-1092; channel.go 396-401, 2103-2105; syncables.go 142-218
- Observed behavior: When team/channel becomes `group_constrained=true`, MM asynchronously removes non-allowed members. Removing other users from constrained team/channel is restricted with specific app errors.
- Notes: Rustchat models and compatibility structs do not include `group_constrained` fields (`backend/src/models/team.rs:10-23`, `backend/src/models/channel.rs:21-35`, `backend/src/mattermost_compat/models.rs:29-63`).

- Endpoint or component: Members-minus-group-members and counts endpoints
- Source path: `../mattermost/server/channels/api4/team.go`, `../mattermost/server/channels/api4/channel.go`, `../mattermost/server/public/model/user.go`
- Source lines: team.go 1942-1992; channel.go 2199-2240, 2251-2276; user.go 1101-1105
- Observed behavior: Team/channel minus-group endpoints return `UsersWithGroupsAndCount` (`users` + `total_count`). Channel member counts by group returns computed aggregate list.
- Notes: Rustchat currently returns empty arrays for these endpoints (`backend/src/api/v4/teams.rs:939-945`, `backend/src/api/v4/channels/compat.rs:440-456`).

- Endpoint or component: Team/channel/user groups association endpoints
- Source path: `../mattermost/server/channels/api4/group.go`
- Source lines: 70-88, 794-836, 899-957, 959-1006, 1008-1059
- Observed behavior: MM serves `/users/{id}/groups`, `/teams/{id}/groups`, `/channels/{id}/groups`, and `/teams/{id}/groups_by_channels` with license/permission-aware filtering and response wrappers.
- Notes: Rustchat stubs these responses as empty arrays in teams/channels/users routes (`backend/src/api/v4/teams.rs:1130-1148`, `backend/src/api/v4/channels/compat.rs:486-493`, `backend/src/api/v4/users.rs:2954-2966`).

- Endpoint or component: Login LDAP-only and auth migration contracts
- Source path: `../mattermost/server/channels/api4/user.go`
- Source lines: 2016-2023, 2051-2052, 3417-3474
- Observed behavior: Login accepts `ldap_only` and passes it to auth flow. `/users/migrate_auth/ldap` validates `from`, `force`, `match_field`, requires `manage_system`, checks LDAP license, and runs migration.
- Notes: Rustchat login request struct does not include `ldap_only` (`backend/src/api/v4/users.rs:225-232`), and `/users/migrate_auth/ldap` is placeholder status OK (`backend/src/api/v4/users.rs:2923-2926`).

- Endpoint or component: License and model contract for LDAP/LDAPGroups + group_constrained
- Source path: `../mattermost/server/public/model/license.go`, `../mattermost/server/public/model/team.go`, `../mattermost/server/public/model/channel.go`
- Source lines: license.go 157-161, 194-198; team.go 42-43, 73; channel.go 96, 153
- Observed behavior: MM models expose `Features.LDAP`, `Features.LDAPGroups`, and `group_constrained` in team/channel + patch payloads.
- Notes: Rustchat client-license endpoint currently returns unlicensed minimal payload and compatibility license model has no LDAP/LDAPGroups feature fields (`backend/src/api/v4/config_client.rs:82-100`, `backend/src/mattermost_compat/models.rs:164-173`).

- Endpoint or component: Group websocket contracts
- Source path: `../mattermost/server/channels/app/group.go`, `../mattermost/server/public/model/websocket_message.go`
- Source lines: app/group.go 347-349, 366-368, 456-463, 557-565, 875-883; websocket_message.go 63-69
- Observed behavior: MM publishes `received_group`, association/dissociation events for team/channel, and group member add/delete events with serialized payload keys (`group`, `group_team`/`group_channel` via `group_id`+broadcast scope, `group_member`).
- Notes: Rustchat websocket mapper currently covers posts/reactions/status/member events but not group events (`backend/src/api/v4/websocket.rs:1226-1493`), and there are no matching group event identifiers found in Rustchat source.
