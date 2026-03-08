# Mobile Findings

- Screen, store, or service: Login form LDAP enable gate
- Source path: `../mattermost-mobile/app/screens/login/form.tsx`
- Source lines: 128-131
- Observed behavior: Mobile enables LDAP login UX only when all are true: `license.IsLicensed === 'true'`, `config.EnableLdap === 'true'`, and `license.LDAP === 'true'`.
- Notes: Rustchat currently exposes `EnableLdap` config key (`backend/src/api/v4/config_client.rs:194`) but client license payload lacks LDAP feature flags (`backend/src/api/v4/config_client.rs:82-100`, `backend/src/mattermost_compat/models.rs:164-173`).

- Screen, store, or service: Login REST client payload
- Source path: `../mattermost-mobile/app/client/rest/users.ts`
- Source lines: 122-132
- Observed behavior: `login(..., ldapOnly=true)` sends `ldap_only: 'true'` in `/users/login` request body.
- Notes: Rustchat login request struct does not include `ldap_only` (`backend/src/api/v4/users.rs:225-232`), so LDAP-only login behavior is not implemented.

- Screen, store, or service: Group-constrained user search queries
- Source path: `../mattermost-mobile/app/client/rest/users.ts`
- Source lines: 250-254, 284-288
- Observed behavior: `getProfilesNotInTeam` and `getProfilesNotInChannel` append `group_constrained=true` when constrained filtering is needed.
- Notes: Rustchat needs matching backend query semantics for compatibility-sensitive invite/member selection flows.

- Screen, store, or service: Groups REST dependencies
- Source path: `../mattermost-mobile/app/client/rest/groups.ts`
- Source lines: 18-29, 32-39, 43-53
- Observed behavior: Mobile calls `/groups`, `/channels/{id}/groups`, `/teams/{id}/groups`, `/users/{id}/groups` and expects MM-style wrappers for team/channel group lists.
- Notes: Rustchat currently stubs `teams/{id}/groups`, `channels/{id}/groups`, and `users/{id}/groups` to empty arrays (`backend/src/api/v4/teams.rs:1130-1138`, `backend/src/api/v4/channels/compat.rs:486-493`, `backend/src/api/v4/users.rs:2954-2966`).

- Screen, store, or service: Group fetch behavior and constrained fetches
- Source path: `../mattermost-mobile/app/actions/remote/groups.ts`
- Source lines: 69-99, 101-130, 132-161, 179-201
- Observed behavior: Mobile syncs groups for channel/team/member and conditionally fetches group associations when `team.isGroupConstrained` or `channel.isGroupConstrained` is true.
- Notes: Without `group_constrained` propagation in team/channel payloads, constrained fetch path will not trigger correctly.

- Screen, store, or service: Team/channel typing contracts
- Source path: `../mattermost-mobile/types/api/teams.d.ts`, `../mattermost-mobile/types/api/channels.d.ts`
- Source lines: teams.d.ts 44; channels.d.ts 45, 58
- Observed behavior: API contracts include nullable `group_constrained` on both Team and Channel and in channel patch.
- Notes: Rustchat compatibility models currently omit this field (`backend/src/mattermost_compat/models.rs:29-63`).

- Screen, store, or service: License/config typing contracts
- Source path: `../mattermost-mobile/types/api/config.d.ts`, `../mattermost-mobile/types/api/license.d.ts`
- Source lines: config.d.ts 75; license.d.ts 20-21
- Observed behavior: Mobile contract requires `EnableLdap` in config and both `LDAP` and `LDAPGroups` in license object.
- Notes: Rustchat client license currently returns minimal fields and does not provide these feature keys.

- Screen, store, or service: Group websocket event names
- Source path: `../mattermost-mobile/app/constants/websocket.ts`, `../mattermost-mobile/app/actions/websocket/event.ts`, `../mattermost-mobile/app/actions/websocket/group.ts`
- Source lines: websocket.ts 90-96; event.ts 255-269; group.ts 10-24, 44-149
- Observed behavior: Mobile registers group websocket event constants and handlers expecting payload keys like `group`, `group_member`, `group_team`, `group_channel`.
- Notes: Rustchat websocket mapper currently has no group event mapping (`backend/src/api/v4/websocket.rs:1226-1493`).
