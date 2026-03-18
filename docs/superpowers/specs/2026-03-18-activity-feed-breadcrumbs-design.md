# Design: Activity Feed and Breadcrumbs Navigation

**Status**: Draft
**Created**: 2026-03-18
**Phase**: Phase 2 of WebUI Enhancement Initiative

---

## Problem Statement

The current RustChat navigation experience lacks modern collaboration tool patterns:

1. **No centralized activity view** - Users must check each channel individually to find mentions, replies, or reactions to their messages
2. **No visual hierarchy indicator** - Users lose context about which team/channel they're in, especially when deep in threads
3. **No quick navigation** - Switching channels requires multiple clicks through the sidebar, no keyboard shortcuts

These gaps reduce user efficiency and make the tool feel less polished than competitors (Slack, Discord, Teams).

---

## Goals

1. Provide an **Activity Feed** panel showing personalized recent activity (mentions, replies, reactions, DMs) with real-time updates
2. Add **Breadcrumbs** showing navigation path (Team > Channel > Thread) with clickable segments
3. Implement **Quick Switcher** (Cmd+K) for fuzzy-search navigation to channels, teams, and recent threads
4. Maintain **keyboard-first navigation** - everything accessible without mouse
5. Ensure **real-time updates** via WebSocket for new activity

---

## Non-Goals

1. Email/push notifications (Phase 3)
2. Custom notification rules per channel (Phase 3)
3. Search within activity feed (Phase 2b)
4. Activity filtering by date range (Phase 2b)

---

## Architecture

### High-Level UI Structure

```
┌─────────────────────────────────────────────────────────────────┐
│  [Logo]  TeamName ▼      🔍 Quick Switcher (Cmd+K)    [👤 User] │
├─────────────────────────────────────────────────────────────────┤
│           │                                                     │
│  Channels │  Breadcrumb: Engineering > #general > Thread        │
│  ─────────┤                                                     │
│  #general │  ┌─────────────────────────────────────────────┐   │
│  #random  │  │ Message content...                          │   │
│  #dev     │  │                                             │   │
│           │  └─────────────────────────────────────────────┘   │
│  [Activity│                                                  │   │
│   Feed]   │                                                  │   │
│           │                                                  │   │
│           │                                                  │   │
├───────────┴──────────────────────────────────────────────────┤   │
│  Composer                                                    │   │
└──────────────────────────────────────────────────────────────┘   │
                                                                  │
┌──────────────────────────────────────────┐                      │
│  Activity Feed (slide-over from right)   │◄─────────────────────┘
│  ┌────────────────────────────────────┐  │
│  │ 🔴 Activity (3 unread)      [×]   │  │
│  ├────────────────────────────────────┤  │
│  │ [@] Alice mentioned you          │  │
│  │     #general · 2 min ago         │  │
│  ├────────────────────────────────────┤  │
│  │ [💬] Bob replied to your thread  │  │
│  │     #dev · 15 min ago            │  │
│  ├────────────────────────────────────┤  │
│  │ [❤️] Carol reacted :heart:       │  │
│  │     #random · 1 hour ago         │  │
│  └────────────────────────────────────┘  │
└──────────────────────────────────────────┘

Quick Switcher Modal (Cmd+K):
┌─────────────────────────────────────────┐
│  > general                              │
│  ┌─────────────────────────────────────┐│
│  │ 🔍 "general"                        ││
│  ├─────────────────────────────────────┤│
│  │ #general (Engineering)        ↵     ││
│  │ #general-random (Design)      ↵     ││
│  │ @general-manager (User)       ↵     ││
│  ├─────────────────────────────────────┤│
│  │ Recent                              ││
│  │ #dev (Engineering)            ↵     ││
│  │ Thread: "Release planning"    ↵     ││
│  └─────────────────────────────────────┘│
└─────────────────────────────────────────┘
```

---

## Activity Feed

### Data Model

