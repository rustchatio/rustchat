# Design: Message Threads View and Formatting Toolbar

**Status**: Draft
**Created**: 2026-03-18
**Phase**: Phase 1 of WebUI Enhancement Initiative

---

## Problem Statement

The current RustChat messaging experience has significant gaps compared to modern collaboration tools (Slack, Discord, Teams):

1. **Threads are displayed inline** within the channel view, making it hard to follow long conversations without losing context of the main channel
2. **No formatting assistance** in the composer - users must know markdown syntax for any styling
3. **Limited emoji reactions** - no visual picker, no custom emoji support
4. **Poor discoverability** of mentions and rich content features

These gaps reduce user engagement and make the tool feel less polished than competitors.

---

## Goals

1. Provide a **dedicated thread panel** that slides in from the right (Slack-style), keeping channel context visible while focusing on the conversation
2. Add a **formatting toolbar** above the composer with WYSIWYG-style formatting (bold, italic, code, links, lists)
3. Maintain **backward compatibility** with existing markdown-based message storage
4. Support **real-time updates** in threads via WebSocket
5. Ensure **accessibility** (keyboard navigation, screen readers)

---

## Non-Goals

1. Custom emoji uploads (Phase 2)
2. Rich text editing in existing messages (edit remains plain text)
3. Message scheduling (Phase 2)
4. Huddles/audio rooms (Phase 3)

---

## Architecture

### High-Level UI Structure

```
┌─────────────────────────────────────────────────────────┐
│  Channel View                    │  Thread Panel        │
│                                  │  (slides from right) │
│  ┌─────────────────────────┐     │  ┌──────────────┐    │
│  │ Parent message          │     │  │ Thread       │    │
│  │ "Check this out..."     │◄────┼──┤ header       │    │
│  │ [View 12 replies]       │─────┘  │              │    │
│  └─────────────────────────┘        │  ┌──────────┐│    │
│                                     │  │Reply 1   ││    │
│  ┌─────────────────────────┐        │  │Reply 2   ││    │
│  │ Another message         │        │  │Reply 3   ││    │
│  └─────────────────────────┘        │  └──────────┘│    │
│                                     │  ┌──────────┐│    │
│                                     │  │Composer  ││    │
│                                     │  │+ Toolbar ││    │
│                                     │  └──────────┘│    │
│                                     └──────────────┘    │
└─────────────────────────────────────────────────────────┘
```

### Component Hierarchy

```
ChannelView.vue
├── MessageList.vue
│   └── MessageItem.vue
│       └── [NEW] ThreadPreview.vue ("View 12 replies" button)
├── [NEW] ThreadPanel.vue (slide-over panel)
│   ├── ThreadHeader.vue (parent message + close button)
│   ├── ThreadReplyList.vue (scrollable replies)
│   │   └── ThreadReplyItem.vue (individual reply)
│   └── ThreadComposer.vue (reply composer)
│       └── [NEW] FormattingToolbar.vue
└── Composer.vue (main channel composer)
    └── [NEW] FormattingToolbar.vue
```

### Data Flow

**Opening a Thread:**
1. User clicks "View replies" on a message with `reply_count > 0`
2. `threadStore.openThread(postId)` called
3. API request: `GET /api/v4/posts/{post_id}/thread`
4. Thread panel slides in with parent + replies
5. WebSocket subscription established for `root_id = postId`

**Sending a Reply:**
1. User types in thread composer, clicks send
2. `POST /api/v4/posts` with `root_id: parentId`
3. Optimistic UI update (show reply immediately)
4. WebSocket broadcasts to all thread viewers
5. On success: clear composer, on error: show retry

**Real-time Updates:**
- WebSocket `posted` event with `root_id` → append to thread if matching
- WebSocket `post_deleted` → remove from thread or show "deleted"
- WebSocket `reaction_added/removed` → update reaction counts

---

## API Changes

### New Endpoints

**Get Thread**
```
GET /api/v4/posts/{post_id}/thread

Query Parameters:
- cursor (optional): Pagination cursor for large threads
- limit (optional): Number of replies to fetch (default: 50, max: 100)

Response 200:
{
  "order": ["parent_id", "reply_1_id", "reply_2_id", ...],
  "posts": {
    "parent_id": {
      "id": "parent_id",
      "user_id": "user_123",
      "message": "Original message",
      "create_at": 1706000000,
      "reply_count": 5,
      "last_reply_at": 1706000100,
      "participants": ["user_123", "user_456"],
      "metadata": { ... }
    },
    "reply_1_id": {
      "id": "reply_1_id",
      "user_id": "user_456",
      "message": "Reply message",
      "root_id": "parent_id",
      "parent_id": "parent_id",
      "create_at": 1706000050
    }
  },
  "next_cursor": "cursor_for_pagination"
}
```

### Modified Endpoints

