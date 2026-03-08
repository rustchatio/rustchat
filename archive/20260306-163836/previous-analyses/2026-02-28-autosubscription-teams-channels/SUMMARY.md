# Summary

- Topic: Auto-subscription of new users to teams and channels, admin-console configuration model, and implementation plan for rustchat.
- Date: 2026-02-28
- Scope:
  - Mattermost server contracts for default channel join and group-sync membership creation.
  - Mattermost admin-console surfaces for configuring team/channel membership defaults.
  - Mattermost mobile expectations related to default channel availability.
  - rustchat current implementation and compatibility gaps.
  - External benchmark patterns (Zulip + other platforms).

## Compatibility contract (evidence-backed)

### 1) Mattermost has two distinct auto-subscription mechanisms
- Mechanism A: Team default channels (global config-driven)
  - `JoinDefaultChannels` uses `DefaultChannelNames()` and auto-adds users to those channels on team join.
  - `DefaultChannelNames()` always includes `town-square`; if `TeamSettings.ExperimentalDefaultChannels` is empty, `off-topic` is also included.
  - Evidence:
    - `../mattermost/server/channels/app/channel.go:41-57`
    - `../mattermost/server/channels/app/channel.go:59-129`
    - `../mattermost/server/public/model/config.go:2430`
- Mechanism B: Group syncables (LDAP/plugin groups linked to teams/channels)
  - Linking a group to a team/channel triggers async role+membership synchronization.
  - Manual per-user re-sync endpoint exists to create missing memberships from LDAP group mapping.
  - Evidence:
    - `../mattermost/server/channels/api4/group.go:390-392`
    - `../mattermost/server/channels/app/syncables.go:273-310`
    - `../mattermost/server/channels/api4/ldap.go:484-515`

### 2) Admin console in Mattermost is group-sync centric for team/channel default membership
- Group detail page explicitly says: configure default teams/channels for group members.
- Team and channel membership UI is exposed from group settings.
- User list action includes "Re-sync user via LDAP groups" (permission + auth-service gated).
- Evidence:
  - `../mattermost/webapp/channels/src/components/admin_console/group_settings/group_details/group_details.tsx:665-683`
  - `../mattermost/webapp/channels/src/components/admin_console/group_settings/group_details/group_details.tsx:724-733`
  - `../mattermost/webapp/channels/src/components/admin_console/system_users/system_users_list_actions/index.tsx:534-545`
  - `../mattermost/webapp/channels/src/components/admin_console/system_users/system_users_list_actions/create_group_syncables_membership_modal.tsx:24-29`

### 3) Group syncability is intentionally restricted
- Only LDAP and plugin-prefixed groups are syncable.
- Custom groups are valid groups but not syncable for team/channel auto-membership.
- Evidence:
  - `../mattermost/server/public/model/group.go:233-245`

### 4) Default-channel auto-join is soft-fail in Mattermost join flow
- Team membership succeeds even if default-channel auto-join fails (warning only).
- This creates eventual divergence risk between "team member" and "expected default channel member".
- Evidence:
  - `../mattermost/server/channels/app/team.go:815-824`

### 5) Mattermost mobile expects default channel availability during initial channel selection
- Mobile entry logic checks membership in the default channel for initial navigation.
- If unavailable, falls back to first available public channel.
- Evidence:
  - `../mattermost-mobile/app/actions/remote/entry/common.ts:278-291`
  - `../mattermost-mobile/app/queries/servers/channel.ts:355-390`
  - `../mattermost-mobile/app/utils/channel/index.ts:162-164`

## Mattermost design risks (from code + docs)

- Global default-channel list is system-wide (`TeamSettings.ExperimentalDefaultChannels`), not per-team/per-segment.
- Default-channel join errors are logged and ignored in team-join path.
- Group-based default membership is tied to syncable groups (LDAP/plugin); no equivalent syncability for custom groups.
- Link/patch/unlink sync flows run async (`Srv().Go(...)`), so UI-level "saved" and membership convergence are not atomic.

## rustchat current state (high-level)

- `api/v1` and admin add-team-member flows currently auto-add users to **all public channels** of a team:
  - `backend/src/api/teams.rs:202-214`
  - `backend/src/api/teams.rs:349-361`
  - `backend/src/api/admin.rs:1592-1604`
- `api/v4` add-team-member does **not** auto-add any channels:
  - `backend/src/api/v4/teams.rs:269-301`
- Team creation does not create Mattermost-style default channels (no `town-square`/`off-topic` bootstrap in create-team path):
  - `backend/src/api/teams.rs:50-113`
  - Test note confirming this risk: `backend/tests/api_integrations.rs:79-83`
- `api/v4` LDAP/group routes for this area are largely placeholders/stubs:
  - LDAP route exposes `GET` for `group_sync_memberships` instead of Mattermost `POST`: `backend/src/api/v4/ldap.rs:42-44`
  - LDAP handlers return enterprise-needed `501`: `backend/src/api/v4/ldap.rs:11-22`
  - Groups endpoints return empty/stub payloads: `backend/src/api/v4/groups.rs:45-200`

## External benchmark patterns (official docs)

- Zulip:
  - Organization-wide default channels for new users are first-class admin settings.
  - Docs: https://zulip.com/help/set-default-channels-for-new-users
- Slack:
  - Default channels can be configured for members joining a workspace.
  - User groups can be subscribed to channels.
  - Docs: https://slack.com/help/articles/201898998-Set-default-channels-for-new-members
  - Docs: https://slack.com/help/articles/212906697-Create-a-user-group
- Rocket.Chat:
  - Explicit "Default Channels" admin setting with behavior for registration/manual create/LDAP import.
  - Docs: https://docs.rocket.chat/docs/channels
- Microsoft Teams:
  - Dynamic membership is strongly supported at team level via Entra groups; channel-level defaults are less policy-centric.
  - Docs: https://learn.microsoft.com/en-us/microsoftteams/dynamic-memberships

## Open questions

- Should rustchat target strict Mattermost parity first (default channels + LDAP/plugin group sync), then add a richer policy engine?
- For OSS-only deployments without LDAP, should rustchat expose a non-enterprise admin policy UI for auto-subscription rules?
