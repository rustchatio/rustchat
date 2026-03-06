# Mobile Findings

## Upstream evidence (`../mattermost-mobile`)

1. Date separator behavior in post list:
- day-boundary insertion in list shaping: `../mattermost-mobile/app/utils/post_list/index.ts:231-246`
- rendered separator component (`Today` / `Yesterday` / date): `../mattermost-mobile/app/components/post_list/date_separator/index.tsx:46-77`

2. Edited indicator behavior:
- edited state check is `post.editAt > 0`: `../mattermost-mobile/app/utils/post/index.ts:51-53`
- edited badge text rendering (`Edited`): `../mattermost-mobile/app/components/edited_indicator/index.tsx:83-87`
- websocket `post_edited` path applies update immediately via DB batch: `../mattermost-mobile/app/actions/websocket/posts.ts:201-274`

3. Reaction toggle behavior:
- toggle add/remove based on whether current user already reacted: `../mattermost-mobile/app/actions/remote/reactions.ts:33-40`
- emoji aliases normalized before add/remove: `../mattermost-mobile/app/actions/remote/reactions.ts:54-57`, `:100-102`

4. Mobile edit menu permissions depend on config values:
- reads `AllowEditPost` and `PostEditTimeLimit`: `../mattermost-mobile/app/screens/post_options/index.ts:85-88`
- computes allowed edit window from limit: `../mattermost-mobile/app/screens/post_options/index.ts:105-111`

## Rustchat mobile-impact notes

- If Rustchat does not expose/obey `PostEditTimeLimit`, mobile menu and server behavior can diverge.
- If edit websocket/state update is delayed, mobile/web parity expectations fail (edited marker should appear immediately after edit).
