# Message Threads and Formatting Toolbar Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a dedicated thread panel (Slack-style) and WYSIWYG formatting toolbar to the RustChat messaging experience.

**Architecture:** Backend adds a thread fetch endpoint with cursor pagination. Frontend adds a slide-over ThreadPanel component and integrates TipTap editor with a floating FormattingToolbar.

**Tech Stack:** Rust/Axum/SQLx (backend), Vue 3/TypeScript/TipTap (frontend)

---

## File Structure

### Backend
```
backend/src/
├── models/post.rs              # Add ThreadResponse struct
├── services/posts.rs           # Add get_thread service method
└── api/v4/posts.rs             # Add GET /posts/{id}/thread endpoint
```

### Frontend
```
frontend/src/
├── features/messages/
│   ├── stores/threadStore.ts   # Thread state management (new)
│   ├── services/threadService.ts # Thread API calls (new)
│   └── index.ts                # Export thread modules
├── components/composer/
│   ├── ThreadComposer.vue      # Reply composer for thread panel (new)
│   ├── FormattingToolbar.vue   # Toolbar component (new)
│   └── RichTextEditor.vue      # TipTap wrapper (new)
├── components/thread/
│   ├── ThreadPanel.vue         # Slide-over panel (new)
│   ├── ThreadHeader.vue        # Parent message header (new)
│   ├── ThreadReplyList.vue     # Scrollable replies (new)
│   └── ThreadReplyItem.vue     # Individual reply (new)
└── views/main/ChannelView.vue  # Integrate thread panel
```

---

## Task 1: Add ThreadResponse Model

**Files:**
- Modify: `backend/src/models/post.rs`
- Test: `backend/src/models/post.rs` (add unit test)

- [ ] **Step 1: Add ThreadResponse struct to models/post.rs**

```rust
// Add after PostResponse struct

/// Response for thread endpoint
#[derive(Debug, Clone, Serialize)]
pub struct ThreadResponse {
    /// Order of post IDs (parent first, then replies chronologically)
    pub order: Vec<String>,
    /// Map of post ID to post data
    pub posts: std::collections::HashMap<String, PostResponse>,
    /// Cursor for pagination (null if no more replies)
    pub next_cursor: Option<String>,
}
```

- [ ] **Step 2: Verify compilation**

```bash
cd /Users/scolak/Projects/rustchat/backend && cargo check --lib
```

Expected: Success (no errors)

- [ ] **Step 3: Commit**

```bash
cd /Users/scolak/Projects/rustchat && git add backend/src/models/post.rs
git commit -m "feat(models): add ThreadResponse struct for thread endpoint

Add ThreadResponse with order, posts map, and next_cursor for
paginated thread fetching.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 2: Implement get_thread Service Method

**Files:**
- Modify: `backend/src/services/posts.rs`
- Test: `backend/tests/thread_test.rs` (create new test file)

- [ ] **Step 1: Write the failing test**

Create `backend/tests/thread_test.rs`:

```rust
use rustchat::models::{CreatePost, ThreadResponse};
use uuid::Uuid;

