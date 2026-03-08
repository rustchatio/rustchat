# SPEC: Message Timeline, Edit Policy, Reaction Toggle, and Notification Dot Parity Fixes

## Problem Statement

You reported four user-visible regressions in Rustchat WebUI (with mobile compatibility expectations):

1. Older messages show only time, not day separation context.
2. Message editing should follow a global policy (disabled/enabled/time-limited like 30 minutes), show an "edited" marker, and update immediately after edit.
3. Emoji reactions should toggle correctly per user (second click removes own reaction, counts decrement correctly while preserving others).
4. Top notification dot should clear after unread messages are viewed.

These are compatibility-sensitive because they touch Mattermost-consumed post/edit/reaction/unread behaviors and mobile-consumed config/edit policy semantics.

## Goals

1. Match Mattermost-style day-boundary rendering behavior in WebUI message timeline.
2. Enforce and expose global post edit policy with Mattermost-compatible semantics.
3. Ensure edited posts are reflected immediately and visibly marked as edited.
4. Guarantee reaction toggle parity for add/remove/count behavior.
5. Ensure bell unread indicator clears reliably when read state is updated.

## Non-Goals

1. No large frontend architecture migration from legacy stores to `features/*` stores.
2. No unrelated visual redesign.
3. No upstream repository edits (`../mattermost`, `../mattermost-mobile`).

## Scope and Contract Impact

Compatibility GAP baseline:
- For all API compatibility GAP definitions in this spec and related analysis artifacts,
  the reference implementation is Mattermost.

In scope:
- WebUI timeline rendering for day separators.
- WebUI message row edited indicator + immediate local update path.
- Backend v4 post edit enforcement for global edit limit.
- Backend config/client compatibility payload additions for edit-policy keys.
- WebUI unread bell-dot state unification.
- Reaction toggle behavior verification/fix in existing WebUI flow.

Contract-sensitive surfaces:
- `PUT /api/v4/posts/{post_id}`
- `PUT /api/v4/posts/{post_id}/patch`
- `GET /api/v4/config/client?format=old`
- Websocket `post_edited`, `reaction_added`, `reaction_removed`, `unread_counts_updated`

## API Contract Requirements (Normative)

### FR-API-001 Edit Endpoints and Policy Modes
- The system MUST enforce a global edit policy with three modes:
  - `disabled`: all edit attempts are denied.
  - `enabled`: edits are allowed without a time limit.
  - `time_limited`: edits are allowed only within `PostEditTimeLimit` seconds from post creation.
- `PostEditTimeLimit` MUST be interpreted in whole seconds.
- Boundary behavior for `time_limited` MUST be explicit:
  - edit is allowed when `now - create_at <= PostEditTimeLimit`
  - edit is denied when `now - create_at > PostEditTimeLimit`
- `PostEditTimeLimit <= 0` in `time_limited` mode MUST be treated as deny-all edits.

### FR-API-002 Config Client Compatibility Keys
- `GET /api/v4/config/client?format=old` MUST expose compatibility keys:
  - `AllowEditPost`
  - `PostEditTimeLimit`
- Value formats MUST be stable and documented:
  - `AllowEditPost`: stringified boolean (`"true"` or `"false"`)
  - `PostEditTimeLimit`: base-10 integer string in seconds

### FR-API-003 Authorization and Permission Matrix
- `PUT /api/v4/posts/{post_id}` and `PUT /api/v4/posts/{post_id}/patch`:
  - caller MUST be authenticated
  - caller MUST be the post author or otherwise authorized by server policy
  - unauthorized/forbidden edits MUST return the documented deny response
- `GET /api/v4/config/client?format=old`:
  - caller MUST satisfy route-level authentication requirements used by the compatibility layer
- Websocket events:
  - `post_edited`, `reaction_added`, `reaction_removed`, `unread_counts_updated` MUST only be delivered to sessions with channel/team visibility required by existing read permissions

### FR-API-004 Error Semantics for Edit Deny Paths
- When edit is denied by policy or authorization, both edit endpoints MUST return:
  - documented HTTP status code
  - documented error ID string
  - documented JSON error envelope fields
- Error semantics MUST be identical across full update and patch routes for equivalent deny reasons.
- Status code and error envelope values for deny paths MUST match Mattermost behavior exactly,
  and evidence paths with line numbers MUST be recorded in compatibility artifacts before merge.
- Required deny-path values:
  - unauthenticated: `401` + `api.context.session_expired.app_error`
  - authenticated but unauthorized: `403` + `api.context.permissions.app_error`
  - outside edit window: `400` + `api.post.update_post.permissions_time_limit.app_error`
