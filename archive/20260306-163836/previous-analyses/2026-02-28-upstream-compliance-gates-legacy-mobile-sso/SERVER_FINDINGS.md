# Server Findings

- Endpoint or component: Group-association API license gate
- Source path: `../mattermost/server/channels/api4/group.go`
- Source lines: `getGroupsByUserId` (~794-836), `getGroupsByTeamCommon` (~920-957), `getGroupsByChannelCommon` (~959-1006), `getGroupsAssociatedToChannelsByTeam` (~1008-1063)
- Observed behavior:
  - Each path checks license + LDAPGroups feature before query execution.
  - On missing feature, API returns forbidden with LDAP-groups license error semantics.
- Notes: Team/channel/user association routes are not treated as always-on in upstream.

- Endpoint or component: Mobile SSO code-exchange feature gate
- Source path: `../mattermost/server/channels/api4/user.go`
- Source lines: `loginSSOCodeExchange` (~120-124)
- Observed behavior:
  - Endpoint immediately returns bad request when `FeatureFlags.MobileSSOCodeExchange` is disabled.
- Notes: Feature toggle is evaluated at request time.

- Endpoint or component: Legacy OAuth login entrypoints
- Source path: `../mattermost/server/channels/web/oauth.go`
- Source lines: route init (~38-45), `loginWithOAuth` (~458-493), `mobileLoginWithOAuth` (~495-523)
- Observed behavior:
  - Server exposes `/oauth/{service}/login` and `/oauth/{service}/mobile_login`.
  - Mobile route validates `redirect_to` as custom URL scheme and redirects to provider auth endpoint.
- Notes: Old route surface is preserved for client compatibility.

- Endpoint or component: Mobile OAuth completion behavior
- Source path: `../mattermost/server/channels/web/oauth.go`
- Source lines: complete flow around isMobile branch (~430-456)
- Observed behavior:
  - For mobile flow with redirect URL, server redirects to mobile custom scheme completion URL with auth payload.
- Notes: Mattermost supports both legacy token payload and newer code-exchange paths across auth mechanisms.
