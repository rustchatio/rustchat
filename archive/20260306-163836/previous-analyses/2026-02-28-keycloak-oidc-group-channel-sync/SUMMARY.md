# Summary

- Topic: Keycloak OIDC login with Keycloak-authoritative user/group sync, channel auto-subscriptions, and permission propagation in Rustchat.
- Date: 2026-02-28
- Scope: Rustchat (`backend/`, `frontend/`) plus Mattermost server/mobile reference behavior in `../mattermost` and `../mattermost-mobile` for SSO, group syncables, and group-constrained memberships.
- Compatibility contract:
  - Authentication can be OIDC-first, but server-side login enforcement must reject local password login when SSO-only mode is configured.
  - Group-driven autosubscription must use persisted groups + syncables and deterministic reconciliation (add/remove) based on identity-source truth.
  - Syncable groups should follow Mattermost-compatible source semantics (`ldap` or `plugin_` prefix) when linking groups to teams/channels.
  - Mattermost-compatible endpoints used by mobile (`/users/{id}/groups`, `/teams/{id}/groups`, `/channels/{id}/groups`, `/users/login/sso/code-exchange`) must be functional, not stubs, if Mattermost mobile compatibility is required.
- Open questions:
  - Should Keycloak group membership be enforced only at login (JIT) or continuously (background pull / event-driven sync)?
  - Should direct messages be group-constrained? Current Rustchat direct-channel creation does not enforce group-based ACL.
  - Should strict SSO-only mode include a break-glass local admin path?