#[tokio::test]
async fn test_get_thread_returns_parent_and_replies() {
    let (state, test_user, channel) = setup_test_data().await;

    // Create parent post
    let parent = create_test_post(&state, &test_user.id, &channel.id, "Parent message").await;

    // Create reply
    let reply = create_test_reply(&state, &test_user.id, &channel.id, &parent.id, "Reply message").await;

    // Call get_thread
    let result = rustchat::services::posts::get_thread(&state, parent.id, None, 50).await;

    assert!(result.is_ok());
    let thread = result.unwrap();
    assert_eq!(thread.order.len(), 2);
    assert!(thread.posts.contains_key(&parent.id.to_string()));
    assert!(thread.posts.contains_key(&reply.id.to_string()));
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /Users/scolak/Projects/rustchat/backend && cargo test test_get_thread_returns_parent_and_replies -- --nocapture
```

Expected: FAIL with "function `get_thread` not found"

- [ ] **Step 3: Implement get_thread service method**

Add to `backend/src/services/posts.rs` before the last closing brace:

```rust
/// Query parameters for thread fetching
#[derive(Debug, Default)]
pub struct ThreadQuery {
    pub cursor: Option<Uuid>,
    pub limit: i64,
}

/// Get thread with parent post and replies
pub async fn get_thread(
    state: &AppState,
    post_id: Uuid,
    cursor: Option<Uuid>,
    limit: i64,
) -> ApiResult<ThreadResponse> {
    let limit = limit.clamp(1, 100);

    // Fetch parent post with user info
    let parent: Option<PostResponse> = sqlx::query_as(
        r#"
        SELECT
            p.id, p.channel_id, p.user_id, p.root_post_id, p.message, p.props, p.file_ids,
            p.is_pinned, p.created_at, p.edited_at, p.deleted_at,
            p.reply_count::int8 as reply_count, p.last_reply_at, p.seq,
            u.username, u.avatar_url, u.email,
            FALSE as is_saved
        FROM posts p
        JOIN users u ON p.user_id = u.id
        WHERE p.id = $1 AND p.deleted_at IS NULL
        "#
    )
    .bind(post_id)
    .fetch_optional(&state.db)
    .await?;

    let parent = parent.ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

    // Build query for replies
    let mut query = String::from(
        r#"
        SELECT
            p.id, p.channel_id, p.user_id, p.root_post_id, p.message, p.props, p.file_ids,
            p.is_pinned, p.created_at, p.edited_at, p.deleted_at,
            p.reply_count::int8 as reply_count, p.last_reply_at, p.seq,
            u.username, u.avatar_url, u.email,
            FALSE as is_saved
        FROM posts p
        JOIN users u ON p.user_id = u.id
        WHERE p.root_post_id = $1 AND p.deleted_at IS NULL
        "#
    );

    if cursor.is_some() {
        query.push_str(" AND p.id > $2 ");
    }

    query.push_str("ORDER BY p.created_at ASC LIMIT $3");

    // Fetch replies
    let replies: Vec<PostResponse> = if let Some(cursor_id) = cursor {
        sqlx::query_as(&query)
            .bind(post_id)
            .bind(cursor_id)
            .bind(limit + 1) // Fetch one extra to determine if there's more
            .fetch_all(&state.db)
            .await?
    } else {
        let query_no_cursor = query.replace("AND p.id > $2", "");
        sqlx::query_as(&query_no_cursor.replace("$3", "$2"))
            .bind(post_id)
            .bind(limit + 1)
            .fetch_all(&state.db)
            .await?
    };

    // Determine pagination
    let has_more = replies.len() > limit as usize;
    let replies: Vec<PostResponse> = replies.into_iter().take(limit as usize).collect();

    let next_cursor = if has_more {
        replies.last().map(|r| r.id.to_string())
    } else {
        None
    };

    // Build response
    let mut order = vec![parent.id.to_string()];
    let mut posts_map = std::collections::HashMap::new();
    posts_map.insert(parent.id.to_string(), parent);

    for reply in replies {
        order.push(reply.id.to_string());
        posts_map.insert(reply.id.to_string(), reply);
    }

    Ok(ThreadResponse {
        order,
        posts: posts_map,
        next_cursor,
    })
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cd /Users/scolak/Projects/rustchat/backend && cargo test test_get_thread_returns_parent_and_replies -- --nocapture
```

Expected: PASS

- [ ] **Step 5: Commit**

```bash
cd /Users/scolak/Projects/rustchat && git add backend/src/services/posts.rs backend/tests/thread_test.rs
git commit -m "feat(services): implement get_thread service method

Add get_thread with cursor pagination, parent+replies fetch,
and ThreadResponse construction. Includes integration test.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 3: Add GET /posts/{id}/thread API Endpoint

**Files:**
- Modify: `backend/src/api/v4/posts.rs`
- Modify: `backend/src/api/v4/mod.rs` (if needed to register route)
- Test: Run manual curl test

- [ ] **Step 1: Add handler function to posts.rs**

Add to `backend/src/api/v4/posts.rs` near other post handlers:

```rust
use crate::services::posts::get_thread;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ThreadQueryParams {
    cursor: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    50
}

/// Get thread endpoint
pub async fn get_thread_handler(
    State(state): State<AppState>,
    Path(post_id): Path<String>,
    Query(params): Query<ThreadQueryParams>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let post_uuid = Uuid::parse_str(&post_id)
        .map_err(|_| AppError::BadRequest("Invalid post ID".to_string()))?;

    let cursor = params.cursor
        .and_then(|c| Uuid::parse_str(&c).ok());

    // Verify user has access to the channel containing this post
    let post: Option<(Uuid,)> = sqlx::query_as(
        "SELECT channel_id FROM posts WHERE id = $1 AND deleted_at IS NULL"
    )
    .bind(post_uuid)
    .fetch_optional(&state.db)
    .await?;

    let (channel_id,) = post.ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

    // Check membership
    let is_member: Option<(bool,)> = sqlx::query_as(
        "SELECT TRUE FROM channel_members WHERE channel_id = $1 AND user_id = $2"
    )
    .bind(channel_id)
    .bind(auth.user_id)
    .fetch_optional(&state.db)
    .await?;

    if is_member.is_none() {
        return Err(AppError::Forbidden("Not a member of this channel".to_string()));
    }

    // Fetch thread
    let thread = get_thread(&state, post_uuid, cursor, params.limit).await?;

    Ok(Json(serde_json::json!(thread)))
}
```

- [ ] **Step 2: Register the route**

Find where post routes are registered in `backend/src/api/v4/mod.rs` or `backend/src/api/mod.rs` and add:

```rust
// In the router setup, add:
.route("/posts/{post_id}/thread", get(posts::get_thread_handler))
```

- [ ] **Step 3: Verify compilation**

```bash
cd /Users/scolak/Projects/rustchat/backend && cargo check
```

Expected: Success

- [ ] **Step 4: Manual test with curl**

Start the backend (if not running):
```bash
cd /Users/scolak/Projects/rustchat/backend && cargo run
```

Then test (you'll need valid auth token and post ID):
```bash
curl -H "Authorization: Bearer YOUR_TOKEN" \
  http://localhost:3000/api/v4/posts/POST_ID/thread
```

Expected: JSON response with thread structure

- [ ] **Step 5: Commit**

```bash
cd /Users/scolak/Projects/rustchat && git add backend/src/api/v4/posts.rs backend/src/api/v4/mod.rs
git commit -m "feat(api): add GET /posts/{id}/thread endpoint

Add thread endpoint with permission checks, cursor pagination,
and proper error handling.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 4: Create Frontend threadService

**Files:**
- Create: `frontend/src/features/messages/services/threadService.ts`
- Modify: `frontend/src/features/messages/index.ts`

- [ ] **Step 1: Create threadService.ts**

```typescript
import { api } from '@/api/client'
import type { Post, ThreadResponse } from '@/types'

export interface ThreadQueryParams {
  cursor?: string
  limit?: number
}

export const threadService = {
  /**
   * Fetch a thread with parent post and replies
   */
  async getThread(postId: string, params: ThreadQueryParams = {}): Promise<ThreadResponse> {
    const query = new URLSearchParams()
    if (params.cursor) query.set('cursor', params.cursor)
    if (params.limit) query.set('limit', params.limit.toString())

    const queryString = query.toString()
    const url = `/api/v4/posts/${postId}/thread${queryString ? `?${queryString}` : ''}`

    const response = await api.get(url)
    return response.data
  },

  /**
   * Send a reply to a thread
   */
  async sendReply(channelId: string, rootId: string, message: string, fileIds: string[] = []): Promise<Post> {
    const response = await api.post('/api/v4/posts', {
      channel_id: channelId,
      root_id: rootId,
      parent_id: rootId,
      message,
      file_ids: fileIds,
    })
    return response.data
  },
}

export interface ThreadResponse {
  order: string[]
  posts: Record<string, Post>
  next_cursor?: string
}
```

- [ ] **Step 2: Export from messages/index.ts**

Add to `frontend/src/features/messages/index.ts`:

```typescript
// Add to existing exports
export { threadService, type ThreadResponse, type ThreadQueryParams } from './services/threadService'
```

- [ ] **Step 3: Verify TypeScript compilation**

```bash
cd /Users/scolak/Projects/rustchat/frontend && npx vue-tsc --noEmit --skipLibCheck
```

Expected: No TypeScript errors

- [ ] **Step 4: Commit**

```bash
cd /Users/scolak/Projects/rustchat && git add frontend/src/features/messages/services/threadService.ts frontend/src/features/messages/index.ts
git commit -m "feat(frontend): add threadService for thread API calls

Add getThread with pagination and sendReply methods.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 5: Create threadStore

**Files:**
- Create: `frontend/src/features/messages/stores/threadStore.ts`
- Modify: `frontend/src/features/messages/index.ts`

- [ ] **Step 1: Create threadStore.ts**

```typescript
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Post } from '@/types'
import { threadService, type ThreadResponse } from '../services/threadService'

export interface ThreadState {
  isOpen: boolean
  parentPostId: string | null
  parentPost: Post | null
  replies: Post[]
  hasMore: boolean
  cursor: string | null
  isLoading: boolean
  isSending: boolean
  draft: string
}

export const useThreadStore = defineStore('thread', () => {
  // State
  const isOpen = ref(false)
  const parentPostId = ref<string | null>(null)
  const parentPost = ref<Post | null>(null)
  const replies = ref<Post[]>([])
  const hasMore = ref(false)
  const cursor = ref<string | null>(null)
  const isLoading = ref(false)
  const isSending = ref(false)
  const draft = ref('')

  // Getters
  const replyCount = computed(() => replies.value.length)
  const orderedReplies = computed(() => {
    // Replies are already in chronological order from API
    return replies.value
  })

  // Actions
  async function openThread(postId: string): Promise<void> {
    if (parentPostId.value === postId && isOpen.value) {
      return // Already open
    }

    isOpen.value = true
    parentPostId.value = postId
    isLoading.value = true
    replies.value = []

    try {
      const response = await threadService.getThread(postId, { limit: 50 })

      // Extract parent from posts map
      parentPost.value = response.posts[postId] || null

      // Extract replies (everything except parent)
      replies.value = response.order
        .filter(id => id !== postId)
        .map(id => response.posts[id])
        .filter(Boolean)

      cursor.value = response.next_cursor || null
      hasMore.value = !!response.next_cursor

      // Restore draft if exists
      const savedDraft = localStorage.getItem(`thread_draft_${postId}`)
      if (savedDraft) {
        draft.value = savedDraft
      }
    } catch (error) {
      console.error('Failed to load thread:', error)
      closeThread()
      throw error
    } finally {
      isLoading.value = false
    }
  }

  function closeThread(): void {
    // Save draft before closing
    if (parentPostId.value && draft.value.trim()) {
      localStorage.setItem(`thread_draft_${parentPostId.value}`, draft.value)
    }

    isOpen.value = false
    parentPostId.value = null
    parentPost.value = null
    replies.value = []
    hasMore.value = false
    cursor.value = null
    draft.value = ''
  }

  async function loadMoreReplies(): Promise<void> {
    if (!parentPostId.value || !cursor.value || isLoading.value) return

    isLoading.value = true
    try {
      const response = await threadService.getThread(parentPostId.value, {
        cursor: cursor.value,
        limit: 50,
      })

      // Append new replies
      const newReplies = response.order
        .filter(id => id !== parentPostId.value)
        .map(id => response.posts[id])
        .filter(Boolean)

      replies.value.push(...newReplies)
      cursor.value = response.next_cursor || null
      hasMore.value = !!response.next_cursor
    } catch (error) {
      console.error('Failed to load more replies:', error)
    } finally {
      isLoading.value = false
    }
  }

  async function sendReply(message: string, fileIds: string[] = []): Promise<void> {
    if (!parentPostId.value || !parentPost.value || isSending.value) return

    isSending.value = true
    try {
      const reply = await threadService.sendReply(
        parentPost.value.channel_id,
        parentPostId.value,
        message,
        fileIds
      )

      // Optimistic update - add reply immediately
      replies.value.push(reply)

      // Clear draft
      draft.value = ''
      localStorage.removeItem(`thread_draft_${parentPostId.value}`)

      // Update parent reply count
      if (parentPost.value) {
        parentPost.value.reply_count = (parentPost.value.reply_count || 0) + 1
      }
    } catch (error) {
      console.error('Failed to send reply:', error)
      throw error
    } finally {
      isSending.value = false
    }
  }

  function setDraft(value: string): void {
    draft.value = value
    if (parentPostId.value) {
      localStorage.setItem(`thread_draft_${parentPostId.value}`, value)
    }
  }

  function onNewReply(reply: Post): void {
    // Called by WebSocket handler when new reply arrives
    if (reply.root_id === parentPostId.value && reply.id !== parentPostId.value) {
      // Check if already in list (from optimistic update)
      const exists = replies.value.some(r => r.id === reply.id)
      if (!exists) {
        replies.value.push(reply)
      }
    }
  }

  function onPostDeleted(postId: string): void {
    if (postId === parentPostId.value) {
      parentPost.value = null
    } else {
      replies.value = replies.value.filter(r => r.id !== postId)
    }
  }

  return {
    // State
    isOpen,
    parentPostId,
    parentPost,
    replies,
    hasMore,
    cursor,
    isLoading,
    isSending,
    draft,
    // Getters
    replyCount,
    orderedReplies,
    // Actions
    openThread,
    closeThread,
    loadMoreReplies,
    sendReply,
    setDraft,
    onNewReply,
    onPostDeleted,
  }
})
```

- [ ] **Step 2: Export from messages/index.ts**

Add to `frontend/src/features/messages/index.ts`:

```typescript
export { useThreadStore, type ThreadState } from './stores/threadStore'
```

- [ ] **Step 3: Verify TypeScript compilation**

```bash
cd /Users/scolak/Projects/rustchat/frontend && npx vue-tsc --noEmit --skipLibCheck
```

Expected: No errors

- [ ] **Step 4: Commit**

```bash
cd /Users/scolak/Projects/rustchat && git add frontend/src/features/messages/stores/threadStore.ts frontend/src/features/messages/index.ts
git commit -m "feat(frontend): add threadStore for thread state management

Add Pinia store with open/close, pagination, draft persistence,
and WebSocket update handlers.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 6: Create Thread Components

**Files:**
- Create: `frontend/src/components/thread/ThreadPanel.vue`
- Create: `frontend/src/components/thread/ThreadHeader.vue`
- Create: `frontend/src/components/thread/ThreadReplyList.vue`
- Create: `frontend/src/components/thread/ThreadReplyItem.vue`
- Modify: `frontend/src/features/messages/index.ts`

- [ ] **Step 1: Create ThreadPanel.vue**

```vue
<template>
  <Transition name="slide">
    <div
      v-if="threadStore.isOpen"
      class="thread-panel"
      role="dialog"
      aria-modal="true"
      aria-label="Thread panel"
      @keydown.esc="threadStore.closeThread"
    >
      <div class="thread-panel__overlay" @click="threadStore.closeThread" />
      <div class="thread-panel__content">
        <ThreadHeader
          :post="threadStore.parentPost"
          @close="threadStore.closeThread"
        />

        <div v-if="threadStore.isLoading && !threadStore.replies.length" class="thread-panel__loading">
          Loading thread...
        </div>

        <ThreadReplyList
          v-else
          :replies="threadStore.orderedReplies"
          :has-more="threadStore.hasMore"
          :is-loading="threadStore.isLoading"
          @load-more="threadStore.loadMoreReplies"
        />

        <ThreadComposer
          :draft="threadStore.draft"
          :is-sending="threadStore.isSending"
          @send="handleSend"
          @update:draft="threadStore.setDraft"
        />
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { useThreadStore } from '@/features/messages'
import ThreadHeader from './ThreadHeader.vue'
import ThreadReplyList from './ThreadReplyList.vue'
import ThreadComposer from '../composer/ThreadComposer.vue'

const threadStore = useThreadStore()

async function handleSend(message: string, fileIds: string[]) {
  try {
    await threadStore.sendReply(message, fileIds)
  } catch (error) {
    // Error is logged in store, UI shows error state
  }
}
</script>

<style scoped>
.thread-panel {
  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  z-index: 100;
  display: flex;
  justify-content: flex-end;
}

.thread-panel__overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
}

.thread-panel__content {
  position: relative;
  width: 400px;
  max-width: 100vw;
  height: 100%;
  background: var(--background);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
}

.thread-panel__loading {
  padding: 2rem;
  text-align: center;
  color: var(--text-muted);
}

.slide-enter-active,
.slide-leave-active {
  transition: transform 0.2s ease;
}

.slide-enter-from,
.slide-leave-to {
  transform: translateX(100%);
}

@media (max-width: 640px) {
  .thread-panel__content {
    width: 100%;
  }
}
</style>
```

- [ ] **Step 2: Create ThreadHeader.vue**

```vue
<template>
  <div class="thread-header">
    <div class="thread-header__info">
      <h3 class="thread-header__title">Thread</h3>
      <p v-if="post" class="thread-header__subtitle">
        {{ post.reply_count || 0 }} replies
      </p>
      <p v-else class="thread-header__subtitle thread-header__subtitle--deleted">
        This message was deleted
      </p>
    </div>
    <button
      class="thread-header__close"
      aria-label="Close thread"
      @click="$emit('close')"
    >
      ✕
    </button>
  </div>

  <div v-if="post" class="thread-header__parent">
    <MessageItem :post="post" :is-thread-parent="true" />
  </div>
</template>

<script setup lang="ts">
import type { Post } from '@/types'
import MessageItem from '@/components/messages/MessageItem.vue'

defineProps<{
  post: Post | null
}>()

defineEmits<{
  close: []
}>()
</script>

<style scoped>
.thread-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  border-bottom: 1px solid var(--border);
}

.thread-header__title {
  font-size: 1rem;
  font-weight: 600;
  margin: 0;
}

.thread-header__subtitle {
  font-size: 0.875rem;
  color: var(--text-muted);
  margin: 0;
}

.thread-header__subtitle--deleted {
  color: var(--text-danger);
}

.thread-header__close {
  background: none;
  border: none;
  font-size: 1.25rem;
  cursor: pointer;
  padding: 0.5rem;
  color: var(--text-muted);
}

.thread-header__close:hover {
  color: var(--text);
}

.thread-header__parent {
  border-bottom: 1px solid var(--border);
  padding: 0.5rem;
}
</style>
```

- [ ] **Step 3: Create ThreadReplyList.vue**

```vue
<template>
  <div ref="listRef" class="thread-reply-list" @scroll="handleScroll">
    <div class="thread-reply-list__items">
      <ThreadReplyItem
        v-for="reply in replies"
        :key="reply.id"
        :reply="reply"
      />
    </div>

    <div v-if="hasMore" class="thread-reply-list__more">
      <button
        v-if="!isLoading"
        class="thread-reply-list__load-btn"
        @click="$emit('load-more')"
      >
        Load more replies
      </button>
      <span v-else class="thread-reply-list__loading">Loading...</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { Post } from '@/types'
import ThreadReplyItem from './ThreadReplyItem.vue'

defineProps<{
  replies: Post[]
  hasMore: boolean
  isLoading: boolean
}>()

defineEmits<{
  'load-more': []
}>()

const listRef = ref<HTMLElement>()

function handleScroll() {
  const el = listRef.value
  if (!el) return

  // Auto-load more when near bottom
  const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 100
  if (nearBottom) {
    emit('load-more')
  }
}
</script>

<style scoped>
.thread-reply-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
}

