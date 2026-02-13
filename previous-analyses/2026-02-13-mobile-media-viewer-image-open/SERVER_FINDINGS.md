# Server Findings (Mattermost)

## Observed Upstream Behavior

1. Post metadata file list is populated from server file infos.
- `preparePostFilesForClient` sets `post.Metadata.Files = fileInfos`: `../mattermost/server/channels/app/post_metadata.go:246` to `../mattermost/server/channels/app/post_metadata.go:251`.
- `PostMetadata.Files` type is `[]*FileInfo`: `../mattermost/server/public/model/post_metadata.go:19` to `../mattermost/server/public/model/post_metadata.go:20`.

2. File metadata contract includes image dimensions.
- `FileInfo` contains `Width`, `Height`, and `HasPreviewImage`: `../mattermost/server/public/model/file_info.go:53` to `../mattermost/server/public/model/file_info.go:56`.

## Rustchat Comparison

1. Rustchat post mapping for mobile-compatible responses does not preserve dimensions.
- In `impl From<PostResponse> for mm::Post`, each metadata file entry sets `width: 0` and `height: 0`: `backend/src/mattermost_compat/mappers.rs:178` to `backend/src/mattermost_compat/mappers.rs:179`.

2. Rustchat has real file dimensions available before mapping.
- `populate_files` fetches full `files` rows: `backend/src/services/posts.rs:343`.
- `FileUploadResponse` currently stores only id/name/mime/size/url/thumbnail_url, so dimensions are discarded before mapping: `backend/src/models/file.rs:30` to `backend/src/models/file.rs:38`.

## Contract-Relevant Conclusion

For Mattermost-mobile compatibility, Rustchat must carry DB `width`/`height` from `populate_files` into `metadata.files` mapping instead of hardcoding zeros.
