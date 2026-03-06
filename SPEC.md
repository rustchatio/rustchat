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

Concrete manual command examples:
- `curl -s "$BASE/api/v4/config/client?format=old" -H "Authorization: Bearer $TOKEN" | jq '.PostEditTimeLimit,.AllowEditPost'`
- `curl -si -X PUT "$BASE/api/v4/posts/$POST_ID" -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d '{"id":"'$POST_ID'","message":"edited text"}'`

## Approval Gate

Does this plan meet your expectations? Please approve or provide feedback.
