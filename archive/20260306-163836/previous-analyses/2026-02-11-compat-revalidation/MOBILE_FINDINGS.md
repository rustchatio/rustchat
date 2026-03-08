# Mobile Findings

- Screen, store, or service: Integrations REST client (commands)
- Source path: `../mattermost-mobile/app/client/rest/integrations.ts`
- Source lines: 20-39, 41-60
- Observed behavior:
  - Calls:
    - `GET /commands?team_id={teamId}`
    - `GET /teams/{team_id}/commands/autocomplete_suggestions?...`
    - `GET /teams/{team_id}/commands/autocomplete?page=&per_page=`
    - `POST /commands/execute`
    - `POST /commands`
    - `POST /actions/dialogs/submit`
- Notes: Rustchat currently lacks full parity for create command and interactive dialog submit.

- Screen, store, or service: Slash command execution flow
- Source path: `../mattermost-mobile/app/actions/remote/command.ts`
- Source lines: 41-46, 72-83
- Observed behavior:
  - Sends `channel_id`, `team_id`, `root_id`, `parent_id` when executing commands.
  - Stores `trigger_id` from command response for follow-up interactive flows.
- Notes: Backend `CommandResponse` currently lacks `trigger_id` model field.

- Screen, store, or service: Slash suggestion UI expectations
- Source path: `../mattermost-mobile/app/components/autocomplete/slash_suggestion/slash_suggestion.tsx`
- Source lines: 29-47, 96-107, 158-166
- Observed behavior:
  - Expects suggestions as array entries with `Complete/Suggestion/Hint/Description/IconData`.
  - Uses `command.auto_complete_desc` and `command.auto_complete_hint` fields.
- Notes: Rustchat `/autocomplete_suggestions` returns wrapped object with lowercase keys, causing contract mismatch.

- Screen, store, or service: Command and suggestion type contracts
- Source path: `../mattermost-mobile/types/api/integrations.d.ts`
- Source lines: 4-23, 33-39
- Observed behavior:
  - `Command` includes Mattermost-style fields (`create_at`, `update_at`, `delete_at`, `method` in `P|G|''`, etc.).
  - `AutocompleteSuggestion` is typed with TitleCase keys.
- Notes: Rustchat `SlashCommand`/response shapes are still reduced vs this contract.

- Screen, store, or service: Command response type contract
- Source path: `../mattermost-mobile/types/api/commands.d.ts`
- Source lines: 4-7
- Observed behavior:
  - `CommandResponse` includes `trigger_id` as optional but used in runtime flow.
- Notes: Missing field in backend strongly impacts dialog/app command handoffs.

- Screen, store, or service: Interactive dialog submit action
- Source path: `../mattermost-mobile/app/actions/remote/integrations.ts`
- Source lines: 14-23
- Observed behavior:
  - Mobile submits dialogs to `/actions/dialogs/submit` during integration workflows.
- Notes: Rustchat currently responds 501 for this endpoint.

- Screen, store, or service: Files API expectations
- Source path: `../mattermost-mobile/app/client/rest/files.ts`
- Source lines: 37-53, 62-93
- Observed behavior:
  - Uses `/files/{id}/thumbnail` and `/files/{id}/preview`.
  - Upload requires `localPath`; sends multipart with `channel_id`.
- Notes: Rustchat route coverage exists for these paths.

- Screen, store, or service: Scheduled posts REST client
- Source path: `../mattermost-mobile/app/client/rest/base.ts`
- Source lines: 174-176, 210-212
- Observed behavior:
  - Scheduled post routes are under `/posts/schedule` and `/posts/scheduled/...`.
- Notes: Confirms old `/scheduled_posts` path report is outdated for current mobile client.

- Screen, store, or service: Scheduled posts response expectations
- Source path: `../mattermost-mobile/app/client/rest/scheduled_post.ts` and `../mattermost-mobile/app/actions/remote/scheduled_post.ts`
- Source lines: `scheduled_post.ts:47-51`; `actions/remote/scheduled_post.ts:80-83`
- Observed behavior:
  - Fetch expects response containing team-keyed groups and optional `directChannels`, then flattens values.
- Notes: Rustchat currently returns a flat list from `GET /posts/scheduled/team/{team_id}`.

- Screen, store, or service: WebSocket event consumption
- Source path: `../mattermost-mobile/app/constants/websocket.ts` and `../mattermost-mobile/app/actions/websocket/event.ts`
- Source lines: `websocket.ts:7-12,22,31-33,42,45,47-49,55,101-103`; `event.ts:27-37,64-69,111-113,138-150,162-164`
- Observed behavior:
  - Mobile handles standard events (`posted`, `typing`, `status_change`, `reaction_*`, `channel_viewed`, `user_added`, `user_removed`) plus `open_dialog` and scheduled-post websocket events.
- Notes: Rustchat maps standard events, but scheduled-post and `open_dialog` server emission paths were not found in the current backend mapping.