**ActivityType Enum:**
```typescript
enum ActivityType {
  MENTION = 'mention',      // Someone @mentioned you
  REPLY = 'reply',          // Someone replied to your message
  REACTION = 'reaction',    // Someone reacted to your message
  DM = 'dm',                // New direct message
  THREAD_REPLY = 'thread_reply'  // Reply in a thread you're following
}
```

**Activity Interface:**
```typescript
interface Activity {
  id: string;
  type: ActivityType;
  userId: string;           // Who triggered it
  userName: string;
  userAvatar?: string;
  channelId: string;
  channelName: string;
  teamId: string;
  teamName: string;
  postId: string;           // Link to the message
  rootId?: string;          // For thread replies
  message?: string;         // Snippet of the message
  reaction?: string;        // For reactions: emoji name
  createAt: number;         // Unix timestamp ms
  read: boolean;
}
```

### API Endpoints

**Get Activity Feed:**
```
GET /api/v4/users/{user_id}/activity

Query Parameters:
- cursor (optional): Pagination cursor
- limit (optional): Default 50, max 100
- type (optional): Filter by ActivityType (comma-separated)
- unread_only (optional): boolean

Response 200:
{
  "order": ["activity_1", "activity_2", ...],
  "activities": {
    "activity_1": {
      "id": "activity_1",
      "type": "mention",
      "user_id": "user_123",
      "user_name": "Alice",
      "channel_id": "channel_456",
      "channel_name": "general",
      "team_id": "team_789",
      "team_name": "Engineering",
      "post_id": "post_abc",
      "message": "@bob check this out...",
      "create_at": 1706000000000,
      "read": false
    }
  },
  "unread_count": 3,
  "next_cursor": "cursor_for_pagination"
}
```

**Mark Activity Read:**
```
POST /api/v4/users/{user_id}/activity/read

Request Body:
{
  "activity_ids": ["activity_1", "activity_2"]
}

Response 200: { "updated": 2 }
```

**Mark All Read:**
```
POST /api/v4/users/{user_id}/activity/read-all

Response 200: { "updated": 15 }
```

### Database

**New table `activities`:**
```sql
CREATE TABLE activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    type VARCHAR(20) NOT NULL CHECK (type IN ('mention', 'reply', 'reaction', 'dm', 'thread_reply')),
    actor_id UUID NOT NULL REFERENCES users(id),  -- Who triggered it
    channel_id UUID NOT NULL REFERENCES channels(id),
    team_id UUID NOT NULL REFERENCES teams(id),
    post_id UUID NOT NULL REFERENCES posts(id),
    root_id UUID REFERENCES posts(id),  -- For thread replies
    message_text TEXT,  -- Snippet, truncated
    reaction VARCHAR(50),  -- For reactions
    read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_activities_user_created ON activities(user_id, created_at DESC);
CREATE INDEX idx_activities_user_read ON activities(user_id, read) WHERE read = FALSE;
CREATE INDEX idx_activities_post ON activities(post_id);
```

### Component Architecture

**ActivityFeed.vue** - Slide-over panel (similar to ThreadPanel)
```typescript
const props = defineProps<{
  isOpen: boolean;
}>();

// Features:
// - Slide from right, fixed width 400px
// - Real-time badge count in header
// - "Mark all read" button
// - Empty state: "No new activity"
// - Group by date: "Today", "Yesterday", "Earlier"
```

**ActivityItem.vue** - Individual activity row
```typescript
const props = defineProps<{
  activity: Activity;
}>();

// Features:
// - Icon based on type (@, 💬, ❤️, ✉️)
// - Click navigates to relevant message
// - Hover shows relative time tooltip
// - Unread indicator (left border)
// - Right-click: "Mark read", "Dismiss"
```

**ActivityIcon.vue** - Type-specific icon component
```typescript
const icons: Record<ActivityType, string> = {
  mention: 'AtSign',
  reply: 'MessageCircle',
  reaction: 'Heart',
  dm: 'Mail',
  thread_reply: 'MessageSquare'
};
```

