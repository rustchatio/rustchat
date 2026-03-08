# Mobile Findings

## Entry Flow Analysis

From `mattermost-mobile/app/actions/remote/entry/common.ts`:

### Entry Function (line 65)
```typescript
export const entry = async (serverUrl: string, teamId?: string, channelId?: string, since = 0, groupLabel?: RequestGroupLabel): Promise<EntryResponse> => {
    const {database} = DatabaseManager.getServerDatabaseAndOperator(serverUrl);
    const result = await entryRest(serverUrl, teamId, channelId, since, groupLabel);
    // ...
};
```

### entryRest Function (line 80)
Key API calls in order:
1. `fetchConfigAndLicense()` - Gets server config
2. `fetchMyPreferences()` - Gets user preferences
3. `fetchMyTeams()` - Gets teams user is member of
4. `fetchMe()` - Gets current user
5. `fetchMyChannelsForTeam()` - **CRITICAL** - Gets channels

### Channel Fetching (line 169-171)
```typescript
if (initialTeamId) {
    chData = await fetchMyChannelsForTeam(serverUrl, initialTeamId, false, lastDisconnectedAt, true, false, isCRTEnabled, groupLabel);
}
```

This is called with `lastDisconnectedAt` which is used for delta sync.

## WebSocket Reconnect Flow

From `mattermost-mobile/app/actions/websocket/index.ts`:

### doReconnect Function (line 50)
```typescript
async function doReconnect(serverUrl: string, groupLabel?: BaseRequestGroupLabel) {
    // ...
    const entryData = await entry(serverUrl, currentTeamId, currentChannelId, lastFullSync, groupLabel);
    // ...
    await fetchPostDataIfNeeded(serverUrl, groupLabel);
    // ...
    await deferredAppEntryActions(serverUrl, lastFullSync, currentUserId, currentUserLocale, prefData.preferences, config, license, teamData, chData, meData, initialTeamId, undefined, groupLabel);
    // ...
}
```

## Status Update Mechanism

From `mattermost-mobile/app/managers/websocket_manager.ts`:

### Periodic Status Updates (line 200-221)
```typescript
private startPeriodicStatusUpdates(serverUrl: string) {
    const getStatusForUsers = async () => {
        const database = DatabaseManager.serverDatabases[serverUrl];
        const currentUserId = await getCurrentUserId(database.database);
        const userIds = (await queryAllUsers(database.database).fetchIds()).filter((id) => id !== currentUserId);
        fetchStatusByIds(serverUrl, userIds);
    };

    currentId = setInterval(getStatusForUsers, General.STATUS_INTERVAL);
    this.statusUpdatesIntervalIDs[serverUrl] = currentId;
    getStatusForUsers();  // Called immediately
}
```

`General.STATUS_INTERVAL` = 5 minutes (300000ms)

### Status API
From `mattermost-mobile/app/client/rest/users.ts` line 415:
```typescript
getStatus = (userId: string) => {
    return this.doFetch<UserStatus>(
        `${this.getUserRoute(userId)}/status`,
        {method: 'get'},
    );
};
```

Expected response: `UserStatus` object

## Typing Events

From `mattermost-mobile/app/actions/websocket/users.ts` line 84:

### handleUserTypingEvent
```typescript
export async function handleUserTypingEvent(serverUrl: string, msg: WebSocketMessage) {
    // ...
    const data = {
        channelId: msg.broadcast.channel_id,  // <-- KEY: Uses broadcast.channel_id
        rootId: msg.data.parent_id,
        userId: msg.data.user_id,
        username,
        now: Date.now(),
    };
    DeviceEventEmitter.emit(Events.USER_TYPING, data);
    // ...
}
```

### Sending Typing
From line 117:
```typescript
export const userTyping = async (serverUrl: string, channelId: string, rootId?: string) => {
    const client = WebsocketManager.getClient(serverUrl);
    client?.sendUserTypingEvent(channelId, rootId);
};
```

## Expected Data Formats

### Status Object
From `mattermost-mobile/types/api/users.d.ts` line 116:
```typescript
interface UserStatus {
    user_id: string;
    status: string;
    manual: boolean;
    last_activity_at: number;
}
```

### WebSocket Message
From `mattermost-mobile/app/constants/websocket.ts`:
- `STATUS_CHANGED: 'status_change'`
- `TYPING: 'typing'`
- `STOP_TYPING: 'stop_typing'`

### User Typing Event Data
Expected fields in `msg.data`:
- `user_id`: string
- `parent_id`: string (for threads, empty for regular)

Expected fields in `msg.broadcast`:
- `channel_id`: string
- `user_id`: string (sender)
- `team_id`: string
