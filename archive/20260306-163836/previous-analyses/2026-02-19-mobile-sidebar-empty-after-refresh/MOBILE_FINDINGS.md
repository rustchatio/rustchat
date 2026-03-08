# Mobile Findings

## Mattermost Mobile data flow

### 1) Startup sync requires channels + memberships + categories together
- Path: `../mattermost-mobile/app/actions/remote/channel.ts`
- Lines: 495-499
- Behavior: `fetchMyChannelsForTeam` uses `Promise.all` on:
  - `getMyChannels(teamId, includeDeleted, since)`
  - `getMyChannelMembers(teamId)`
  - `getCategories('me', teamId)`

### 2) Categories are persisted as authoritative sidebar structure
- Path: `../mattermost-mobile/app/actions/remote/channel.ts`
- Lines: 516-519
- Behavior: categories are stored with re-sync/prune path (`storeCategories(..., true, true)`).

### 3) Empty categories => empty sidebar items
- Path: `../mattermost-mobile/app/screens/home/channel_list/categories_list/categories/helpers/observe_flattened_categories.ts`
- Lines: 206-208
- Behavior: if category collection is empty, observable emits empty item list.

### 4) Team channel request contract
- Path: `../mattermost-mobile/app/client/rest/channels.ts`
- Lines: 223-228
- Behavior: team channels endpoint includes query params `include_deleted` and `last_delete_at`.

## Connection banner note

- Path: `../mattermost-mobile/app/components/connection_banner/use_connection_banner.ts`
- Lines: 62-71
- Behavior: “Unable to connect to network” is driven by websocket state transitions and shown after initial session conditions; this is related but not equivalent to sidebar data population.

## Interpretation for reported screenshots

- Screenshot `v1` (blank channels/DMs) is consistent with empty/incomplete categories in local/server sync state.
- Screenshot `v2` (connection banner + channels shown) is consistent with websocket instability occurring while category/channel data becomes available later.
