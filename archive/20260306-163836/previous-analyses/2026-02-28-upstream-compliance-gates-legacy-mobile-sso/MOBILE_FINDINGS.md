# Mobile Findings

- Screen, store, or service: SSO screen login URL construction
- Source path: `../mattermost-mobile/app/screens/sso/index.tsx`
- Source lines: switch on `ssoType` (~76-98)
- Observed behavior:
  - OAuth SSO entrypoints are hardcoded to legacy paths:
    - `/oauth/google/mobile_login`
    - `/oauth/gitlab/mobile_login`
    - `/oauth/office365/mobile_login`
    - `/oauth/openid/mobile_login`
- Notes: Route availability is required for wire compatibility.

- Screen, store, or service: SSO browser challenge parameters
- Source path: `../mattermost-mobile/app/screens/sso/sso_authentication.tsx`
- Source lines: query augmentation (~62-76)
- Observed behavior:
  - Mobile app appends `redirect_to`, `state`, `code_challenge`, `code_challenge_method` to SSO login URL.
- Notes: Server should preserve these for mobile code exchange.

- Screen, store, or service: Code-exchange REST client
- Source path: `../mattermost-mobile/app/client/rest/users.ts`
- Source lines: `exchangeSsoLoginCode` (~448-466)
- Observed behavior:
  - Mobile calls `/api/v4/users/login/sso/code-exchange` with `{login_code, code_verifier, state}` and expects `{token, csrf}`.
- Notes: Failure semantics should remain bad-request class for feature-off/invalid cases.
