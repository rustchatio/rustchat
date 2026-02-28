# SERVER_FINDINGS

## Endpoint/component: LDAP API license gating in MM server handlers
- Source path: `../mattermost/server/channels/api4/ldap.go`
- Source lines: 49-51, 69-71, 88-90, 113-115, 158-160
- Observed behavior: LDAP endpoints return `http.StatusNotImplemented` with error id `api.ldap_groups.license_error` when license flags are missing.
- Notes: This is MM licensing behavior; Rustchat can intentionally diverge for no-license mode, but must keep endpoint/status compatibility where possible.

## Endpoint/component: Client license old-format payload
- Source path: `../mattermost/server/channels/utils/license.go`
- Source lines: 157-203
- Observed behavior: `GetClientLicense` returns map with `IsLicensed` plus feature flags (`LDAP`, `LDAPGroups`, `SAML`, etc.) as string booleans.
- Notes: Mobile/web clients rely on these keys existing in old-format payload.

## Endpoint/component: OpenAPI contract for LDAP endpoints
- Source path: `../mattermost/api/v4/source/ldap.yaml`
- Source lines: 1-22, 23-46, 47-83, 84-135
- Observed behavior: LDAP endpoints document `501 NotImplemented` among expected responses.
- Notes: Keeping `501` for unimplemented internals is contract-safe.

## Endpoint/component: OpenAPI contract for SAML endpoints
- Source path: `../mattermost/api/v4/source/saml.yaml`
- Source lines: 1-19, 20-47, 48-88, 89-114, 115-155, 156-181
- Observed behavior: SAML endpoints also document `501 NotImplemented` responses.
- Notes: `501` itself is compatible; enterprise-license-specific messaging is not required by client parsing.

## Endpoint/component: System license client route in OpenAPI
- Source path: `../mattermost/api/v4/source/system.yaml`
- Source lines: 595-639
- Observed behavior: `/api/v4/license/client` exists and supports `format=old`; returns `200` (and may return `501` for other formats).
- Notes: Rustchat should keep this route and old-format payload for clients.
