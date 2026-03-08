# Server Findings

- Endpoint or component: Mobile SSO code exchange
- Source path: `../mattermost/server/channels/api4/user.go`
- Source lines: `loginSSOCodeExchange` (~120-217)
- Observed behavior:
  - Requires feature flag `MobileSSOCodeExchange`.
  - Requires request params `login_code`, `code_verifier`, `state`.
  - Consumes one-time token (`TokenTypeSSOCodeExchange`) atomically.
  - Validates token expiry, expected state match, and challenge method (`S256` supported; `PLAIN` rejected; unknown rejected).
  - Returns JSON payload containing `token` and `csrf`.
- Notes: Error path uses bad request semantics for missing/mismatch/expired code.

- Endpoint or component: `GET /api/v4/users/{user_id}/groups`
- Source path: `../mattermost/server/channels/api4/group.go`
- Source lines: `getGroupsByUserId` (~794-836)
- Observed behavior:
  - Requires self-access or system-manage permission.
  - Filters to `allow_reference=true` when caller lacks sysconsole group-read permission.
  - Returns plain array of groups.
- Notes: No pagination response envelope for this endpoint.

- Endpoint or component: `GET /api/v4/teams/{team_id}/groups`
- Source path: `../mattermost/server/channels/api4/group.go`
- Source lines: `getGroupsByTeamCommon` (~920-957)
- Observed behavior:
  - Requires team channel-list permission.
  - Applies `GroupSearchOpts` from query: `q`, `include_member_count`, `filter_allow_reference`, pagination (`page`, `per_page`, `paginate`).
  - Returns object `{groups, total_group_count}`.
- Notes: `filter_allow_reference` is forced on when caller lacks sysconsole group-read permission.

- Endpoint or component: `GET /api/v4/channels/{channel_id}/groups`
- Source path: `../mattermost/server/channels/api4/group.go`
- Source lines: `getGroupsByChannelCommon` (~959-1006)
- Observed behavior:
  - Resolves channel type then checks channel-scoped read-group permission.
  - Applies query options and pagination analogous to team groups.
  - Returns object `{groups, total_group_count}`.
- Notes: Permission differs by channel type (private vs public).

- Endpoint or component: `GET /api/v4/teams/{team_id}/groups_by_channels`
- Source path: `../mattermost/server/channels/api4/group.go`
- Source lines: `getGroupsAssociatedToChannelsByTeam` (~1008-1063)
- Observed behavior:
  - Requires team channel-list permission.
  - Applies same `GroupSearchOpts` inputs.
  - Returns object `{groups: {<channel_id>: [group...]}}`.
- Notes: Endpoint does not include `total_group_count` key.
