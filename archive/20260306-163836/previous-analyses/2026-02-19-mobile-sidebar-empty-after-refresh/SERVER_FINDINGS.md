# Server Findings

## Mattermost Evidence

### 1) Category init on empty response
- Path: `../mattermost/server/channels/app/channel_category.go`
- Lines: 26-35
- Behavior: `GetSidebarCategoriesForTeamForUser` creates initial sidebar categories when category set is empty.

### 2) Orphan-channel backfill on category reads
- Path: `../mattermost/server/channels/store/sqlstore/channel_store_categories.go`
- Lines: 356-390 (`completePopulatingCategoriesT`)
- Behavior: Any user channel missing from explicit sidebar mappings is appended to Channels/DM categories.

- Path: `../mattermost/server/channels/store/sqlstore/channel_store_categories.go`
- Lines: 398-450 (`getOrphanedSidebarChannels`)
- Behavior: Finds user channels not represented in sidebar mappings, split by public/private vs direct/group semantics.

- Path: `../mattermost/server/channels/store/sqlstore/channel_store_categories.go`
- Lines: 490-536
- Behavior: `getSidebarCategoriesT` always invokes `completePopulatingCategoriesT` before returning.

### 3) Team channel query includes teamless channels
- Path: `../mattermost/server/channels/store/sqlstore/channel_store.go`
- Lines: 1076-1080
- Behavior: Team-scoped channel query includes `TeamId = teamId OR TeamId = ''`.

## Rustchat Evidence

### 1) Prior category-read behavior lacked orphan backfill
- Path: `backend/src/api/v4/users/sidebar_categories.rs`
- Pre-fix behavior (same function area now fixed): category IDs were read from `channel_category_channels` only, without reconciliation to current channel memberships.

### 2) Implemented Rustchat fix
- Path: `backend/src/api/v4/users/sidebar_categories.rs`
- Lines: 117-187
- Behavior: category read now tracks assigned channel IDs and backfills orphan channels before returning.

- Path: `backend/src/api/v4/users/sidebar_categories.rs`
- Lines: 265-336
- Behavior: candidate-channel query and backfill routing (public/private -> Channels, direct/group -> DMs/fallback).

- Path: `backend/src/api/v4/users/sidebar_categories.rs`
- Lines: 366-377
- Behavior: default categories pull channel IDs from the same candidate-channel source.

## Interpretation

- Mattermost actively prevents stale mappings from producing an empty sidebar.
- Rustchat previously trusted persisted mappings too strictly; this is sufficient to explain empty sidebar state after refresh/reconnect sequences when mappings drift.
