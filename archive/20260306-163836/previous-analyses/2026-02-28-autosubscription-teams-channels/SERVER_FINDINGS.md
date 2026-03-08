# Server Findings

## Mattermost (reference)

- Endpoint or component: Default-channel name resolution.
- Source path: `../mattermost/server/channels/app/channel.go`
- Source lines: `41-57`
- Observed behavior: Default list always starts with `town-square`; `off-topic` is default only when `TeamSettings.ExperimentalDefaultChannels` is empty; deduplication is applied.
- Notes: This is global config behavior, not per-team policy behavior.

- Endpoint or component: Auto-join implementation for default channels.
- Source path: `../mattermost/server/channels/app/channel.go`
- Source lines: `59-129`
- Observed behavior: For each default channel name, it fetches by team + name, requires open channel type, inserts `ChannelMember`, logs channel history, optionally posts join messages, and emits websocket `user_added`.
- Notes: Non-existent default channels are skipped with warning.

- Endpoint or component: Team join flow auto-applies default channels for non-guests.
- Source path: `../mattermost/server/channels/app/team.go`
- Source lines: `773-850` (focus: `815-824`)
- Observed behavior: `JoinUserToTeam` calls `JoinDefaultChannels` for non-guest users, but treats failure as soft warning.
- Notes: Membership to team can succeed while default channel membership partially fails.

- Endpoint or component: Invite/token-based user creation + default channel effects.
- Source path: `../mattermost/server/channels/app/user.go`
- Source lines: `39-122`, `215-258`
- Observed behavior: Both token-based and invite-id flows call `JoinUserToTeam`; this triggers default channel joining via team join path.
- Notes: Additional invited/private channel memberships are layered after team join.

- Endpoint or component: Group syncable linking triggers membership sync.
- Source path: `../mattermost/server/channels/api4/group.go`
- Source lines: `375-394`, `566-577`, `637-639`
- Observed behavior: Link/patch/unlink of group syncables schedules async sync or cleanup via goroutine.
- Notes: Observable eventual-consistency model.

- Endpoint or component: Group sync membership creation internals.
- Source path: `../mattermost/server/channels/app/syncables.go`
- Source lines: `22-84`, `91-120`, `128-139`, `273-310`
- Observed behavior: Computes memberships-to-add from group syncables, ensures team before channel membership, and supports re-add behavior; can be scoped to syncable/user.
- Notes: LDAP uses config-driven `ReAddRemovedMembers` behavior during sync.

- Endpoint or component: Manual user resync via LDAP group mappings.
- Source path: `../mattermost/server/channels/api4/ldap.go`
- Source lines: `45`, `484-515`
- Observed behavior: `POST /api/v4/ldap/users/{user_id}/group_sync_memberships` requires `sysconsole_write_user_management_groups`; only LDAP or SAML+LDAP users allowed; invokes `CreateDefaultMemberships` with `ReAddRemovedMembers: true` and scoped user.
- Notes: Explicit admin-triggered repair/reconciliation hook.

- Endpoint or component: Web client route for LDAP user resync.
- Source path: `../mattermost/webapp/platform/client/src/client4.ts`
- Source lines: `4161-4165`
- Observed behavior: `createGroupTeamsAndChannels(userID)` posts to `/ldap/users/{user_id}/group_sync_memberships`.
- Notes: Confirms POST semantics from client.

- Endpoint or component: Group syncability constraints.
- Source path: `../mattermost/server/public/model/group.go`
- Source lines: `233-245`
- Observed behavior: `IsSyncable()` returns true for LDAP and plugin groups; not for custom groups.
- Notes: Custom groups cannot be used for team/channel auto-membership syncable contracts.

- Endpoint or component: Config surface for experimental default channels.
- Source path: `../mattermost/server/public/model/config.go`
- Source lines: `2430`, `2510-2512`
- Observed behavior: `TeamSettings.ExperimentalDefaultChannels` exists as a config field and defaults to empty list.
- Notes: This field drives mechanism A in `DefaultChannelNames`.

## rustchat (current)

- Endpoint or component: `api/v1` add-team-member behavior.
- Source path: `backend/src/api/teams.rs`
- Source lines: `167-217`
- Observed behavior: Adding a team member also inserts the user into all public channels in that team.
- Notes: This differs from Mattermost default-channel contract (selective defaults, not all public).

- Endpoint or component: `api/v1` join-team behavior.
- Source path: `backend/src/api/teams.rs`
- Source lines: `305-364`
- Observed behavior: Joining a public team also inserts user into all public channels.
- Notes: Same mismatch risk as above.

- Endpoint or component: Admin add-team-member behavior.
- Source path: `backend/src/api/admin.rs`
- Source lines: `1571-1607`
- Observed behavior: Admin action also bulk-adds to all public channels.
- Notes: Creates behavior coupling that is not expressed as policy.

- Endpoint or component: `api/v4` add-team-member behavior.
- Source path: `backend/src/api/v4/teams.rs`
- Source lines: `269-301`
- Observed behavior: Adds only `team_members`; does not add channel memberships.
- Notes: Internal API inconsistency between `v1/admin` and `v4`.

- Endpoint or component: Team creation bootstrap behavior.
- Source path: `backend/src/api/teams.rs`
- Source lines: `50-113`
- Observed behavior: Team creation inserts team and creator membership; no default channel creation path.
- Notes: Test comments acknowledge default-channel absence risk (`backend/tests/api_integrations.rs:79-83`).

- Endpoint or component: LDAP group sync memberships route contract.
- Source path: `backend/src/api/v4/ldap.rs`
- Source lines: `42-44`, `137-144`
- Observed behavior: Route is `GET /ldap/users/{user_id}/group_sync_memberships`; handler returns enterprise-required `501`.
- Notes: Mattermost contract for this path is `POST` with action semantics.

- Endpoint or component: Groups syncable API surface.
- Source path: `backend/src/api/v4/groups.rs`
- Source lines: `10-200`
- Observed behavior: Routes exist but return empty or placeholder payloads; no membership orchestration.
- Notes: Admin-console and compatibility clients will not receive real sync behavior.

- Endpoint or component: Admin UI capabilities around team/channel membership automation.
- Source path: `frontend/src/views/admin/TeamsManagement.vue`
- Source lines: `75-108`, `194-235`, `256-272`
- Observed behavior: UI provides manual team details, channel CRUD, and manual member add/remove.
- Notes: No policy editor for default team/channel auto-subscription rules.
