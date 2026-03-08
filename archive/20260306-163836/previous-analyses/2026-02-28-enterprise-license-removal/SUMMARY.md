# SUMMARY

- Topic: Remove enterprise-license limitation definitions from Rustchat while preserving client wire compatibility.
- Date: 2026-02-28
- Scope: Server API (`/api/v4/license/client`, `/api/v4/ldap/*`, `/api/v4/saml/*`, `/api/v4/system/support_packet`) and mobile compatibility impact from license payload usage.

## Compatibility contract (observed)

- Mattermost mobile always fetches `/api/v4/license/client?format=old` via `getClientLicenseOld` and uses string fields like `IsLicensed`, `SAML`, `LDAP` for login option decisions.
- Mattermost server OpenAPI and handlers show LDAP/SAML endpoints may return `501 Not Implemented`; in MM this is often due license/config status.
- `IsLicensed` old-format value is a string (`"true"|"false"`) in MM server (`GetClientLicense`).

## Rustchat decision for this iteration

- Keep license endpoint shape and keys for compatibility, but remove Rustchat-owned enterprise gating definitions and toggles.
- Do not block LDAP/SAML/group APIs with enterprise-license-specific error IDs/messages.
- Keep `501` stubs where implementation is not present, but with neutral non-license wording.

## Open questions

- None blocking for this task.
