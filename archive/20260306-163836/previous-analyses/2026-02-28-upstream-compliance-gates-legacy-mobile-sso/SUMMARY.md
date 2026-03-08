# Summary

- Topic: Remaining upstream compatibility hardening for license/feature gates and legacy mobile SSO routes.
- Date: 2026-02-28
- Scope: API v4 group-association gating semantics, mobile SSO code-exchange feature flag behavior, and web compatibility aliases for `/oauth/{service}/mobile_login`.
- Compatibility contract:
  - Group-association endpoints (`/users/{id}/groups`, `/teams/{id}/groups`, `/teams/{id}/groups_by_channels`, `/channels/{id}/groups`) return `403` when LDAP-groups enterprise feature is unavailable.
  - `/api/v4/users/login/sso/code-exchange` returns bad request when mobile SSO code exchange feature flag is disabled.
  - Legacy mobile OAuth entrypoints (`/oauth/{service}/mobile_login`, `/oauth/{service}/login`) remain available and redirect into OAuth provider auth flow.
  - Mobile OAuth `redirect_to` is validated against configured `site.app_custom_url_schemes` (default `mmauth://`, `mmauthbeta://`) and strict `://callback` shape.
- Open questions:
  - Rustchat currently models licensing via runtime config, not signed license files; this iteration implements behavior parity controls, not full Mattermost license key semantics.
  - Legacy OAuth `service` names (`gitlab`, `office365`, `openid`) are mapped to active configured providers using deterministic resolution rules; ambiguity handling is documented in GAP_PLAN.
