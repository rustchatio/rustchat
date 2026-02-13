# API v4 Compatibility Report

Date: 2026-02-07

## Scope and goal
This report audits `/api/v4` handlers for silent stub behavior and tracks high-priority endpoints used by Mattermost mobile/desktop clients.

Primary objective: no silent placeholder behavior on prioritized endpoints; each must be either:
- implemented with real behavior, or
- explicit `501 Not Implemented` with Mattermost-style payload (`id`, `message`, `detailed_error`, `request_id`, `status_code`).

## Prioritization method
Priority was derived from:
- `docs/mattermost_compat/minimum_mobile_endpoints.md`
- `tools/mm-compat/output/endpoints_final.json`
- static endpoint references in `mattermost-mobile` (startup, sync, websocket, posting, plugin-related checks)

## Audit: endpoints with stub/placeholder/hardcoded behavior
The following handlers currently return hardcoded/placeholder data (or no-op success) rather than full feature behavior.

### Enterprise/admin surface (mostly non-client-critical)
- `GET /api/v4/access_control_policies`
- `POST /api/v4/access_control_policies/cel/check`
- `POST /api/v4/access_control_policies/cel/validate_requester`
- `POST /api/v4/access_control_policies/cel/test`
- `POST /api/v4/access_control_policies/search`
- `GET /api/v4/access_control_policies/cel/autocomplete/fields`
- `GET /api/v4/access_control_policies/{policy_id}`
- `POST /api/v4/access_control_policies/{policy_id}/activate`
- `POST /api/v4/access_control_policies/{policy_id}/assign`
- `POST /api/v4/access_control_policies/{policy_id}/unassign`
- `GET /api/v4/access_control_policies/{policy_id}/resources/channels`
- `POST /api/v4/access_control_policies/{policy_id}/resources/channels/search`
- `GET /api/v4/access_control_policies/cel/visual_ast`
- `POST /api/v4/access_control_policies/activate`
- `GET /api/v4/agents`
- `GET /api/v4/agents/status`
- `GET /api/v4/llmservices`
- `GET /api/v4/ai/agents`
- `GET /api/v4/ai/services`
- `GET /api/v4/cloud/limits`
- `GET /api/v4/cloud/products`
- `POST /api/v4/cloud/payment`
- `POST /api/v4/cloud/payment/confirm`
- `GET /api/v4/cloud/customer`
- `PUT /api/v4/cloud/customer/address`
- `GET /api/v4/cloud/subscription`
- `GET /api/v4/cloud/installation`
- `GET /api/v4/cloud/subscription/invoices`
- `POST /api/v4/cloud/webhook`
- `GET /api/v4/cloud/preview/modal_data`
- `GET /api/v4/content_flagging/flag/config`
- `GET /api/v4/content_flagging/team/{team_id}/status`
- `POST /api/v4/content_flagging/post/{post_id}/flag`
- `GET /api/v4/content_flagging/fields`
- `GET /api/v4/content_flagging/post/{post_id}/field_values`
- `GET /api/v4/content_flagging/post/{post_id}`
- `POST /api/v4/content_flagging/post/{post_id}/remove`
- `POST /api/v4/content_flagging/post/{post_id}/keep`
- `GET /api/v4/content_flagging/config`
- `POST /api/v4/content_flagging/team/{team_id}/reviewers/search`
- `POST /api/v4/content_flagging/post/{post_id}/assign/{content_reviewer_id}`
- `GET /api/v4/data_retention/policy`
- `GET /api/v4/data_retention/policies_count`
- `GET /api/v4/data_retention/policies`
- `GET /api/v4/data_retention/policies/{policy_id}`
- `GET /api/v4/data_retention/policies/{policy_id}/teams`
- `POST /api/v4/data_retention/policies/{policy_id}/teams`
- `POST /api/v4/data_retention/policies/{policy_id}/teams/search`
- `GET /api/v4/data_retention/policies/{policy_id}/channels`
- `POST /api/v4/data_retention/policies/{policy_id}/channels`
- `POST /api/v4/data_retention/policies/{policy_id}/channels/search`
- `GET /api/v4/groups`
- `POST /api/v4/groups`
- `GET /api/v4/groups/{group_id}`
- `PUT /api/v4/groups/{group_id}/patch`
- `DELETE /api/v4/groups/{group_id}`
- `POST /api/v4/groups/{group_id}/restore`
- `POST /api/v4/groups/{group_id}/{syncable_type}/{syncable_id}/link`
- `DELETE /api/v4/groups/{group_id}/{syncable_type}/{syncable_id}/link`
- `GET /api/v4/groups/{group_id}/{syncable_type}/{syncable_id}`
- `GET /api/v4/groups/{group_id}/{syncable_type}`
- `PUT /api/v4/groups/{group_id}/{syncable_type}/{syncable_id}/patch`
- `GET /api/v4/groups/{group_id}/stats`
- `GET /api/v4/groups/{group_id}/members`
- `POST /api/v4/groups/{group_id}/members`
- `DELETE /api/v4/groups/{group_id}/members`
- `POST /api/v4/groups/names`
- `GET /api/v4/imports`
- `POST /api/v4/imports`
- `GET /api/v4/imports/{import_name}`
- `GET /api/v4/exports`
- `POST /api/v4/exports`
- `GET /api/v4/exports/{export_name}`
- `DELETE /api/v4/exports/{export_name}`
- `GET /api/v4/ip_filtering`
- `GET /api/v4/ip_filtering/my_ip`
- `GET /api/v4/reports/users`
- `GET /api/v4/reports/users/count`
- `GET /api/v4/reports/users/export`
- `GET /api/v4/reports/posts`
- `GET /api/v4/audit_logs/certificate`
- `GET /api/v4/schemes`
- `POST /api/v4/schemes`
- `GET /api/v4/schemes/{scheme_id}`
- `PUT /api/v4/schemes/{scheme_id}/patch`
- `DELETE /api/v4/schemes/{scheme_id}`
- `GET /api/v4/schemes/{scheme_id}/teams`
- `GET /api/v4/schemes/{scheme_id}/channels`
- `GET /api/v4/sharedchannels/{team_id}`
- `GET /api/v4/sharedchannels/remote_info/{remote_id}`
- `GET /api/v4/sharedchannels/{channel_id}/remotes`
- `GET /api/v4/sharedchannels/users/{user_id}/can_dm/{other_user_id}`
- `GET /api/v4/terms_of_service`
- `POST /api/v4/terms_of_service`

