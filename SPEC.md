# SPEC: App Shell Redesign Plan from DESIGN.md (2026-03-28)

## Problem Statement

RustChat now has a clear design system in `DESIGN.md`, but the authenticated shell still does not fully express it.

The structure is already correct. The product feels familiar and usable. The problem is that the shell still behaves like a collection of competent components instead of one strong product surface.

The main gaps are:

1. The global header still gives too much visual weight to generic utility chrome, especially search, and not enough to product identity and current working context.
2. The left-side navigation stack is functional, but the team rail and channel sidebar do not yet feel like a crisp scanning system with deliberate hierarchy.
3. The center canvas still has weak low-activity and empty-state presence, so the main workspace can feel visually abandoned.
4. Shell surfaces are more consistent than before, but they still need one end-to-end pass to align with the new “Focused Warm Utility” system in `DESIGN.md`.

## Goals

1. Translate `DESIGN.md` into a concrete implementation plan for the authenticated shell.
2. Make the first viewport feel more like a product and less like a neutral internal tool.
3. Improve scan speed and context clarity in the header, left navigation, and message canvas.
4. Keep the rollout incremental and verifiable, without rewriting the whole UI at once.

## Non-Goals

1. No backend or API changes.
2. No marketing-site redesign.
3. No repo-wide reskin of every settings, admin, or modal surface in this phase.
4. No new information architecture for channels, threads, or right sidebar workflows.
5. No new persisted preferences or theme schema changes.

## Scope and Contract Impact

In scope:
- `frontend/src/components/layout/AppShell.vue`
- `frontend/src/components/layout/GlobalHeader.vue`
- `frontend/src/components/layout/TeamRail.vue`
- `frontend/src/components/layout/ChannelSidebar.vue`
- `frontend/src/components/channel/ChannelHeader.vue`
- `frontend/src/components/channel/MessageList.vue`
- small supporting shared pieces only if needed for consistency

Contract impact:
- No server/API contract change.
- No intended behavioral change to channel data, calls, settings, or permissions.
- User-visible contract change: the authenticated shell should have stronger identity, clearer hierarchy, and more intentional empty/loading states.

Out of scope for this phase:
- broad admin redesign
- deep modal redesign outside shell-adjacent surfaces
- removing old files unless directly required for shell correctness

## Current Implementation Findings

1. `frontend/src/components/layout/AppShell.vue` already has the correct structural layout, but it reads visually flat. The frame is functional, not yet distinct.
2. `frontend/src/components/layout/GlobalHeader.vue` now has a better logo treatment, but search still dominates the center and the header still feels like generic utility chrome first, brand/context second.
3. `frontend/src/components/layout/ChannelSidebar.vue` has good channel grouping logic, but the visual language is still more “list of rows” than “high-confidence collaboration navigator”.
4. `frontend/src/components/channel/ChannelHeader.vue` works well functionally, but the current context hierarchy is still light. Channel name, topic, and actions need stronger prioritization.
5. `frontend/src/components/channel/MessageList.vue` already has date separators and a usable empty state, but the empty/loading states still need more intentional visual anchoring so the center panel does not feel empty by accident.

## Design Translation Principles

Derived directly from `DESIGN.md`:

1. **Slack ease + Mattermost trust**
   - approachable, low-friction interaction
   - serious, dependable visual tone

2. **Focused Warm Utility**
   - warm neutrals
   - restrained accents
   - stronger context than decoration

3. **Content-first shell**
   - users should orient in under a second
   - brand supports the workspace, it does not fight it

4. **Minimal-functional motion**
   - transitions help comprehension only
   - no ornamental flourishes

## Implementation Outline

### Phase 1: Frame and Header

Target files:
- `frontend/src/components/layout/AppShell.vue`
- `frontend/src/components/layout/GlobalHeader.vue`

Work:
- reduce the visual dominance of the search field
- strengthen brand and workspace presence in the first viewport
- improve spacing and separation between brand, search, notifications, and user controls
- make the shell frame feel more deliberate through subtle surface contrast and hierarchy, not heavier decoration

Expected outcome:
- users immediately understand “where they are” and “what product this is” before utility chrome takes over