**ActivityFilters.vue** - Filter tabs
```typescript
const filters = [
  { label: 'All', value: null },
  { label: 'Mentions', value: 'mention' },
  { label: 'Replies', value: 'reply,thread_reply' },
  { label: 'Reactions', value: 'reaction' },
  { label: 'DMs', value: 'dm' }
];
```

### Store: activityStore.ts

```typescript
interface ActivityState {
  activities: Activity[];
  unreadCount: number;
  hasMore: boolean;
  cursor: string | null;
  filter: ActivityType | null;
  isLoading: boolean;
  isOpen: boolean;
}

const actions = {
  openFeed(): void;
  closeFeed(): void;
  loadActivities(refresh?: boolean): Promise<void>;
  loadMore(): Promise<void>;
  markRead(activityId: string): Promise<void>;
  markAllRead(): Promise<void>;
  onNewActivity(activity: Activity): void;  // WebSocket handler
  setFilter(type: ActivityType | null): void;
};
```

### Real-Time Updates

**WebSocket Events:**
- `activity` - New activity item (broadcast to affected user only)
- `activity_read` - Activity marked as read (for multi-device sync)

**Frontend Handler:**
```typescript
// handlers/activitySocketHandlers.ts
socket.on('activity', (activity: Activity) => {
  activityStore.onNewActivity(activity);
  // Play subtle notification sound if enabled
  // Show toast notification if permission granted
});
```

---

## Breadcrumbs

### Breadcrumb Levels

```typescript
interface BreadcrumbSegment {
  label: string;
  icon?: string;
  to?: RouteLocationRaw;
  isClickable: boolean;
}

// Examples:
// Channel view:  [Engineering] > [#general]
// Thread view:   [Engineering] > [#general] > [Thread]
// DM view:       [Direct Messages] > [@Alice]
```

### BreadcrumbBar.vue

```typescript
const props = defineProps<{
  segments: BreadcrumbSegment[];
}>();

// Features:
// - Responsive: truncate middle on small screens
// - Separator: chevron icon between segments
// - Last segment not clickable (current location)
// - Team segment shows team icon or initials
```

### Integration with Current View

```typescript
// ChannelView.vue
const breadcrumbs = computed(() => [
  { label: currentTeam.value?.name, to: `/teams/${teamId}`, icon: 'Users' },
  { label: `#${currentChannel.value?.name}`, isClickable: false }
]);

// When thread open:
const breadcrumbs = computed(() => [
  { label: currentTeam.value?.name, to: `/teams/${teamId}`, icon: 'Users' },
  { label: `#${currentChannel.value?.name}`, to: `/channels/${channelId}` },
  { label: 'Thread', isClickable: false }
]);
```

---

## Quick Switcher

### QuickSwitcherModal.vue

```typescript
// Keyboard shortcut: Cmd+K (Mac) / Ctrl+K (Windows/Linux)
// ESC or click outside to close

interface QuickSwitcherItem {
  id: string;
  type: 'channel' | 'dm' | 'thread' | 'team';
  name: string;
  subtitle?: string;  // Team name for channels, preview for threads
  icon: string;
  to: RouteLocationRaw;
}

const items = computed(() => [
  // Channels from all teams
  ...allChannels.value.map(c => ({
    type: 'channel',
    name: c.name,
    subtitle: c.teamName,
    icon: c.type === 'public' ? 'Hash' : 'Lock',
    to: `/channels/${c.id}`
  })),
  // Recent DMs
  ...recentDMs.value.map(dm => ({
    type: 'dm',
    name: dm.userName,
    icon: 'User',
    to: `/dm/${dm.userId}`
  })),
  // Recent threads
  ...recentThreads.value.map(t => ({
    type: 'thread',
    name: truncate(t.preview, 30),
    subtitle: t.channelName,
    icon: 'MessageSquare',
    to: `/channels/${t.channelId}?thread=${t.rootId}`
  }))
]);
```

### Fuzzy Search

```typescript
import { matchSorter } from 'match-sorter';