- Required error envelope fields for deny responses:
  - `id`, `message`, `detailed_error`, `status_code`, `request_id`

## Evidence Summary

Upstream references:
- Web date separators: `../mattermost/webapp/channels/src/packages/mattermost-redux/src/utils/post_list.ts:115-135`
- Web edited semantics: `../mattermost/webapp/channels/src/packages/mattermost-redux/src/utils/post_utils.ts:51-53,69-74`
- Server edit limit enforcement: `../mattermost/server/channels/api4/post.go:1074-1076,1186-1188`
- Mobile date separators: `../mattermost-mobile/app/utils/post_list/index.ts:231-246`
- Mobile edit config consumption: `../mattermost-mobile/app/screens/post_options/index.ts:85-88,105-111`
- Mobile reaction toggle: `../mattermost-mobile/app/actions/remote/reactions.ts:33-40`

Current Rustchat evidence:
- No date separator rendering: `frontend/src/components/channel/MessageList.vue:183-209`
- Message rows show only time stamps: `frontend/src/components/channel/MessageItem.vue:226,245,289,333`
- Edit emits update but parent chain does not handle it: `frontend/src/components/channel/MessageItem.vue:134-144`, `frontend/src/views/main/ChannelView.vue:231-237`
- No v4 edit-limit check: `backend/src/api/v4/posts.rs:1249-1301`
- Config/client missing edit-policy keys: `backend/src/api/v4/config_client.rs:133-420`
- Bell dot uses different unread store namespace: `frontend/src/components/layout/GlobalHeader.vue:13`

## Implementation Outline

1. Add date-separator rendering in `MessageList.vue` based on day transitions.
2. Extend message model/store to carry edit timestamp and render edited badge in `MessageItem.vue`.
3. Wire `MessageItem` edit update event through `MessageList` -> `ChannelView` -> message store for immediate UI update.
4. Implement global post edit limit enforcement in backend v4 edit routes (and v1 route if current WebUI path still uses it).
5. Expose `PostEditTimeLimit` (and compatible `AllowEditPost`) in `/api/v4/config/client?format=old`.
6. Unify unread bell source to `stores/unreads` used by websocket/read flows.
7. Verify reaction toggle behavior matrix and patch any discovered edge-case in frontend handlers.

## Security and Audit Requirements (Normative)

### FR-SEC-001 Integrity and Trust Boundaries
- Edit-policy configuration values used for enforcement MUST come from trusted server configuration sources only.
- Clients MUST NOT be able to bypass server-side policy enforcement by local state manipulation.

### FR-SEC-002 Mutation Auditability
- Post edit, reaction mutation, and read-state mutation flows MUST emit auditable server logs.
- At minimum, audit records MUST include actor ID, target entity ID, action type, and timestamp.
- Audit logging requirements in this spec MUST align with repository retention and operational policy.

## Error Semantics and Permission Matrix

### FR-MAT-001 Permission Outcomes
- Unauthenticated callers: MUST receive the documented authentication error for each protected surface.
- Authenticated but unauthorized callers: MUST receive the documented authorization error.
- Authorized callers outside edit window (time-limited mode): MUST receive the documented policy-deny error.

### FR-MAT-002 Contract Uniformity
- Equivalent permission and policy outcomes MUST produce consistent status and error-envelope shapes across both edit endpoints.
- Uniform outcomes MUST also match Mattermost wire contract semantics for the same scenario class.

## Websocket Event Payload Contract (Normative)

### FR-WS-001 `post_edited`
- Event name MUST be `post_edited`.
- `data.post` MUST be present and contain a serialized post JSON string in compatibility format.

### FR-WS-002 `reaction_added` and `reaction_removed`
- Event names MUST be `reaction_added` and `reaction_removed`.
- `data.reaction` MUST be present and contain a serialized reaction JSON string.
- Reaction payload MUST include user, post, emoji, and timestamps required by compatible clients.

### FR-WS-003 `unread_counts_updated`
- Event name MUST be `unread_counts_updated`.
- Payload MUST include: `channel_id`, `msg_count`, `msg_count_root`, `mention_count`,
  `mention_count_root`, `urgent_mention_count`, `last_viewed_at`.
- If compatibility mapping emits `post_unread` for client parity, field values MUST remain
  semantically equivalent to `unread_counts_updated`.

### FR-WS-004 Ordering and Precedence
- For the same post/reaction/unread entity, later server state MUST supersede earlier state.
- When websocket events arrive out of order, client-visible state MUST converge to the latest
  server-acknowledged state after resync.

## Recovery and Event Ordering Requirements