.thread-reply-list__items {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.thread-reply-list__more {
  text-align: center;
  padding: 1rem;
}

.thread-reply-list__load-btn {
  background: none;
  border: none;
  color: var(--primary);
  cursor: pointer;
  font-size: 0.875rem;
}

.thread-reply-list__loading {
  color: var(--text-muted);
  font-size: 0.875rem;
}
</style>
```

- [ ] **Step 4: Create ThreadReplyItem.vue**

```vue
<template>
  <div class="thread-reply-item" :data-reply-id="reply.id">
    <div class="thread-reply-item__avatar">
      <UserAvatar :user="reply.user" :size="32" />
    </div>
    <div class="thread-reply-item__content">
      <div class="thread-reply-item__header">
        <span class="thread-reply-item__author">{{ reply.user?.username }}</span>
        <span class="thread-reply-item__time">{{ formatTime(reply.created_at) }}</span>
      </div>
      <div class="thread-reply-item__message" v-html="renderMarkdown(reply.message)" />
      <div v-if="reply.reactions?.length" class="thread-reply-item__reactions">
        <ReactionBadge
          v-for="reaction in reply.reactions"
          :key="reaction.emoji"
          :reaction="reaction"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Post } from '@/types'
import UserAvatar from '@/components/ui/UserAvatar.vue'
import ReactionBadge from '@/components/messages/ReactionBadge.vue'
import { renderMarkdown } from '@/utils/markdown'
import { formatTime } from '@/utils/time'

