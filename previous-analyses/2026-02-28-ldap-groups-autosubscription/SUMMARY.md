# Summary

- Topic: LDAP + Groups compatibility and autosubscription architecture for Rustchat
- Date: 2026-02-28
- Scope: Mattermost server (`../mattermost`) and mobile (`../mattermost-mobile`) contracts for LDAP, LDAP-groups syncables, group-constrained membership, websocket events, and user/team/channel group endpoints; compared to current Rustchat backend implementation.
- Compatibility contract:
  - LDAP endpoints must be real (not static 501), with license/permission semantics aligned to MM: LDAP features and LDAPGroups feature gates, `migrate_auth/ldap` validation, and user-to-group-syncables membership sync.
  - Group-sync endpoints must support full CRUD/link lifecycle with MM response shapes/statuses, then trigger asynchronous role/membership sync and unlink cleanup.
  - Team/channel/user association endpoints (`/teams/{id}/groups`, `/channels/{id}/groups`, `/users/{id}/groups`, `/teams/{id}/groups_by_channels`) and minus-group-members/count endpoints must return MM-compatible payloads, not placeholders.
  - Team/channel models and patch contracts must include `group_constrained`; backend must enforce constrained membership rules and cleanup.
  - Websocket must emit MM group events/payloads (`received_group`, group member events, team/channel association events) for mobile/web cache consistency.
  - Config/license client payloads consumed by mobile must include LDAP-related keys (`EnableLdap`, `LDAP`, `LDAPGroups`) with MM-compatible types.
  - Autosubscription must integrate true group-based applicability (not only `all_users`/`auth_service`) and consume synchronized LDAP/group memberships.
- Open questions:
  - MM server constant currently includes `group_member_deleted` while the inspected mobile constants include `group_member_delete`; Rustchat should decide target contract by pinned client version and may need alias compatibility.
  - Whether Rustchat should preserve current simplified permission model short-term or fully mirror MM granular permission checks in first implementation phase.
  - Whether LDAP features remain license-gated in Rustchat product strategy or are always enabled; this affects exact `license/client` contract behavior.
