# Summary

- Topic: 2026-02 re-validation of Rustchat <-> Mattermost mobile compatibility (post-archive consolidation)
- Date: 2026-02-11
- Scope: Commands/integrations, interactive dialogs, scheduled posts, websocket event contract, files preview contract

- Compatibility contract:
  - Confirmed aligned:
    - `/api/v4` fallback and explicit Mattermost-style 501 payloads (`backend/src/api/v4/mod.rs:103-125`).
    - `X-MM-COMPAT: 1` response header (`backend/src/api/v4/mod.rs:106-110`).
    - File preview/thumbnail routes for mobile (`backend/src/api/v4/files.rs:25-30`).
    - Core websocket event name mapping exists for major post/reaction/status/channel/member events (`backend/src/api/v4/websocket.rs:763-949`).
  - Confirmed mismatches / gaps:
    - Commands API still partial/stubbed:
      - `GET /commands` is hardcoded (`backend/src/api/v4/commands.rs:63-77`).
      - `POST /commands` not exposed in v4 router (`backend/src/api/v4/commands.rs:16-33`).
      - `/teams/{team_id}/commands/autocomplete` returns empty (`backend/src/api/v4/teams.rs:903-909`).
      - `/autocomplete_suggestions` shape mismatches mobile contract (wrapper + lowercase keys + /call-only) (`backend/src/api/v4/commands.rs:137-159` vs `../mattermost-mobile/types/api/integrations.d.ts:33-39`).
    - Interactive dialogs:
      - Mobile actively submits dialogs (`../mattermost-mobile/app/actions/remote/integrations.ts:14-23`), but Rustchat returns 501 (`backend/src/api/v4/dialogs.rs:23-42`).
    - Command response contract:
      - Mobile uses `trigger_id` (`../mattermost-mobile/app/actions/remote/command.ts:78-80`), while Rustchat `CommandResponse` lacks it (`backend/src/models/integration.rs:166-173`).
    - Scheduled posts:
      - Contrary one prior report, scheduled-post routes exist and align on path family (`backend/src/api/v4/posts.rs:74-79`; `../mattermost-mobile/app/client/rest/base.ts:174-176,210-212`).
      - But team-list response shape differs from Mattermost/mobile expectation (Rustchat vector vs Mattermost map with `directChannels`) (`backend/src/api/v4/posts.rs:958-996` vs `../mattermost/server/channels/api4/scheduled_post.go:131-142`).

- Open questions:
  - Should Rustchat preserve strict Mattermost scheduled-post response envelope (`{teamId: [...], directChannels: [...]}`) even if local logic can flatten?
  - Should v4 command CRUD be fully wired to DB now, or gated behind explicit 501 until parity is complete?
  - Do we need scheduled-post websocket events in this iteration or can we defer to REST correctness first?

- Prior-analysis consistency check:
  - `commands_gap.md` claims full closure (`previous-analyses/2026-02-07-compat-audit-reports/commands_gap.md:5-20`) but `commands_gap_matrix.md` reports major gaps (`previous-analyses/2026-02-07-compat-audit-reports/commands_gap_matrix.md:11-20`).
  - Current source validates the matrix-style “partial” view.
  - `compat_score.md` lists `/scheduled_posts*` as missing (`previous-analyses/2026-02-07-compat-audit-reports/compat_score.md:40-43`), but current mobile and backend use `/posts/schedule*`; that section is stale.