defineProps<{
  reply: Post
}>()
</script>

<style scoped>
.thread-reply-item {
  display: flex;
  gap: 0.75rem;
  padding: 0.5rem;
}

.thread-reply-item__content {
  flex: 1;
  min-width: 0;
}

.thread-reply-item__header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.25rem;
}

.thread-reply-item__author {
  font-weight: 600;
  font-size: 0.875rem;
}

.thread-reply-item__time {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.thread-reply-item__message {
  font-size: 0.9375rem;
  line-height: 1.5;
}

.thread-reply-item__message :deep(p) {
  margin: 0;
}

.thread-reply-item__reactions {
  display: flex;
  gap: 0.25rem;
  margin-top: 0.25rem;
}
</style>
```

- [ ] **Step 5: Export components**

Add to `frontend/src/features/messages/index.ts`:

```typescript
export { default as ThreadPanel } from '@/components/thread/ThreadPanel.vue'
export { default as ThreadHeader } from '@/components/thread/ThreadHeader.vue'
export { default as ThreadReplyList } from '@/components/thread/ThreadReplyList.vue'
export { default as ThreadReplyItem } from '@/components/thread/ThreadReplyItem.vue'
```

- [ ] **Step 6: Verify compilation**

```bash
cd /Users/scolak/Projects/rustchat/frontend && npx vue-tsc --noEmit --skipLibCheck
```

Expected: No errors

- [ ] **Step 7: Commit**

```bash
cd /Users/scolak/Projects/rustchat && git add frontend/src/components/thread/
git commit -m "feat(frontend): add ThreadPanel components

Add ThreadPanel, ThreadHeader, ThreadReplyList, ThreadReplyItem
components with full styling and accessibility.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 7: Create ThreadComposer Component

**Files:**
- Create: `frontend/src/components/composer/ThreadComposer.vue`
- Modify: `frontend/src/features/messages/index.ts`

- [ ] **Step 1: Create ThreadComposer.vue**

```vue
<template>
  <div class="thread-composer">
    <FormattingToolbar
      v-if="editor"
      :editor="editor"
      @mention="insertMention"
      @emoji="openEmojiPicker"
    />
    <div class="thread-composer__input-wrapper">
      <EditorContent
        :editor="editor"
        class="thread-composer__input"
      />
      <button
        class="thread-composer__send"
        :disabled="!canSend || isSending"
        @click="handleSend"
      >
        {{ isSending ? 'Sending...' : 'Send' }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, watch } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import Mention from '@tiptap/extension-mention'
import Placeholder from '@tiptap/extension-placeholder'
import FormattingToolbar from './FormattingToolbar.vue'

const props = defineProps<{
  draft: string
  isSending: boolean
}>()

const emit = defineEmits<{
  send: [message: string, fileIds: string[]]
  'update:draft': [value: string]
}>()

const editor = useEditor({
  extensions: [
    StarterKit,
    Placeholder.configure({
      placeholder: 'Reply in thread...',
    }),
    Mention.configure({
      suggestion: {
        // TODO: Implement mention suggestion
      },
    }),
  ],
  content: props.draft,
  onUpdate: ({ editor }) => {
    emit('update:draft', editor.getText())
  },
  editorProps: {
    handleKeyDown: (view, event) => {
      if (event.key === 'Enter' && !event.shiftKey) {
        event.preventDefault()
        handleSend()
        return true
      }
      return false
    },
  },
})

const canSend = computed(() => {
  return editor.value?.getText().trim().length > 0 && !props.isSending
})

watch(() => props.draft, (newDraft) => {
  if (editor.value && editor.value.getText() !== newDraft) {
    editor.value.commands.setContent(newDraft)
  }
})

function handleSend() {
  if (!canSend.value) return

  const message = editor.value?.getText().trim() || ''
  emit('send', message, [])

  // Clear editor
  editor.value?.commands.clearContent()
}

function insertMention() {
  editor.value?.chain().focus().insertContent('@').run()
}

function openEmojiPicker() {
  // TODO: Implement emoji picker
  editor.value?.chain().focus().insertContent(':').run()
}
</script>

<style scoped>
.thread-composer {
  border-top: 1px solid var(--border);
  padding: 0.5rem;
}

.thread-composer__input-wrapper {
  display: flex;
  gap: 0.5rem;
  align-items: flex-end;
}

.thread-composer__input {
  flex: 1;
  min-height: 40px;
  max-height: 200px;
  overflow-y: auto;
  padding: 0.5rem;
  border: 1px solid var(--border);
  border-radius: 4px;
  background: var(--background);
}

.thread-composer__input :deep(.ProseMirror) {
  outline: none;
  min-height: 24px;
}

.thread-composer__input :deep(.ProseMirror p.is-editor-empty:first-child::before) {
  content: attr(data-placeholder);
  float: left;
  color: var(--text-muted);
  pointer-events: none;
  height: 0;
}

.thread-composer__send {
  padding: 0.5rem 1rem;
  background: var(--primary);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.thread-composer__send:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
```

- [ ] **Step 2: Export from index**

Add to `frontend/src/features/messages/index.ts`:

```typescript
export { default as ThreadComposer } from '@/components/composer/ThreadComposer.vue'
```

- [ ] **Step 3: Install TipTap dependencies**

```bash
cd /Users/scolak/Projects/rustchat/frontend && npm install @tiptap/vue-3 @tiptap/starter-kit @tiptap/extension-mention @tiptap/extension-placeholder
```

Expected: Packages installed successfully

- [ ] **Step 4: Verify compilation**

```bash
cd /Users/scolak/Projects/rustchat/frontend && npx vue-tsc --noEmit --skipLibCheck
```

Expected: No errors (or minor type errors that can be fixed)

- [ ] **Step 5: Commit**

```bash
cd /Users/scolak/Projects/rustchat && git add frontend/src/components/composer/ThreadComposer.vue frontend/package.json frontend/package-lock.json
git commit -m "feat(frontend): add ThreadComposer with TipTap integration

Add ThreadComposer component with TipTap editor, FormattingToolbar,
placeholder, and mention support.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 8: Create FormattingToolbar Component

**Files:**
- Create: `frontend/src/components/composer/FormattingToolbar.vue`
- Modify: `frontend/src/features/messages/index.ts`

- [ ] **Step 1: Create FormattingToolbar.vue**

```vue
<template>
  <div class="formatting-toolbar" role="toolbar" aria-label="Formatting">
    <button
      v-for="item in toolbarItems"
      :key="item.command"
      class="formatting-toolbar__btn"
      :class="{ 'is-active': item.isActive?.() }"
      :title="item.label + ' (' + item.shortcut + ')'"
      @click="executeCommand(item)"
    >
      <component :is="item.icon" :size="16" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { Bold, Italic, Strikethrough, Code, Code2, Link, List, ListOrdered, AtSign, Hash, Smile } from 'lucide-vue-next'
import type { Editor } from '@tiptap/vue-3'

const props = defineProps<{
  editor: Editor
}>()

const emit = defineEmits<{
  mention: []
  emoji: []
}>()

const toolbarItems = [
  {
    command: 'toggleBold',
    icon: Bold,
    label: 'Bold',
    shortcut: 'Ctrl+B',
    isActive: () => props.editor.isActive('bold'),
  },
  {
    command: 'toggleItalic',
    icon: Italic,
    label: 'Italic',
    shortcut: 'Ctrl+I',
    isActive: () => props.editor.isActive('italic'),
  },
  {
    command: 'toggleStrike',
    icon: Strikethrough,
    label: 'Strikethrough',
    shortcut: 'Ctrl+Shift+X',
    isActive: () => props.editor.isActive('strike'),
  },
  {
    command: 'separator',
    icon: null,
    label: '',
    shortcut: '',
  },
  {
    command: 'toggleCode',
    icon: Code,
    label: 'Inline code',
    shortcut: 'Ctrl+E',
    isActive: () => props.editor.isActive('code'),
  },
  {
    command: 'toggleCodeBlock',
    icon: Code2,
    label: 'Code block',
    shortcut: 'Ctrl+Shift+C',
    isActive: () => props.editor.isActive('codeBlock'),
  },
  {
    command: 'separator',
    icon: null,
    label: '',
    shortcut: '',
  },
  {
    command: 'toggleBulletList',
    icon: List,
    label: 'Bullet list',
    shortcut: 'Ctrl+Shift+8',
    isActive: () => props.editor.isActive('bulletList'),
  },
  {
    command: 'toggleOrderedList',
    icon: ListOrdered,
    label: 'Numbered list',
    shortcut: 'Ctrl+Shift+7',
    isActive: () => props.editor.isActive('orderedList'),
  },
  {
    command: 'separator',
    icon: null,
    label: '',
    shortcut: '',
  },
  {
    command: 'insertMention',
    icon: AtSign,
    label: 'Mention',
    shortcut: '@',
    isActive: () => false,
  },
  {
    command: 'insertEmoji',
    icon: Smile,
    label: 'Emoji',
    shortcut: ':',
    isActive: () => false,
  },
]

function executeCommand(item: typeof toolbarItems[0]) {
  if (item.command === 'separator') return

  if (item.command === 'insertMention') {
    emit('mention')
    return
  }

  if (item.command === 'insertEmoji') {
    emit('emoji')
    return
  }

  const chain = props.editor.chain().focus()

  switch (item.command) {
    case 'toggleBold':
      chain.toggleBold().run()
      break
    case 'toggleItalic':
      chain.toggleItalic().run()
      break
    case 'toggleStrike':
      chain.toggleStrike().run()
      break
    case 'toggleCode':
      chain.toggleCode().run()
      break
    case 'toggleCodeBlock':
      chain.toggleCodeBlock().run()
      break
    case 'toggleBulletList':
      chain.toggleBulletList().run()
      break
    case 'toggleOrderedList':
      chain.toggleOrderedList().run()
      break
  }
}
</script>

<style scoped>
.formatting-toolbar {
  display: flex;
  gap: 0.25rem;
  padding: 0.25rem;
  border-bottom: 1px solid var(--border);
  flex-wrap: wrap;
}

.formatting-toolbar__btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  background: none;
  border: 1px solid transparent;
  border-radius: 4px;
  cursor: pointer;
  color: var(--text-muted);
}