const filteredItems = computed(() => {
  if (!query.value) return recentItems.value;  // Show recent when no query
  return matchSorter(items.value, query.value, {
    keys: ['name', 'subtitle'],
    threshold: matchSorter.rankings.CONTAINS
  });
});
```

### Keyboard Navigation

```typescript
// Arrow up/down: navigate list
// Enter: select item
// ESC: close
// Cmd+K: toggle open/close

const selectedIndex = ref(0);

const onKeydown = (e: KeyboardEvent) => {
  if (e.key === 'ArrowDown') {
    selectedIndex.value = (selectedIndex.value + 1) % filteredItems.value.length;
  } else if (e.key === 'ArrowUp') {
    selectedIndex.value = (selectedIndex.value - 1 + filteredItems.value.length) % filteredItems.value.length;
  } else if (e.key === 'Enter') {
    selectItem(filteredItems.value[selectedIndex.value]);
  }
};
```

### API Endpoint

```
GET /api/v4/users/{user_id}/recent-items

Response 200:
{
  "channels": [...],
  "dms": [...],
  "threads": [...]
}
```

---

## Error Handling

| Scenario | User Experience | Implementation |
|----------|----------------|----------------|
| Activity feed fails to load | "Failed to load activity" with retry button | Error state + retry action |
| Mark read fails | Silent failure, retry on next open | Background retry queue |
| Quick switcher empty | "Start typing to search" | Placeholder state |
| New activity while viewing | Increment badge, flash header | WebSocket handler |

---

## Accessibility

**Activity Feed:**
- `aria-label="Activity feed"` on panel
- `aria-live="polite"` for new activity announcements
- Focus trap when open

**Breadcrumbs:**
- `aria-label="Breadcrumb"` on nav element
- `aria-current="location"` on last segment

**Quick Switcher:**
- `role="dialog"` with `aria-modal="true"`
- `aria-activedescendant` for list navigation
- Announces "N results found" on search

---

## Testing Strategy

### Unit Tests

| Component | Coverage |
|-----------|----------|
| ActivityFeed.vue | Open/close, filter changes, mark all read |
| ActivityItem.vue | Click navigation, mark read, type icons |
| BreadcrumbBar.vue | Render segments, click navigation |
| QuickSwitcherModal.vue | Search, keyboard nav, selection |
| activityStore.ts | Actions, WebSocket handling |

### Integration Tests

```typescript
test('new mention appears in activity feed', async () => {
  // Open activity feed
  // Simulate WebSocket mention
  // Verify appears at top
  // Verify unread count increments
});

test('quick switcher navigates to channel', async () => {
  // Open quick switcher (Cmd+K)
  // Type channel name
  // Press Enter
  // Verify navigation
});
```

### E2E Tests

```typescript
test('activity feed workflow', async ({ page }) => {
  await page.goto('/channels/general');
  // Click activity feed button
  // Verify panel opens
  // Click activity item
  // Verify navigated to message
});
```

---

## Migration Plan

**Phase 2a: Activity Feed (5 days)**
- Day 1: Database schema + backend API
- Day 2: Activity generation hooks (mention, reply, reaction)
- Day 3: Frontend store + service layer
- Day 4: ActivityFeed component + real-time updates
- Day 5: Testing + polish

**Phase 2b: Breadcrumbs + Quick Switcher (4 days)**
- Day 1: BreadcrumbBar component + integration
- Day 2: Quick Switcher modal + search
- Day 3: Recent items API + keyboard shortcuts
- Day 4: Testing + accessibility audit

**Total: 9 days**

---

## Success Criteria

- [ ] Activity feed opens in <100ms
- [ ] New activity appears in real-time (<200ms)
- [ ] Unread badge updates correctly
- [ ] Breadcrumbs render correctly for all views
- [ ] Quick switcher opens with Cmd+K, searches in <50ms
- [ ] All keyboard navigation works
- [ ] Accessible (axe-core passing)

---

## Related Documents

- Phase 1 Design: `docs/superpowers/specs/2026-03-18-message-threads-formatting-toolbar-design.md`
- API Documentation: `docs/architecture.md`
