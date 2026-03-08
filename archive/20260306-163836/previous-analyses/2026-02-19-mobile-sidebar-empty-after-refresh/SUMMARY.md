# Summary

- Topic: Mobile sidebar is empty after phone refresh (channels/DMs disappear until reconnect/reset).
- Date: 2026-02-19
- Scope: Mattermost Mobile startup sync path + Rustchat `/users/*/teams/{team_id}/channels/categories` behavior.

## Observed Failure Shape

- Mobile startup sync (`fetchMyChannelsForTeam`) requests three endpoints in parallel: channels, memberships, categories.
- Sidebar rendering depends on categories; when categories are empty/incomplete, channel list can render as empty.
- A websocket/network reset banner can appear independently of sidebar population timing.

## Compatibility Contract (derived from upstream)

1. Sidebar category reads must be self-healing:
- If categories are missing, server creates initial defaults.
- If category mappings are stale/incomplete, orphan user channels are still surfaced in Channels/DM buckets.

2. Team channel list and sidebar categories are coupled for mobile sync:
- Mobile startup expects channels + memberships + categories together.
- Categories must reference valid, user-visible channels for the active team (plus DM/GM semantics expected by client behavior).

3. Empty categories implies empty sidebar on mobile:
- Mobile does not synthesize fallback sidebar items when category set is empty.

## Rustchat Gap Found

- Rustchat category read returned persisted `channel_category_channels` as-is and did not backfill orphan channels.
- This allows stale category mappings to produce an empty sidebar even when user channel memberships exist.

## Implemented Fix

- Added Mattermost-style orphan backfill in Rustchat category reads:
  - Query candidate channels for user/team context.
  - Detect channels not assigned to any returned category.
  - Append unassigned public/private channels to Channels bucket and DM/GM to Direct Messages bucket (with safe fallbacks).
- Updated default category channel source to use the same candidate channel selection.

## Verification

- Unit: `backfills_orphaned_channels_into_default_buckets`
- Integration: `get_categories_backfills_orphaned_channels`
- Existing categories API test suite still passes.

## Residual Risk

- Full parity for team-channel endpoint filters (`include_deleted`, `last_delete_at`, MM teamless DM semantics) remains a separate compatibility task.