**Create Post** (existing)
```
POST /api/v4/posts

Request Body (addition):
{
  "channel_id": "channel_id",
  "message": "Reply text",
  "root_id": "parent_post_id",  // NEW: for thread replies
  "parent_id": "parent_post_id" // NEW: for thread replies
}
```

### Database

**No schema changes required.** The existing `posts` table structure supports threads:

```sql
-- Existing columns used for threads:
-- root_id: UUID (nullable) - top-level parent of the thread
-- parent_id: UUID (nullable) - immediate parent (for nested, though we use flat)

-- Recommended index for performance:
CREATE INDEX idx_posts_thread ON posts(root_id, create_at) WHERE root_id IS NOT NULL;
```

---

## Frontend Implementation

### New Components

#### ThreadPanel.vue
```typescript
// State managed by threadStore
const props = defineProps<{
  isOpen: boolean;
  parentPostId: string | null;
}>();

// Features:
// - Slide transition from right (CSS transform)
// - Fixed width: 400px on desktop, 100% on mobile
// - Close on Escape key, close button, or click outside
// - Virtual scrolling for threads > 50 replies
```

#### FormattingToolbar.vue
```typescript
// Uses TipTap editor commands
const toolbarItems = [
  { icon: 'Bold', command: 'toggleBold', shortcut: 'Mod+b' },
  { icon: 'Italic', command: 'toggleItalic', shortcut: 'Mod+i' },
  { icon: 'Strikethrough', command: 'toggleStrike', shortcut: 'Mod+Shift+x' },
  { type: 'separator' },
  { icon: 'Code', command: 'toggleCode', shortcut: 'Mod+e' },
  { icon: 'Code2', command: 'toggleCodeBlock', shortcut: 'Mod+Shift+c' },
  { type: 'separator' },
  { icon: 'Link', command: 'setLink', shortcut: 'Mod+k' },
  { icon: 'List', command: 'toggleBulletList' },
  { icon: 'ListOrdered', command: 'toggleOrderedList' },
  { type: 'separator' },
  { icon: 'AtSign', command: 'insertMention', label: 'Mention' },
  { icon: 'Hash', command: 'insertChannelMention', label: 'Channel' },
  { icon: 'Smile', command: 'openEmojiPicker', label: 'Emoji' }
];
```

#### RichTextEditor.vue
```typescript
// Wraps TipTap editor
// Props: modelValue (markdown string), placeholder
// Emits: update:modelValue (markdown), submit (on Enter)
// Features:
// - Markdown input/output
// - Mention autocomplete (@)
// - Emoji autocomplete (:)
// - Keyboard shortcuts
```

### Store Updates

#### threadStore.ts
```typescript
interface ThreadState {
  isOpen: boolean;
  parentPostId: string | null;
  parentPost: Post | null;
  replies: Post[];
  hasMore: boolean;
  cursor: string | null;
  isLoading: boolean;
  draft: string; // unsent reply
}

const actions = {
  openThread(postId: string): Promise<void>;
  closeThread(): void;
  loadMoreReplies(): Promise<void>;
  sendReply(message: string): Promise<void>;
  onNewReply(reply: Post): void; // WebSocket handler
};
```

### Dependencies

```json
{
  "@tiptap/vue-3": "^2.2.0",
  "@tiptap/starter-kit": "^2.2.0",
  "@tiptap/extension-mention": "^2.2.0",
  "@tiptap/extension-placeholder": "^2.2.0",
  "@tiptap/extension-link": "^2.2.0"
}
```

---

## Backend Implementation

### New Service Methods

```rust
// src/services/posts.rs

pub async fn get_thread(
    &self,
    post_id: Uuid,
    cursor: Option<Uuid>,
    limit: i64,
) -> Result<ThreadResponse, AppError> {
    // Fetch parent post
    // Fetch replies with cursor pagination
    // Return ordered map
}

pub struct ThreadResponse {
    pub order: Vec<Uuid>,
    pub posts: HashMap<String, Post>,
    pub next_cursor: Option<String>,
}
```

### Handler

```rust
// src/api/v4/posts.rs

async fn get_thread(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
    Query(params): Query<ThreadQueryParams>,
    auth: AuthUser,
) -> Result<Json<ThreadResponse>, AppError> {
    // Verify user has access to channel containing post
    // Call service.get_thread()
    // Return response
}
```

---

## Error Handling

| Scenario | User Experience | Implementation |
|----------|----------------|----------------|
| Thread not found | "Thread not found" message in panel, auto-close after 3s | 404 error → clear thread state |
| Permission denied | Close panel, redirect to channel, toast notification | 403 error → navigation + toast |
| Parent deleted while viewing | Header shows "Message deleted", composer disabled | WebSocket `post_deleted` handler |
| Network error loading | Retry button, 3 automatic retries | Repository retry logic |
| Send reply fails | Inline error, preserve draft in localStorage | Draft key: `thread_draft_{root_id}` |
| Very large thread (>1000) | "Load more" button, virtual scrolling | Cursor pagination, 50 items/page |