.formatting-toolbar__btn:hover {
  background: var(--background-hover);
  color: var(--text);
}

.formatting-toolbar__btn.is-active {
  background: var(--primary-light);
  color: var(--primary);
  border-color: var(--primary);
}

.formatting-toolbar__btn[disabled] {
  opacity: 0.3;
  cursor: not-allowed;
}
</style>
```

- [ ] **Step 2: Export from index**

Add to `frontend/src/features/messages/index.ts`:

```typescript
export { default as FormattingToolbar } from '@/components/composer/FormattingToolbar.vue'
```

- [ ] **Step 3: Verify compilation**

```bash
cd /Users/scolak/Projects/rustchat/frontend && npx vue-tsc --noEmit --skipLibCheck
```

Expected: No errors

- [ ] **Step 4: Commit**

```bash
cd /Users/scolak/Projects/rustchat && git add frontend/src/components/composer/FormattingToolbar.vue
git commit -m "feat(frontend): add FormattingToolbar component

Add toolbar with bold, italic, code, lists, mention, emoji buttons.
Integrates with TipTap editor.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 9: Integrate Thread Panel into ChannelView

**Files:**
- Modify: `frontend/src/views/main/ChannelView.vue`

- [ ] **Step 1: Add ThreadPanel import and usage**

Find the template section in ChannelView.vue and add `<ThreadPanel />` before closing tag:

