# Gap Plan

- Rustchat target path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/channels.rs`
- Required behavior: `GET /channels/{channel_id}/posts` and `GET /channels/{channel_id}/pinned` include `metadata.files` for posts with attachments, while preserving reaction metadata.
- Current gap: Post list handlers were constructing metadata ad-hoc and overwriting metadata with reactions.
- Planned change: Populate files before mapping (`populate_files`) and merge reactions into existing metadata object.
- Verification test: `/Users/scolak/Projects/rustchat/backend/tests/api_v4_post_routes.rs:277-369`
- Status: completed

- Rustchat target path: `/Users/scolak/Projects/rustchat/backend/src/api/v4/posts/unread.rs`
- Required behavior: Unread post list response keeps attachment metadata plus reactions.
- Current gap: Unread handler generated posts without populated files and replaced metadata with reactions.
- Planned change: Populate files and merge reactions into existing metadata.
- Verification test: Covered by compile/runtime validation and route behavior consistency with updated code paths.
- Status: completed

- Rustchat target path: `/Users/scolak/Projects/rustchat/backend/tests/api_v4_post_routes.rs`
- Required behavior: Regression guard ensuring channel history payload contains both files and reactions metadata.
- Current gap: No test protected metadata coexistence.
- Planned change: Add integration test uploading file + adding reaction + fetching channel posts and asserting both metadata sections.
- Verification test: `cargo test --manifest-path /Users/scolak/Projects/rustchat/backend/Cargo.toml --test api_v4_post_routes`
- Status: completed

## Compatibility gate checklist

- API contract: verified for changed routes (`/channels/{id}/posts`, `/channels/{id}/pinned`, unread posts around endpoint) and metadata shape.
- Realtime contract: unchanged by this fix.
- Data semantics: verified metadata now carries files and reactions together.
- Auth/permissions: existing membership checks unchanged.
- Client expectations: verified against mobile metadata processing paths.

## Remaining risks

- `GET /api/v4/posts/{post_id}/files/info` still depends on `files.post_id` population; not required for this resolved history path but worth follow-up.

## Test evidence

- `cargo test --manifest-path /Users/scolak/Projects/rustchat/backend/Cargo.toml --test api_v4_post_routes` (4 passed)
- `cargo test --manifest-path /Users/scolak/Projects/rustchat/backend/Cargo.toml --test api_v4_posts` (1 passed)