### Partial/placeholder in mixed modules
- `GET /api/v4/jobs/{job_id}`
- `GET /api/v4/jobs/{job_id}/download`
- `POST /api/v4/jobs/{job_id}/cancel`
- `GET /api/v4/jobs/type/{type}`
- `GET /api/v4/jobs/{job_id}/status`
- `GET /api/v4/remotecluster`
- `GET /api/v4/remotecluster/{remote_id}`
- `POST /api/v4/remotecluster/{remote_id}/generate_invite`
- `POST /api/v4/remotecluster/accept_invite`
- `GET /api/v4/remotecluster/{remote_id}/sharedchannelremotes`
- `POST /api/v4/remotecluster/{remote_id}/channels/{channel_id}/invite`
- `POST /api/v4/remotecluster/{remote_id}/channels/{channel_id}/uninvite`
- `GET /api/v4/cluster/status`
- `GET /api/v4/compliance/reports`
- `POST /api/v4/compliance/reports`
- `GET /api/v4/compliance/reports/{report_id}`
- `GET /api/v4/compliance/reports/{report_id}/download`
- `GET /api/v4/roles` (hardcoded role set)
- `POST /api/v4/roles/names` (hardcoded lookups)
- `GET /api/v4/roles/{role_id}` (hardcoded mapping)
- `GET /api/v4/roles/name/{role_name}` (hardcoded mapping)
- `PUT /api/v4/roles/{role_id}/patch` (success stub)