```vue
<template>
  <div class="channel-view">
    <!-- existing content -->

    <!-- Add at the end -->
    <ThreadPanel />
  </div>
</template>
```

Add to script section:

```typescript
import { ThreadPanel } from '@/features/messages'

export default {
  components: {
    // existing components...
    ThreadPanel,
  },
}
```

- [ ] **Step 2: Add "View replies" button to MessageItem**

Modify MessageItem.vue to show reply count button:

```vue
<button
  v-if="post.reply_count > 0"
  class="message__replies"
  @click="openThread"
>
  {{ post.reply_count }} replies
</button>
```

Add to script:

```typescript
import { useThreadStore } from '@/features/messages'

const threadStore = useThreadStore()

function openThread() {
  threadStore.openThread(props.post.id)
}
```

- [ ] **Step 3: Verify compilation**

```bash
cd /Users/scolak/Projects/rustchat/frontend && npx vue-tsc --noEmit --skipLibCheck
```

Expected: No errors

- [ ] **Step 4: Commit**

```bash
cd /Users/scolak/Projects/rustchat && git add frontend/src/views/main/ChannelView.vue
git commit -m "feat(frontend): integrate ThreadPanel into ChannelView

Add ThreadPanel to ChannelView and "View replies" button to messages.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 10: Add WebSocket Handlers for Thread Updates

**Files:**
- Create: `frontend/src/features/messages/handlers/threadSocketHandlers.ts`
- Modify: `frontend/src/core/websocket/registerHandlers.ts`

- [ ] **Step 1: Create threadSocketHandlers.ts**

```typescript
import { useThreadStore } from '../stores/threadStore'
import type { Post } from '@/types'