### Phase 2: Left Navigation Hierarchy

Target files:
- `frontend/src/components/layout/TeamRail.vue`
- `frontend/src/components/layout/ChannelSidebar.vue`

Work:
- tighten selection, unread, mention, and category hierarchy
- improve scanning contrast between favorites, standard channels, DMs, and controls
- make the team rail feel like a compact identity/navigation rail rather than a thin leftover column

Expected outcome:
- faster navigation scanning
- stronger sense of structure and activity on the left side of the app

### Phase 3: Current Context Bar

Target file:
- `frontend/src/components/channel/ChannelHeader.vue`

Work:
- increase emphasis on channel name and current context
- de-emphasize secondary actions slightly while keeping them easy to hit
- make topic handling more intentional, especially when absent or truncated

Expected outcome:
- clearer context handoff between sidebar selection and center canvas

### Phase 4: Center Canvas Presence

Target file:
- `frontend/src/components/channel/MessageList.vue`

Work:
- redesign loading and empty states to feel intentional and branded
- strengthen the visual anchor of the center column when a channel is quiet
- preserve message readability and timeline clarity

Expected outcome:
- the main workspace still feels alive and deliberate even when message volume is low

## Verification Plan

Automated:
- `cd frontend && npm run build`

Manual:
- `cd frontend && npm run dev`
- verify the authenticated shell in these states:
  - active team with unread channels
  - active channel with topic
  - active channel without topic
  - empty channel / no messages
  - mobile drawer open
  - right sidebar open
- check these outcomes:
  - header identity and context are clearer than before
  - left-nav scanning is faster and unread state is easier to parse
  - center panel no longer feels abandoned in empty or quiet states
  - no regression in touch targets or mobile shell behavior

## Proposed Rollout Order

1. Header and frame
2. Left navigation
3. Channel header
4. Message canvas states

This order matters because it changes the first impression quickly while keeping the risk low and the verification obvious.

## Expected Result

After this phase, RustChat should still feel familiar to Slack and Mattermost users, but more ownable, warmer, and more intentional.

The user should feel:
- “this is easy to read”
- “this feels like a serious collaboration product”
- “the product has a point of view”

---

# SPEC: Theme Source-of-Truth Fix + Brand/Typography Second Pass (2026-03-28)

## Problem Statement

The first design pass fixed control density and mobile settings navigation, but Rustchat still reads as generic SaaS. The main reasons are:

1. The product still defaults to Inter plus indigo-heavy tokens, which gives the shell a starter-template feel.
2. The theme system is split across two implementations:
   - active app wiring in `frontend/src/stores/theme.ts`
   - newer parallel implementation in `frontend/src/features/theme/*`
3. Many surfaces still use hardcoded `gray-*`, `dark:*`, and direct Tailwind color classes instead of the app tokens from `frontend/src/style.css`.

That split makes theme behavior unreliable. Theme selection can update some surfaces while other text and surfaces keep old assumptions, which risks unreadable font colors or washed-out hierarchy on certain theme choices.

## Goals

1. Move the app out of the “generic SaaS” band with a more intentional default typography/brand direction.
2. Make theme application predictable by using one theme source of truth for runtime appearance.
3. Audit and fix font/text color usage on key authenticated surfaces so theme changes do not lose readability.
4. Preserve the existing theme options and user-facing theme picker behavior.
5. Keep changes focused on the app shell and settings surfaces first, not a repo-wide redesign.

## Non-Goals

1. No backend API contract changes.
2. No new theme picker UX or new persisted preference schema.
3. No full migration of every legacy component in one pass.
4. No marketing-site redesign. This is app UI work.

## Scope and Contract Impact

In scope:
- `frontend/src/stores/theme.ts`
- `frontend/src/main.ts`
- `frontend/src/style.css`
- authenticated surfaces with strong theme/brand impact, especially:
  - `frontend/src/components/layout/GlobalHeader.vue`
  - `frontend/src/components/settings/SettingsModal.vue`
  - `frontend/src/components/settings/display/DisplayTab.vue`
  - `frontend/src/views/settings/ProfileView.vue`
  - shared setting-row components if needed

