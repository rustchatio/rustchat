# Summary

- Topic: Upstream parity for mobile SSO code exchange and group association endpoints (`/users/{id}/groups`, `/teams/{id}/groups`, `/teams/{id}/groups_by_channels`, `/channels/{id}/groups`).
- Date: 2026-02-28
- Scope: API v4 wire-contract alignment with Mattermost server + mattermost-mobile request expectations, limited to auth code-exchange validation and group endpoint permissions/query semantics.
- Compatibility contract:
  - `/api/v4/users/login/sso/code-exchange` requires `login_code`, `code_verifier`, and `state`; validates state and PKCE-style challenge before issuing session token payload `{token, csrf}`.
  - Team/channel group association endpoints enforce permission gating and support query options (`q`, `include_member_count`, `filter_allow_reference`, `page`, `per_page`, `paginate`) with MM-compatible response envelopes.
  - `/users/{id}/groups` enforces self-or-system permission and filters non-referenceable groups for non-system readers.
- Open questions:
  - Rustchat does not yet implement Mattermost license-feature gate (`LDAPGroups`) for these routes; this iteration aligns runtime behavior but not enterprise licensing semantics.
  - Rustchat currently uses `rustchat://oauth/complete` callback path for mobile provider flow; this iteration preserves compatibility by carrying `login_code` and challenge fields, but not full legacy `/oauth/*/mobile_login` route surface.