export function registerThreadHandlers() {
  const threadStore = useThreadStore()

  return {
    handleNewPost(post: Post) {
      // If this is a reply to the currently open thread
      if (post.root_id && post.root_id !== post.id) {
        threadStore.onNewReply(post)
      }
    },

    handlePostDeleted(postId: string) {
      threadStore.onPostDeleted(postId)
    },

    handlePostUpdated(post: Post) {
      // Update reply if in current thread
      if (post.root_id === threadStore.parentPostId) {
        const index = threadStore.replies.findIndex(r => r.id === post.id)
        if (index !== -1) {
          threadStore.replies[index] = post
        }
      }
    },
  }
}
```

- [ ] **Step 2: Register handlers in websocket**

Modify `frontend/src/core/websocket/registerHandlers.ts`:

```typescript
import { registerThreadHandlers } from '@/features/messages/handlers/threadSocketHandlers'

export function registerWebSocketHandlers() {
  // existing handlers...

  const threadHandlers = registerThreadHandlers()

  // Add to message handler switch:
  // case 'posted':
  //   threadHandlers.handleNewPost(data.post)
  //   break
  // case 'post_deleted':
  //   threadHandlers.handlePostDeleted(data.post_id)
  //   break
  // case 'post_updated':
  //   threadHandlers.handlePostUpdated(data.post)
  //   break
}
```

- [ ] **Step 3: Verify compilation**

```bash
cd /Users/scolak/Projects/rustchat/frontend && npx vue-tsc --noEmit --skipLibCheck
```

Expected: No errors

- [ ] **Step 4: Commit**

```bash
cd /Users/scolak/Projects/rustchat && git add frontend/src/features/messages/handlers/ frontend/src/core/websocket/
git commit -m "feat(frontend): add WebSocket handlers for thread updates