### FR-REC-001 Realtime Recovery Expectations
- Requirements MUST define behavior for delayed, duplicated, and out-of-order websocket events affecting edited markers, reactions, and unread indicators.
- On inconsistency detection, clients MUST have a documented resync path (for example, channel/post refresh) that restores canonical server state.

### FR-REC-002 Deterministic "Immediate" Update Semantics
- "Immediate" UI update after successful edit MUST mean local state reflects the server-accepted change within one successful response cycle.
- If websocket confirmation is delayed or reordered, requirements MUST define precedence rules between direct response data and websocket event data.

## Alternate and Unsupported Scenario Requirements

### FR-ALT-001 Partial/Unsupported Compatibility Paths
- Any out-of-scope or unsupported compatibility route discovered during implementation MUST
  return explicit `501 Not Implemented` semantics with compatibility headers unchanged.
- Unsupported behavior MUST be recorded as a GAP entry with severity and user impact.

### FR-ALT-002 Reaction Concurrency Semantics
- For a given `(post_id, user_id, emoji_name)` tuple, at most one active reaction state is allowed.
- Repeated toggle by the same user MUST be idempotent over retries and converge to one final state.
- Concurrent toggles by multiple users MUST preserve per-user ownership and produce counts equal
  to the number of distinct active user reactions for that emoji on the post.

## Realtime Performance Targets (Normative)

### FR-NFR-001 Websocket Propagation Targets
- Under normal operating load, p95 end-to-end propagation for edit/reaction/unread websocket
  updates SHOULD be <= 1000ms and p99 SHOULD be <= 2000ms.
- Any regression above these targets in compatibility-sensitive flows MUST be documented with
  mitigation or accepted risk before release.

## Verification Prerequisites and Release Decision Rule

### FR-REL-001 Mandatory Prerequisites
- Compatibility smoke checks require a live server endpoint exposing `X-MM-COMPAT: 1`.
- Integration tests require configured test dependencies (`RUSTCHAT_TEST_DATABASE_URL` and
  related services) before results can be used for release decisions.

### FR-REL-002 Ownership and Gate Outcome
- If mandatory prerequisites are unmet, verification status MUST be `BLOCKED`, not `PASS`.
- Release/readiness decision owner MUST document: executed commands, outcomes, unmet gates,
  and explicit go/no-go decision.

## Verification Plan

Automated:
- `cd frontend && npm run build`
- `cd backend && cargo clippy --all-targets --all-features -- -D warnings`
- `cd backend && cargo test --no-fail-fast -- --nocapture`

Manual compatibility checks:
- Date separators visible when scrolling across day boundaries.
- Edit within limit succeeds, over limit fails with compatible error ID.
- Edited badge appears immediately after successful edit.
- Reaction click matrix:
  - add on first click
  - remove on second click by same user
  - decrement but keep emoji when other users reacted
- Bell dot clears after entering unread channel and read marking.
- Denied edit checks return required values:
  - unauthenticated => `401` + `api.context.session_expired.app_error`
  - unauthorized => `403` + `api.context.permissions.app_error`
  - time-limited deny => `400` + `api.post.update_post.permissions_time_limit.app_error`
- Websocket payload checks validate required data keys:
  - `post_edited` => `data.post`
  - `reaction_added` / `reaction_removed` => `data.reaction`
  - `unread_counts_updated` => required unread count fields

Concrete manual command examples:
- `curl -s "$BASE/api/v4/config/client?format=old" -H "Authorization: Bearer $TOKEN" | jq '.PostEditTimeLimit,.AllowEditPost'`
- `curl -si -X PUT "$BASE/api/v4/posts/$POST_ID" -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d '{"id":"'$POST_ID'","message":"edited text"}'`

## Traceability Matrix (FR/AC IDs)

- **AC-001** (Edit policy enforcement): validates FR-API-001, FR-API-004, FR-MAT-001, FR-MAT-002
- **AC-002** (Config compatibility keys): validates FR-API-002
- **AC-003** (Authorization coverage): validates FR-API-003, FR-MAT-001
- **AC-004** (Realtime recovery behavior): validates FR-REC-001, FR-REC-002
- **AC-005** (Security/audit controls): validates FR-SEC-001, FR-SEC-002
- **AC-006** (Websocket payload contract): validates FR-WS-001, FR-WS-002, FR-WS-003, FR-WS-004
- **AC-007** (Alternate/concurrency coverage): validates FR-ALT-001, FR-ALT-002
- **AC-008** (Performance and release gates): validates FR-NFR-001, FR-REL-001, FR-REL-002

Manual acceptance checks MUST cite AC IDs in test notes.

## Approval Gate

Does this plan meet your expectations? Please approve or provide feedback.
