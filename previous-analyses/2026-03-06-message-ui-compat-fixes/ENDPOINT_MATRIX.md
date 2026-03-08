# Endpoint Matrix

| Surface | Upstream Expectation | Rustchat Current | Gap |
|---|---|---|---|
| Web post list date boundaries | Insert date separator when day changes (`DATE_LINE`) | No date separator logic in `MessageList.vue`; message rows show only `h:mm a` | Yes (P2) |
| `PUT /api/v4/posts/{post_id}` | Enforce `PostEditTimeLimit` if configured | Owner-only edit, no time-limit check | Yes (P1) |
| `PUT /api/v4/posts/{post_id}/patch` | Enforce same edit-time contract | Same no-limit path | Yes (P1) |
| `GET /api/v4/config/client?format=old` | Include edit config (`PostEditTimeLimit`, `AllowEditPost`) consumed by clients | Keys absent in legacy config map | Yes (P1) |
| Edited marker display | Show `Edited` when `edit_at > 0` | Message UI has no edited indicator | Yes (P2) |
| Reaction toggle semantics | Same emoji by same user toggles add/remove | Toggle call exists; needs parity validation with local immediate state and counter decrements | Potential (P2) |
| Notification bell unread dot | Clears when unread counts drop after viewing | Header wired to separate unread store namespace | Yes (P1) |