Contract impact:
- No server/API contract change expected.
- Persisted local/server appearance preferences should keep the same stored values (`theme`, `font`, `font_size`).
- User-visible contract change: theme selection should consistently update shell/text colors, and the default appearance should feel more intentional.

Out of scope:
- Removing `frontend/src/features/theme/*` unless that becomes necessary for correctness.
- Restyling every admin page or every old legacy view in this pass.

## Current Implementation Findings

1. `frontend/src/main.ts` boots the legacy store via `useThemeStore(pinia).applyAppearance()`.
2. Settings/profile/theme controls also use the legacy store in `frontend/src/stores/theme.ts`.
3. A second theme stack exists in `frontend/src/features/theme/services/themeService.ts` and `frontend/src/features/theme/stores/themeStore.ts`, but it is not the active entrypoint.
4. `frontend/src/stores/theme.ts` writes extra legacy variables like `--theme-sidebar-bg`, `--theme-center-channel-color`, etc., while the actual app token mapping in `frontend/src/style.css` is driven by `--theme-*` core tokens and `data-theme`.
5. Several UI surfaces, especially `frontend/src/views/settings/ProfileView.vue`, still use direct `bg-gray-*`, `text-gray-*`, and `dark:*` classes rather than tokenized theme colors, which makes theme coverage incomplete.

## Implementation Outline

1. Establish one runtime theme source of truth.
   - Keep the currently active legacy store path for this task.
   - Treat `frontend/src/stores/theme.ts` as the canonical runtime controller.
   - Avoid mixing writes from `frontend/src/features/theme/*` during this pass.

2. Tighten theme token usage.
   - Audit key authenticated surfaces for hardcoded gray/dark classes.
   - Replace them with app tokens (`bg-bg-*`, `text-text-*`, `border-border-*`, `brand`, etc.) where appropriate.
   - Prioritize settings/profile/display surfaces and shell hierarchy first.

3. Improve typography/brand baseline.
   - Change the default app font away from Inter to a more intentional existing font option already defined in `frontend/src/style.css`.
   - Rebalance shell hierarchy so product identity and context feel stronger than utility chrome.
   - Keep the app calm and functional, not decorative.

4. Normalize theme color behavior for text contrast.
   - Review each shipped theme in `frontend/src/style.css` for text/body/subtle text pairings.
   - Adjust token values where necessary so primary text, muted text, and brand accents remain legible.
   - Specifically validate dark, futuristic, dynamic, and high-contrast themes, where failure risk is highest.

5. Verify end-to-end appearance behavior.
   - Confirm theme switches update the app shell, settings surfaces, and profile/preferences screens consistently.
   - Confirm font and font-size selections still apply correctly after the cleanup.

## Verification Plan

Automated:
- `cd frontend && npm run build`
- `cd frontend && npm run test:e2e:settings-parity` (update or re-baseline only after intentional visual review)

Manual:
- Run the app and switch through at least: `light`, `dark`, `modern`, `dynamic`, `futuristic`, `high-contrast`.
- Verify these screens after each switch:
  - main channel view
  - settings modal: notifications, display, profile
  - profile/settings page if still reachable separately
- Verify:
  - primary text remains clearly readable
  - muted/helper text is still readable and not washed out
  - brand/action buttons remain distinct from background
  - font choice updates globally
  - font size updates globally

Concrete manual verification command:
- `cd frontend && npm run dev`

Expected result after implementation:
- Rustchat should feel less like a default template.
- Theme selection should no longer leave key text colors stranded on the wrong contrast assumptions.

---

# SPEC: WebSocket Auth Expiry Enforcement (2026-03-13)

> **📁 MOVED TO SUPERPOWERS STRUCTURE**
> This spec has been restructured to: `docs/superpowers/specs/2026-03-13-websocket-auth-expiry-design.md`

> **Status: ✅ IMPLEMENTED** (2026-03-17)
> 
> - Both `/api/v4/websocket` and `/ws` endpoints enforce token expiry
> - Frontend detects auth expiry and triggers logout flow
> - Close code 1008 (POLICY_VIOLATION) sent on auth expiry

## Problem Statement

Current behavior allows an already-established WebSocket connection to keep receiving realtime events after the JWT access token has expired.  
Observed by user: without refreshing the page, the web UI continues receiving messages after token expiry.

