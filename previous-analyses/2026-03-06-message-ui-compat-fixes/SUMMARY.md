# Summary

- Topic: WebUI + mobile-compat message timeline/edit/reaction/unread fixes
- Date: 2026-03-06
- Scope:
  - Date separators for older messages in WebUI (Mattermost-like behavior)
  - Global post edit limit parity (`disabled` / `enabled` / `N seconds`, e.g. 30 min)
  - Edited indicator rendering + immediate post update after edit
  - Reaction toggle parity (add/remove on second click, count/user list consistency)
  - Top notification dot clearing after unread messages are viewed
- Compatibility contract:
  - Date separators are inserted when message day changes (web and mobile parity paths).
  - Edited state is represented by non-zero edit timestamp (`edit_at` / `editAt`), and edit permission is gated by `PostEditTimeLimit`.
  - Reactions are toggle semantics per user per emoji.
  - Unread indicator must clear when channel is marked viewed/read.
- Open questions:
  - For WebUI editing controls: should we expose exact Mattermost-style `AllowEditPost` + `PostEditTimeLimit` values in the existing admin UI now, or first enforce backend + config/client contract and keep UI wiring minimal in this patch?
