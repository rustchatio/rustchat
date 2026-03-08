# Gap Plan

## Verified Gap

- `metadata.files[].width/height` are zeroed in Rustchat post mapper (`backend/src/mattermost_compat/mappers.rs:178` to `backend/src/mattermost_compat/mappers.rs:179`) despite DB dimensions being available in `populate_files` (`backend/src/services/posts.rs:343`).

## Implementation Tasks

1. Extend `FileUploadResponse` with `width` and `height`.
- Add serde defaults to keep backward compatibility with older serialized payloads.
- Completed in `backend/src/models/file.rs`.

2. Preserve dimensions in file population.
- Set `width`/`height` from DB `files.width`/`files.height` in `populate_files`.
- Completed in `backend/src/services/posts.rs`.

3. Preserve dimensions in Mattermost post metadata mapping.
- Replace hardcoded zeros with carried values in `impl From<PostResponse> for mm::Post`.
- Completed in `backend/src/mattermost_compat/mappers.rs`.

4. Add regression coverage.
- Unit test: `PostResponse -> mm::Post` must output `metadata.files[0].width/height` from source file payload.
- Completed by `test_post_response_file_dimensions_mapped_to_metadata` in `backend/src/mattermost_compat/mappers.rs`.

## Compatibility Checklist Status

- Request shape: N/A (read-path bug).
- Response shape: Completed. `metadata.files` now carries `width/height` from DB-backed file rows.
- Status/error semantics: No changes expected.
- Websocket semantics: Completed for payload shape. Events are unchanged; post payloads now include non-zero file dimensions when available.
- Tests: Completed.

## Test Evidence

- `cargo test test_post_response_file_dimensions_mapped_to_metadata` (pass)
- `cargo test mattermost_compat::mappers::tests` (pass)

## Residual Risks

- Historical files in DB with null dimensions will still produce `0`; mobile behavior for those remains unchanged.
- This patch addresses dimension propagation only, not unrelated call-thread regressions.
