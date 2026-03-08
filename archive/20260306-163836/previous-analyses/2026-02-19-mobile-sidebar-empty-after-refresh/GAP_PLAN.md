# Gap Plan

## Gap 1: Sidebar categories were not self-healing
- Rustchat target path: `backend/src/api/v4/users/sidebar_categories.rs`
- Required behavior: Match Mattermost category read semantics so orphan channels still appear in sidebar.
- Previous gap: Returned only persisted `channel_category_channels`; stale mappings could yield empty sidebar.
- Change implemented:
  - Added candidate channel query for user/team context.
  - Added orphan backfill into category payload before response.
  - Updated default category channel source to same candidate query.
- Verification tests:
  - Unit: `api::v4::users::sidebar_categories::tests::backfills_orphaned_channels_into_default_buckets`
  - Integration: `backend/tests/api_categories.rs::get_categories_backfills_orphaned_channels`
- Status: Completed

## Gap 2: Regression coverage for stale category data
- Rustchat target path: `backend/tests/api_categories.rs`
- Required behavior: Ensure an existing category with missing channel mappings does not produce empty sidebar payload.
- Change implemented: Added dedicated integration test that inserts a broken category row and validates backfilled channel IDs in response.
- Verification command:
  - `cargo test --manifest-path backend/Cargo.toml --test api_categories -- --nocapture`
- Status: Completed

## Remaining compatibility work (separate follow-up)
- Team channel endpoint parity (`include_deleted`, `last_delete_at`, and strict MM teamless-DM behavior) is not fully covered in this patch.
- Recommended follow-up: add endpoint-contract tests against Mattermost semantics for `/users/me/teams/{team_id}/channels` and `/users/me/teams/{team_id}/channels/members`.