---

## Accessibility

**Thread Panel:**
- `aria-label="Thread panel"` on container
- `role="dialog"` with `aria-modal="true"`
- Focus trap when open (Tab cycles within panel)
- Escape key closes panel
- Focus returns to "View replies" button on close

**Formatting Toolbar:**
- Each button has `aria-label` describing action
- Keyboard shortcuts announced via `aria-live`
- High contrast focus indicators

**Screen Reader Announcements:**
- "Thread opened, 12 replies" on open
- "New reply from Alice" when WebSocket update arrives
- "Message sent" on successful reply

---

## Testing Strategy

### Unit Tests

| Component | Coverage |
|-----------|----------|
| ThreadPanel.vue | Open/close, parent display, reply rendering, pagination |
| FormattingToolbar.vue | Command execution, keyboard shortcuts, state sync |
| RichTextEditor.vue | Markdown I/O, mention autocomplete, emoji autocomplete |
| threadStore.ts | State transitions, async actions, WebSocket handling |

### Integration Tests

```typescript
// Thread workflow
test('open thread, send reply, see real-time update', async () => {
  // Open thread
  // Type reply
  // Send
  // Verify optimistic update
  // Simulate WebSocket
  // Verify persistent state
});

test('formatting toolbar produces correct markdown', async () => {
  // Click bold, type text
  // Click italic
  // Verify output: "**bold***italic*"
});
```

### E2E Tests (Playwright)

```typescript
test('complete thread workflow', async ({ page }) => {
  await page.goto('/channels/general');

  // Open thread
  await page.click('[data-testid="view-replies"]:first-child');
  await expect(page.locator('[data-testid="thread-panel"]')).toBeVisible();

  // Send reply with formatting
  await page.click('[data-testid="toolbar-bold"]');
  await page.fill('[data-testid="thread-composer"]', 'Bold reply');
  await page.click('[data-testid="send-reply"]');

  // Verify rendered output
  await expect(page.locator('[data-testid="thread-reply"] strong'))
    .toContainText('Bold reply');
});
```

### Backend Tests

```rust
#[tokio::test]
async fn test_get_thread_success() {
    let (app, db) = setup().await;
    let parent = create_post(&db, "Parent").await;
    let reply = create_reply(&db, &parent.id, "Reply").await;

    let response = get_thread(&app, &parent.id, None).await;

    assert_eq!(response.status(), StatusCode::OK);
    let thread: ThreadResponse = response.json().await;
    assert_eq!(thread.order.len(), 2);
    assert!(thread.posts.contains_key(&parent.id.to_string()));
}

#[tokio::test]
async fn test_get_thread_pagination() {
    // Create 60 replies
    // Fetch with limit=50
    // Verify next_cursor present
    // Fetch next page
    // Verify correct items
}
```

---

## Migration Plan

**Phase 1.1: Backend (3 days)**
- Day 1: Thread endpoint implementation
- Day 2: Pagination + testing
- Day 3: WebSocket integration for real-time updates

**Phase 1.2: Frontend - Thread Panel (4 days)**
- Day 1: ThreadPanel component shell + store
- Day 2: Thread reply list + pagination
- Day 3: Thread composer + draft persistence
- Day 4: WebSocket handlers + polish

**Phase 1.3: Frontend - Formatting Toolbar (4 days)**
- Day 1: TipTap integration + basic toolbar
- Day 2: Markdown serialization/deserialization
- Day 3: Mention/emoji autocomplete integration
- Day 4: Main composer integration + testing

**Phase 1.4: Testing & Polish (3 days)**
- Unit tests
- E2E tests
- Accessibility audit
- Performance optimization

**Total: 14 days (2 weeks)**

---

## Success Criteria

- [ ] Thread panel opens in <200ms for threads <50 replies
- [ ] Replies appear in real-time (<100ms from WebSocket to render)
- [ ] Formatting toolbar produces valid markdown for all operations
- [ ] No regressions in existing message sending/receiving
- [ ] All new components have >80% test coverage
- [ ] Passes accessibility audit (axe-core)
- [ ] Works on mobile (responsive thread panel)
- [ ] Backward compatible with existing messages

---

## Future Enhancements (Phase 2+)

1. **Custom Emoji Uploads** - Store custom emoji metadata, upload to S3
2. **Message Reactions** - Rich reaction picker with custom emoji
3. **Thread Notification Preferences** - Mute/unmute specific threads
4. **Thread Search** - Search within thread context
5. **Rich Embeds** - Link previews, image galleries, video players

---

## Related Documents

- Original analysis: `docs/architecture.md`
- WebSocket architecture: `docs/websocket_architecture.md`
- Frontend migration guide: `frontend/MIGRATION_GUIDE.md`
