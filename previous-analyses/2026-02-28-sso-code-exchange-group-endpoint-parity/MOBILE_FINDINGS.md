# Mobile Findings

- Screen, store, or service: REST users client (code exchange)
- Source path: `../mattermost-mobile/app/client/rest/users.ts`
- Source lines: `exchangeSsoLoginCode` (~448-466)
- Observed behavior:
  - Calls `POST /api/v4/users/login/sso/code-exchange` with body `{login_code, code_verifier, state}`.
  - Expects response shape `{token, csrf}`.
- Notes: Used by session SSO flow after callback receives `login_code`.

- Screen, store, or service: SSO screen challenge generation
- Source path: `../mattermost-mobile/app/utils/saml_challenge.ts`
- Source lines: `createSamlChallenge` (~45-56)
- Observed behavior:
  - Generates random `state`, `codeVerifier`, and S256 `codeChallenge`.
- Notes: Challenge attached to SSO login URL and later replayed to code-exchange API.

- Screen, store, or service: Group REST client
- Source path: `../mattermost-mobile/app/client/rest/groups.ts`
- Source lines: `getAllGroupsAssociatedToChannel` / `getAllGroupsAssociatedToTeam` / `getAllGroupsAssociatedToMembership` (~31-55)
- Observed behavior:
  - Channel groups call sends `paginate=false`, `filter_allow_reference`, `include_member_count=true`.
  - Team groups call sends `paginate=false`, `filter_allow_reference`.
  - User groups call sends `paginate=false`, `filter_allow_reference`.
- Notes: Mobile expects response envelopes for team/channel and plain array for user groups.

- Screen, store, or service: Remote group actions
- Source path: `../mattermost-mobile/app/actions/remote/groups.ts`
- Source lines: `fetchGroupsForChannel` / `fetchGroupsForTeam` / `fetchGroupsForMember` (~62-156)
- Observed behavior:
  - Persists fetched group associations into local DB.
  - Uses license-aware guards but still depends on endpoint schema + permission consistency.
- Notes: Contract regressions here surface as missing mentions/autocomplete group data.