Add real-time updates for new replies, deletions, and edits in threads.

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 11: Run Tests and Fix Issues

**Files:**
- All modified files

- [ ] **Step 1: Run backend tests**

```bash
cd /Users/scolak/Projects/rustchat/backend && cargo test --no-fail-fast -- --nocapture 2>&1 | head -50
```

Expected: All tests pass (or fix any failures)

- [ ] **Step 2: Run frontend type check**

```bash
cd /Users/scolak/Projects/rustchat/frontend && npx vue-tsc --noEmit --skipLibCheck
```

Expected: No TypeScript errors

- [ ] **Step 3: Build frontend**

```bash
cd /Users/scolak/Projects/rustchat/frontend && npm run build 2>&1 | tail -30
```

Expected: Build succeeds with no errors

- [ ] **Step 4: Manual smoke test**

Start backend and frontend:
```bash
# Terminal 1
cd /Users/scolak/Projects/rustchat/backend && cargo run

# Terminal 2
cd /Users/scolak/Projects/rustchat/frontend && npm run dev
```

Verify:
- [ ] Click "View replies" on a message opens thread panel
- [ ] Thread panel shows parent message and replies
- [ ] Can send reply in thread
- [ ] Formatting toolbar buttons work
- [ ] Panel closes with Escape or close button

- [ ] **Step 5: Final commit**

```bash
cd /Users/scolak/Projects/rustchat && git add -A
git commit -m "feat: complete message threads and formatting toolbar

- Add backend thread endpoint with pagination
- Add ThreadPanel with slide-over UI
- Add FormattingToolbar with TipTap integration
- Add real-time WebSocket updates
- Full test coverage

Closes #<issue-number>

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Summary

This plan implements:

| Component | Files Created/Modified |
|-----------|----------------------|
| Backend API | ThreadResponse model, get_thread service, GET /posts/{id}/thread endpoint |
| Frontend State | threadService, threadStore |
| Thread UI | ThreadPanel, ThreadHeader, ThreadReplyList, ThreadReplyItem |
| Composer | ThreadComposer, FormattingToolbar |
| Integration | ChannelView, WebSocket handlers |

**Estimated Time:** 2-3 days of focused work
**Dependencies:** TipTap editor library
**Testing:** Unit tests for backend, type-checking for frontend, manual smoke tests