### User-area stubs in `users.rs` (non-critical for baseline chat flows)
- `GET /api/v4/custom_profile_attributes/fields`
- `GET /api/v4/users/{user_id}/custom_profile_attributes`
- `GET /api/v4/users/{user_id}/sessions`
- `POST /api/v4/users/{user_id}/sessions/revoke`
- `POST /api/v4/users/{user_id}/sessions/revoke/all`
- `GET /api/v4/users/{user_id}/audits`
- `GET /api/v4/users/{user_id}/tokens`
- `GET /api/v4/users/tokens`
- `POST /api/v4/users/tokens/revoke`
- `GET /api/v4/users/tokens/{token_id}`
- `POST /api/v4/users/tokens/disable`
- `POST /api/v4/users/tokens/enable`
- `POST /api/v4/users/tokens/search`
- `GET /api/v4/users/{user_id}/oauth/apps/authorized`
- `GET /api/v4/users/{user_id}/data_retention/team_policies`
- `GET /api/v4/users/{user_id}/data_retention/channel_policies`
- `GET /api/v4/users/{user_id}/groups`

## Prioritized endpoints (mobile/desktop) and current status

| Endpoint | Status | Notes |
|---|---|---|
| `POST /api/v4/users/login` | implemented | Real auth flow and token issuance. |
| `GET /api/v4/users/me` | implemented | Real user profile from DB. |
| `GET /api/v4/users/me/teams` | implemented | Real team membership retrieval. |
| `GET /api/v4/users/me/teams/{team_id}/channels` | implemented | Real channel membership retrieval. |
| `GET /api/v4/channels/{channel_id}/posts` | implemented | Real paginated post retrieval. |
| `POST /api/v4/posts` | implemented | Real post create + websocket broadcast. |
| `GET /api/v4/config/client?format=old` | implemented | Legacy config shape expected by clients. |
| `GET /api/v4/license/client` | implemented | Client license payload (OSS/no license). |
| `GET /api/v4/websocket` | implemented | Auth challenge + event stream. |
| `GET /api/v4/plugins` | implemented | Now reflects real Calls plugin enabled state (`active` vs `inactive`). |
| `GET /api/v4/plugins/{plugin_id}` | implemented | Real status for `com.mattermost.calls`; unknown plugin returns 404. |
| `GET /api/v4/plugins/statuses` | implemented | Real status list for Calls plugin from runtime config. |
| `GET /api/v4/plugins/webapp` | implemented | Real manifest exposure when Calls is enabled. |
| `POST /api/v4/plugins` | explicit 501 | No silent fake install/upload anymore. |
| `POST /api/v4/plugins/install_from_url` | explicit 501 | No silent fake install anymore. |
| `DELETE /api/v4/plugins/{plugin_id}` | explicit 501 | Explicitly unsupported. |
| `POST /api/v4/plugins/{plugin_id}/enable` | explicit 501 | Explicitly unsupported. |
| `POST /api/v4/plugins/{plugin_id}/disable` | explicit 501 | Explicitly unsupported. |
| `POST /api/v4/actions/dialogs/open` | explicit 501 | Explicitly unsupported interactive dialogs. |
| `POST /api/v4/actions/dialogs/submit` | explicit 501 | Explicitly unsupported interactive dialogs. |
| `POST /api/v4/actions/dialogs/lookup` | explicit 501 | Explicitly unsupported interactive dialogs. |
| `POST /api/v4/ldap/sync` | explicit 501 | Enterprise-gated with explicit MM-style 501. |
| `POST /api/v4/ldap/test` | explicit 501 | Enterprise-gated with explicit MM-style 501. |

## Changes made in this pass
- Replaced silent plugin-mutation success stubs with explicit MM-style `501` responses.
- Replaced silent dialogs success stubs with explicit MM-style `501` responses.
- Implemented real Calls plugin read behavior for:
  - `GET /api/v4/plugins`
  - `GET /api/v4/plugins/{plugin_id}`
  - `GET /api/v4/plugins/statuses`
  - `GET /api/v4/plugins/webapp`
- Added shared MM-style not-implemented helper for v4 responses and aligned fallback payload shape.

## Tests updated
- Added: `backend/tests/api_v4_plugins_dialogs.rs`
  - verifies real plugin read behavior tracks config state
  - verifies plugin mutation endpoints return explicit MM-style `501`
  - verifies dialogs endpoints return explicit MM-style `501`

Test command:
- `cargo test --test api_v4_plugins_dialogs`