This is a security and session-integrity bug. Expired credentials must not continue to authorize realtime delivery.
Additionally, when JWT expiry is reached, the web UI must not keep showing stale authenticated screens; it must clear session-scoped UI data and navigate to login.

## Goals

1. Ensure active WebSocket sessions stop receiving events as soon as JWT `exp` is reached.
2. Apply consistent behavior to both websocket entrypoints:
   - `/api/v4/websocket`
   - `/ws` (legacy endpoint)
3. Force web UI logout UX when JWT expires:
   - clear authenticated token/session
   - clear session-scoped messaging state from stores
   - navigate to login screen immediately
4. Keep behavior backward-compatible for valid, unexpired tokens.

## Non-Goals

1. No refresh-token or silent re-auth feature in this task.
2. No JWT revocation list implementation in this task.
3. No changes to unrelated HTTP auth flows.

## Scope and Contract Impact

In scope:
- WebSocket auth validation lifecycle.
- WebSocket connection close behavior when auth expires.
- Frontend auth-expiry detection and UX enforcement (logout + state clear + login redirect).

Contract impact:
- Realtime auth semantics become stricter: expiry now applies during an already-open socket session, not only at handshake.
- Expected close reason/code for expiry will be a policy/auth violation close from server side.
- On JWT expiry, user-visible contract changes from "stale authenticated screen persists" to "forced transition to login with cleared session state."

Out of scope:
- API response body shape changes.
- Refresh-token rotation/re-issue.

## Upstream Compatibility Evidence

Server-side reference (`../mattermost`):
- `../mattermost/server/channels/app/platform/web_conn.go:880` (`ShouldSendEvent`) checks `IsAuthenticated()` before sending each event.
- `../mattermost/server/channels/app/platform/web_conn.go:778` (`IsBasicAuthenticated`) reloads session when expiry passes; invalid/expired session results in unauthenticated state.

Implication: upstream does not continue delivering websocket events to expired sessions.

Current Rustchat behavior:
- `backend/src/api/websocket_core.rs:363` validates token only when extracting initial user id.
- `backend/src/api/v4/websocket.rs:203` and `backend/src/api/ws.rs:87` run the socket loop without any token-expiry deadline enforcement afterward.
- `frontend/src/composables/useWebSocket.ts` reconnects on close but does not treat auth-expiry close as a forced logout flow.

## Implementation Outline

1. Extend websocket auth parsing to capture JWT claims (`sub`, `exp`) instead of only user id.
2. Thread token expiration timestamp through websocket connection setup for:
   - `/api/v4/websocket`
   - `/ws`
3. In the websocket run loop, add an expiry deadline branch:
   - when reached, close connection with policy/auth close code
   - perform normal cleanup (hub unregister, presence/offline handling, actor disconnect)
4. Add frontend session-expiry enforcement:
   - add JWT-expiry timer in auth lifecycle (login + rehydrate paths)
   - on timer expiration, execute one centralized logout path
   - centralized logout must clear token/cookie/user and reset session-scoped stores before redirect
5. Add websocket close handling in frontend:
   - if socket closes with auth/policy violation for expired token, trigger centralized logout immediately
   - do not continue reconnect loop in expired-token scenario
6. Add regression tests for claim extraction / expiry-aware auth plumbing where feasible.

## Verification Plan

Automated:
- `cd backend && cargo test --no-fail-fast -- --nocapture`
- `cd backend && cargo check`
- `cd frontend && npm run build`

Manual:
- Start backend with short token lifetime (for example `RUSTCHAT_JWT_EXPIRY_HOURS=1`).
- Login in web UI, keep channel open, wait until token expiration boundary.
- Verify websocket disconnect occurs at/after expiry and no new realtime messages appear.
- Verify web UI immediately transitions to `/login` and previously visible channel/message content is cleared from authenticated view.
- Verify no reconnect storm continues with expired token (socket remains disconnected until new login).
- Optional wire check: monitor websocket close in browser devtools Network tab.

Concrete command example:
- `cd backend && RUSTCHAT_JWT_EXPIRY_HOURS=1 cargo run`

---

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
