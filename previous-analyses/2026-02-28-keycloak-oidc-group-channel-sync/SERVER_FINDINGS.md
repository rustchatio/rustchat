# Server Findings

- Endpoint or component: OIDC group extraction and provisioning
- Source path: `backend/src/api/oauth.rs`
- Source lines: 1046-1083, 1125-1163, 1281-1394
- Observed behavior:
  - OIDC claims/userinfo groups are extracted (`groups_claim`) and available in `UserInfo.groups`.
  - Existing users are found by email and returned after only `last_login_at` update.
  - New users can be auto-provisioned; group mappings currently affect only initial role assignment.
- Notes:
  - No persistence of IdP groups into `groups/group_members`.
  - No per-login re-sync of roles/group-derived permissions for existing users.

- Endpoint or component: SSO-only enforcement
- Source path: `backend/src/api/auth.rs`, `backend/src/api/v4/users.rs`, `frontend/src/views/auth/LoginView.vue`
- Source lines: `auth.rs` 305-370, `users.rs` 278-325, `LoginView.vue` 21-25 and 121-123
- Observed behavior:
  - Password login endpoints still authenticate local password users.
  - Frontend hides password form when `require_sso` is true.
- Notes:
  - Enforcement is currently mostly UI-level, not strict server policy.

- Endpoint or component: Group syncables and link constraints
- Source path: `backend/src/api/v4/groups.rs`, `backend/migrations/20260228183000_groups_syncables.sql`
- Source lines: `groups.rs` 346-353, 886-903, 1186-1231; migration 3-78
- Observed behavior:
  - Group/syncable persistence exists (`groups`, `group_members`, `group_syncables`, tracking table).
  - Linking syncables requires group source `ldap` or `plugin_` prefix.
  - Public create-group endpoint only allows `custom` groups.
- Notes:
  - Keycloak-imported groups must be inserted by an internal sync service/job (not current create-group API).

- Endpoint or component: Group-association and SSO exchange compatibility endpoints
- Source path: `backend/src/api/v4/users.rs`, `backend/src/api/v4/teams.rs`, `backend/src/api/v4/channels/compat.rs`
- Source lines: `users.rs` 389-397, 2954-2966; `teams.rs` 1130-1148; `compat.rs` 486-493
- Observed behavior:
  - `/users/login/sso/code-exchange` currently returns bad request "not supported".
  - `/users/{id}/groups`, `/teams/{id}/groups`, `/teams/{id}/groups_by_channels`, `/channels/{id}/groups` currently return empty arrays.
- Notes:
  - This blocks full Mattermost mobile parity for group-aware experiences.

- Endpoint or component: Membership policy applicability engine
- Source path: `backend/src/services/membership_policies.rs`, `backend/src/services/membership_reconciliation.rs`, migrations
- Source lines: `membership_policies.rs` 403-460; `membership_reconciliation.rs` 95-126
- Observed behavior:
  - Source types include `all_users|auth_service|group|role|org` but real applicability logic currently fully handles `all_users` and partial `auth_service` only.
  - Queries reference `users.auth_service` and `users.props`.
- Notes:
  - Current migrations clearly add `auth_provider/auth_provider_id` but not `auth_service` or `users.props`; this is a compatibility/implementation mismatch for policy evaluation.

- Endpoint or component: Message permission enforcement model
- Source path: `backend/src/services/posts.rs`, `backend/src/api/v4/channels.rs`
- Source lines: `posts.rs` 31-36; `channels.rs` 698-785
- Observed behavior:
  - Posting is gated by channel membership.
  - Direct channel creation does not enforce group-based ACL beyond caller membership assumptions.
- Notes:
  - Group membership can control channel writeability indirectly (via membership sync), but DM restrictions require extra policy logic.

- Endpoint or component: Upstream Mattermost auth-service enforcement
- Source path: `../mattermost/server/channels/app/authentication.go`
- Source lines: 421-427
- Observed behavior:
  - Password login is rejected when user has non-empty auth service.
- Notes:
  - Rustchat currently does not mirror this enforcement model.

- Endpoint or component: Upstream Mattermost syncable group source semantics
- Source path: `../mattermost/server/public/model/group.go`
- Source lines: 236-246
- Observed behavior:
  - Syncable sources are LDAP and plugin-prefixed groups; custom groups are not syncable.
- Notes:
  - Rustchat mirrors this condition in `ensure_group_is_syncable`.
