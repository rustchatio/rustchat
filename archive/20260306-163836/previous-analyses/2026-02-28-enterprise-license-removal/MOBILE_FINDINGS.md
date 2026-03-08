# MOBILE_FINDINGS

## Screen/store/service: REST client license fetch
- Source path: `../mattermost-mobile/app/client/rest/general.ts`
- Source lines: 67-72
- Observed behavior: Mobile fetches `/api/v4/license/client?format=old` through `getClientLicenseOld`.
- Notes: Endpoint and old-format response are required for normal bootstrap.

## Screen/store/service: Remote bootstrap fetch
- Source path: `../mattermost-mobile/app/actions/remote/systems.ts`
- Source lines: 87-96
- Observed behavior: App startup calls `getClientConfigOld` and `getClientLicenseOld` together.
- Notes: License endpoint failures can block bootstrap.

## Screen/store/service: Login option gating
- Source path: `../mattermost-mobile/app/utils/server/index.ts`
- Source lines: 58-76
- Observed behavior: Mobile enables/disables SSO and LDAP options based on `license.IsLicensed` and per-feature license keys like `SAML`, `LDAP`, `Office365OAuth`.
- Notes: Returning permissive license keys in Rustchat is the cleanest no-license-limits strategy without mobile patches.
