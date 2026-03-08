# Mobile Findings

- Screen, store, or service: REST groups client
- Source path: `../mattermost-mobile/app/client/rest/groups.ts`
- Source lines: 32-55
- Observed behavior:
  - Mobile calls `/channels/{id}/groups`, `/teams/{id}/groups`, and `/users/{id}/groups` with query params (`paginate`, `filter_allow_reference`, etc.).
- Notes:
  - Rustchat currently returns empty arrays for these association endpoints.

- Screen, store, or service: REST users client (SSO)
- Source path: `../mattermost-mobile/app/client/rest/users.ts`
- Source lines: 448-464
- Observed behavior:
  - Mobile exchanges SSO login code using `/users/login/sso/code-exchange`, expects token payload (`token`, optionally `csrf`).
- Notes:
  - Rustchat v4 endpoint currently returns "SSO code exchange is not supported".

- Screen, store, or service: Group actions
- Source path: `../mattermost-mobile/app/actions/remote/groups.ts`
- Source lines: referenced by groups REST usage in client module
- Observed behavior:
  - Group-aware mobile UX relies on non-stub group association APIs.
- Notes:
  - For strict Mattermost-mobile compatibility, server group responses must be real and permission-filtered.
