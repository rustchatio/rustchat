# Mobile Findings

- Screen, store, or service: Channel history fetch
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/client/rest/posts.ts`
- Source lines: `92-117`
- Observed behavior: Mobile message logs are fetched from `/channels/{channel_id}/posts` (including since/before/after variants).
- Notes: Re-login/history visibility depends on this payload.

- Screen, store, or service: Post metadata ingestion for files
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/database/operator/server_data_operator/handlers/post.ts`
- Source lines: `327-330`, `396-399`
- Observed behavior: Mobile extracts files from `post.metadata.files` and persists them via `handleFiles`.
- Notes: If `metadata.files` is missing, file rows are not created from history payload.

- Screen, store, or service: File cleanup during post sync
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/database/operator/server_data_operator/handlers/post.ts`
- Source lines: `335`, `402-406`
- Observed behavior: Sync tracks `file_ids` and can remove file rows not present in received IDs.
- Notes: Missing metadata combined with sync updates can leave posts without attachment records.

- Screen, store, or service: Metadata includes both reactions and files
- Source path: `/Users/scolak/Projects/mattermost-mobile/app/database/operator/server_data_operator/handlers/post.ts`
- Source lines: `315-330`
- Observed behavior: Reactions and files are read from separate metadata keys in the same payload.
- Notes: Server must not replace full metadata object when adding reactions.
