# Mobile Media Viewer Image Open Regression

- Topic: Mattermost Mobile gallery opens blank for file attachments in Rustchat.
- Date: 2026-02-13
- Scope: Mattermost-compatible post metadata contract for `metadata.files` used by mobile gallery.

## Compatibility Contract

1. `metadata.files` in post payloads must contain Mattermost-style `FileInfo` entries, including `width` and `height` for image files.
2. `has_preview_image` must reflect preview availability so mobile picks `/preview` vs `/files/{id}` correctly.
3. For image attachments, dimensions must not be dropped in post-list metadata mapping; otherwise mobile gallery renders a zero-size image.

## Evidence

- Mattermost server writes full file infos into post metadata: `../mattermost/server/channels/app/post_metadata.go:246` and `../mattermost/server/channels/app/post_metadata.go:250`.
- Mattermost `PostMetadata.Files` is `[]*FileInfo`: `../mattermost/server/public/model/post_metadata.go:19`.
- Mattermost `FileInfo` includes `width`, `height`, and `has_preview_image`: `../mattermost/server/public/model/file_info.go:53`, `../mattermost/server/public/model/file_info.go:54`, `../mattermost/server/public/model/file_info.go:55`.
- Mobile gallery computes render size from file dimensions and uses `{0,0}` when absent: `../mattermost-mobile/app/utils/images/index.ts:19`, `../mattermost-mobile/app/screens/gallery/renderers/image/index.tsx:37`.

## Rustchat Gap Summary

- Rustchat currently hardcodes `metadata.files[].width = 0` and `height = 0` in `PostResponse -> mm::Post` mapping, even when DB contains real dimensions: `backend/src/mattermost_compat/mappers.rs:178` and `backend/src/mattermost_compat/mappers.rs:179`.
- Rustchat already reads real dimensions in `populate_files` from `files` table but drops them from `FileUploadResponse`: `backend/src/services/posts.rs:343` and `backend/src/services/posts.rs:369`.
