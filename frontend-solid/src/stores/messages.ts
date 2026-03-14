import { createStore, produce } from 'solid-js/store';
import { createSignal, batch, createMemo } from 'solid-js';
import { authStore } from './auth';

// ============================================
// Types
// ============================================

export interface FileAttachment {
  id: string;
  name: string;
  url: string;
  size: number;
  mime_type: string;
}

export interface Reaction {
  emoji: string;
  count: number;
  users: string[];
}

export interface Message {
  id: string;
  channelId: string;
  userId: string;
  username: string;
  avatarUrl?: string;
  email?: string;
  content: string;
  timestamp: string;
  reactions: Reaction[];
  threadCount?: number;
  lastReplyAt?: string;
  rootId?: string;
  files?: FileAttachment[];
  isPinned: boolean;
  isSaved: boolean;
  status?: 'sending' | 'delivered' | 'failed';
  clientMsgId?: string;
  props?: Record<string, unknown>;
  seq: number | string;
  editedAt?: string;
}

export interface Post {
  id: string;
  channel_id: string;
  user_id: string;
  message: string;
  root_post_id?: string;
  parent_id?: string;
  created_at: string;
  updated_at: string;
  edited_at?: string | number | null;
  edit_at?: string | number | null;
  is_pinned: boolean;
  props?: Record<string, unknown>;
  username?: string;
  avatar_url?: string;
  email?: string;
  reply_count?: number;
  last_reply_at?: string | number | null;
  files?: FileAttachment[];
  reactions?: Reaction[];
  is_saved?: boolean;
  client_msg_id?: string;
  pending_post_id?: string;
  seq: number | string;
}

export interface ReadState {
  last_read_message_id: number | null;
  first_unread_message_id: number | null;
}

export interface CreatePostRequest {
  channel_id: string;
  message: string;
  root_post_id?: string;
  parent_id?: string;
  file_ids?: string[];
  client_msg_id?: string;
}

// ============================================
// Store State
// ============================================

const [messagesByChannel, setMessagesByChannel] = createStore<Record<string, Message[]>>({});
const [repliesByThread, setRepliesByThread] = createStore<Record<string, Message[]>>({});
const [hasMoreOlderByChannel, setHasMoreOlderByChannel] = createStore<Record<string, boolean>>({});
const [readStatesByChannel, setReadStatesByChannel] = createStore<Record<string, ReadState>>({});
const [isLoading, setIsLoading] = createSignal(false);
const [isLoadingOlder, setIsLoadingOlder] = createSignal(false);
const [error, setError] = createSignal<string | null>(null);

// ============================================
// Helpers
// ============================================

function toIsoTimestamp(value: unknown): string {
  if (typeof value === 'number' && Number.isFinite(value)) {
    return new Date(value).toISOString();
  }
  if (typeof value === 'string' && value.length > 0) {
    return value;
  }
  return new Date().toISOString();
}

function toOptionalIsoTimestamp(value: unknown): string | undefined {
  if (value === null || value === undefined || value === '' || value === 0 || value === '0') {
    return undefined;
  }
  return toIsoTimestamp(value);
}

export function postToMessage(post: Post): Message {
  const rootId = post.root_post_id || post.parent_id || undefined;

  return {
    id: post.id,
    channelId: post.channel_id,
    userId: post.user_id,
    username: post.username || 'Unknown',
    avatarUrl: post.avatar_url,
    email: post.email,
    content: post.message,
    timestamp: toIsoTimestamp(post.created_at),
    reactions: post.reactions?.map((r) => ({
      emoji: r.emoji,
      count: r.count,
      users: r.users,
    })) || [],
    rootId,
    threadCount: post.reply_count || 0,
    lastReplyAt: toOptionalIsoTimestamp(post.last_reply_at),
    files: post.files || [],
    isPinned: Boolean(post.is_pinned),
    isSaved: post.is_saved || false,
    status: 'delivered',
    clientMsgId: post.client_msg_id || post.pending_post_id,
    props: post.props,
    seq: post.seq ?? 0,
    editedAt: toOptionalIsoTimestamp(post.edited_at ?? post.edit_at),
  };
}

// ============================================
// Actions
// ============================================

export function getMessages(channelId: string) {
  return createMemo(() => messagesByChannel[channelId] || []);
}

export function getReplies(threadId: string) {
  return createMemo(() => repliesByThread[threadId] || []);
}

export function hasMoreOlder(channelId: string) {
  return createMemo(() => hasMoreOlderByChannel[channelId] ?? true);
}

export async function fetchMessages(channelId: string): Promise<void> {
  batch(() => {
    setIsLoading(true);
    setError(null);
  });

  try {
    const token = authStore.token;
    const response = await fetch(
      `/api/v1/channels/${encodeURIComponent(channelId)}/posts?limit=50`,
      {
        headers: token ? { Authorization: `Bearer ${token}` } : {},
      }
    );

    if (!response.ok) throw new Error('Failed to fetch messages');

    const data = await response.json();
    const messages: Message[] = (data.messages || [])
      .filter((p: Post) => !p.root_post_id && !p.parent_id)
      .map(postToMessage)
      .reverse();

    setMessagesByChannel(channelId, messages);

    // Update read state
    if (data.read_state) {
      setReadStatesByChannel(channelId, data.read_state);
    }

    // If we got fewer than 50, we probably reached the end
    setHasMoreOlderByChannel(channelId, (data.messages || []).length >= 50);
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to fetch messages');
    throw err;
  } finally {
    setIsLoading(false);
  }
}

export async function fetchOlderMessages(channelId: string): Promise<void> {
  if (isLoading() || !hasMoreOlderByChannel[channelId]) return;

  const currentMessages = messagesByChannel[channelId] || [];
  if (currentMessages.length === 0) {
    await fetchMessages(channelId);
    return;
  }

  // Use the ID of the OLDEST message as the cursor
  const before = currentMessages[0]?.id;
  if (!before) return;

  batch(() => {
    setIsLoading(true);
    setIsLoadingOlder(true);
  });

  try {
    const token = authStore.token;
    const response = await fetch(
      `/api/v1/channels/${encodeURIComponent(channelId)}/posts?before=${encodeURIComponent(before)}&limit=50`,
      {
        headers: token ? { Authorization: `Bearer ${token}` } : {},
      }
    );

    if (!response.ok) throw new Error('Failed to fetch older messages');

    const data = await response.json();
    const olderMessages: Message[] = (data.messages || [])
      .filter((p: Post) => !p.root_post_id && !p.parent_id)
      .map(postToMessage)
      .reverse();

    if (olderMessages.length > 0) {
      setMessagesByChannel(channelId, [...olderMessages, ...currentMessages]);
    }

    setHasMoreOlderByChannel(channelId, (data.messages || []).length >= 50);
  } catch (err) {
    console.error('Failed to fetch older messages:', err);
  } finally {
    batch(() => {
      setIsLoading(false);
      setIsLoadingOlder(false);
    });
  }
}

export async function fetchThread(rootId: string): Promise<void> {
  batch(() => {
    setIsLoading(true);
    setError(null);
  });

  try {
    const token = authStore.token;
    const response = await fetch(`/api/v1/posts/${encodeURIComponent(rootId)}/thread`, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    });

    if (!response.ok) throw new Error('Failed to fetch thread');

    const data: Post[] = await response.json();
    const replies = data.map(postToMessage);
    setRepliesByThread(rootId, replies);
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to fetch thread');
    throw err;
  } finally {
    setIsLoading(false);
  }
}

export async function sendMessage(
  channelId: string,
  content: string,
  rootId?: string,
  fileIds: string[] = []
): Promise<Message> {
  const clientMsgId = crypto.randomUUID();
  const currentUser = authStore.user();

  // Create optimistic message
  const optimisticMessage: Message = {
    id: clientMsgId,
    channelId,
    userId: currentUser?.id || '',
    username: currentUser?.username || 'Me',
    avatarUrl: currentUser?.avatar_url,
    content,
    timestamp: new Date().toISOString(),
    reactions: [],
    files: [],
    isPinned: false,
    isSaved: false,
    status: 'sending',
    clientMsgId,
    rootId,
    seq: 0,
  };

  // Add optimistic message
  if (rootId) {
    setRepliesByThread(
      produce((threads) => {
        if (!threads[rootId]) threads[rootId] = [];
        threads[rootId].push(optimisticMessage);
      })
    );
  } else {
    setMessagesByChannel(
      produce((channels) => {
        if (!channels[channelId]) channels[channelId] = [];
        channels[channelId].push(optimisticMessage);
      })
    );
  }

  try {
    const token = authStore.token;
    const response = await fetch(`/api/v1/channels/${encodeURIComponent(channelId)}/posts`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({
        channel_id: channelId,
        message: content,
        root_post_id: rootId,
        file_ids: fileIds,
        client_msg_id: clientMsgId,
      }),
    });

    if (!response.ok) throw new Error('Failed to send message');

    const post: Post = await response.json();
    const finalMessage = postToMessage(post);

    // Replace optimistic message with server message
    updateOptimisticMessage(clientMsgId, finalMessage);

    return finalMessage;
  } catch (err) {
    // Mark as failed
    setMessagesByChannel(
      produce((channels) => {
        const messages = channels[channelId] || [];
        const msg = messages.find((m) => m.clientMsgId === clientMsgId);
        if (msg) msg.status = 'failed';
      })
    );
    throw err;
  }
}

export function addOptimisticMessage(message: Message) {
  if (message.rootId) {
    setRepliesByThread(
      produce((threads) => {
        if (!threads[message.rootId!]) threads[message.rootId!] = [];
        threads[message.rootId!].push(message);
      })
    );
  } else {
    setMessagesByChannel(
      produce((channels) => {
        if (!channels[message.channelId]) channels[message.channelId] = [];
        channels[message.channelId].push(message);
      })
    );
  }
}

export function updateOptimisticMessage(clientMsgId: string, serverMsg: Message) {
  const channelId = serverMsg.channelId;
  const rootId = serverMsg.rootId;

  if (rootId) {
    setRepliesByThread(
      produce((threads) => {
        const replies = threads[rootId];
        if (!replies) return;

        const index = replies.findIndex((m) => m.clientMsgId === clientMsgId || m.id === clientMsgId);
        if (index !== -1) {
          replies[index] = serverMsg;
        } else {
          replies.push(serverMsg);
        }
      })
    );
  } else {
    setMessagesByChannel(
      produce((channels) => {
        const messages = channels[channelId];
        if (!messages) return;

        const index = messages.findIndex((m) => m.clientMsgId === clientMsgId || m.id === clientMsgId);
        if (index !== -1) {
          messages[index] = serverMsg;
        } else {
          messages.push(serverMsg);
        }
      })
    );
  }
}

// ============================================
// WebSocket Event Handlers
// ============================================

export function handleNewMessage(post: Post) {
  if (!post) return;

  const message = postToMessage(post);

  if (message.rootId) {
    // Handle reply
    setRepliesByThread(
      produce((threads) => {
        if (!threads[message.rootId!]) threads[message.rootId!] = [];
        const replies = threads[message.rootId!];

        const index = replies.findIndex((m) => m.id === message.id || m.clientMsgId === message.clientMsgId);
        if (index !== -1) {
          replies[index] = message;
        } else {
          replies.push(message);
        }
      })
    );

    // Remove from main channel if it was accidentally added
    setMessagesByChannel(
      produce((channels) => {
        const messages = channels[message.channelId];
        if (!messages) return;

        const idx = messages.findIndex((m) => m.id === message.id || m.clientMsgId === message.clientMsgId);
        if (idx !== -1) {
          messages.splice(idx, 1);
        }
      })
    );
  } else {
    // Handle root message
    setMessagesByChannel(
      produce((channels) => {
        if (!channels[message.channelId]) channels[message.channelId] = [];
        const messages = channels[message.channelId];

        const index = messages.findIndex((m) => m.id === message.id || m.clientMsgId === message.clientMsgId);
        if (index !== -1) {
          messages[index] = message;
        } else {
          messages.push(message);
        }
      })
    );
  }
}

export function handleMessageUpdate(data: { id: string } & Partial<Message>) {
  if (!data.id) return;

  const updateFields = (msg: Message) => {
    if (data.content !== undefined) msg.content = data.content;
    if (data.isPinned !== undefined) msg.isPinned = data.isPinned;
    if (data.threadCount !== undefined) msg.threadCount = data.threadCount;
    if (data.editedAt !== undefined) msg.editedAt = data.editedAt;
  };

  // Update in main channels
  setMessagesByChannel(
    produce((channels) => {
      for (const cid in channels) {
        const messages = channels[cid];
        const msg = messages?.find((m) => m.id === data.id);
        if (msg) updateFields(msg);
      }
    })
  );

  // Update in cached threads
  setRepliesByThread(
    produce((threads) => {
      for (const rootId in threads) {
        const replies = threads[rootId];
        const msg = replies?.find((m) => m.id === data.id);
        if (msg) updateFields(msg);
      }
    })
  );
}

export function handleMessageDelete(messageId: string) {
  // Remove from main channels
  setMessagesByChannel(
    produce((channels) => {
      for (const cid in channels) {
        const messages = channels[cid];
        if (messages) {
          const index = messages.findIndex((m) => m.id === messageId);
          if (index !== -1) messages.splice(index, 1);
        }
      }
    })
  );

  // Remove from cached threads
  setRepliesByThread(
    produce((threads) => {
      for (const rootId in threads) {
        const replies = threads[rootId];
        if (replies) {
          const index = replies.findIndex((m) => m.id === messageId);
          if (index !== -1) replies.splice(index, 1);
        }
      }
    })
  );
}

export function handleReactionAdded(data: { post_id: string; user_id: string; emoji_name: string }) {
  const updateReactions = (msg: Message | undefined) => {
    if (!msg) return;

    const existing = msg.reactions.find((r) => r.emoji === data.emoji_name);
    if (existing) {
      if (!existing.users.includes(data.user_id)) {
        existing.users.push(data.user_id);
        existing.count++;
      }
    } else {
      msg.reactions.push({
        emoji: data.emoji_name,
        count: 1,
        users: [data.user_id],
      });
    }
  };

  setMessagesByChannel(
    produce((channels) => {
      for (const cid in channels) {
        const msg = channels[cid]?.find((m) => m.id === data.post_id);
        if (msg) updateReactions(msg);
      }
    })
  );

  setRepliesByThread(
    produce((threads) => {
      for (const rootId in threads) {
        const msg = threads[rootId]?.find((m) => m.id === data.post_id);
        if (msg) updateReactions(msg);
      }
    })
  );
}

export function handleReactionRemoved(data: { post_id: string; user_id: string; emoji_name: string }) {
  const updateReactions = (msg: Message | undefined) => {
    if (!msg) return;

    const reactionIndex = msg.reactions.findIndex((r) => r.emoji === data.emoji_name);
    if (reactionIndex === -1) return;

    const reaction = msg.reactions[reactionIndex];
    const userIndex = reaction.users.indexOf(data.user_id);
    if (userIndex !== -1) {
      reaction.users.splice(userIndex, 1);
      reaction.count--;
      if (reaction.count <= 0) {
        msg.reactions.splice(reactionIndex, 1);
      }
    }
  };

  setMessagesByChannel(
    produce((channels) => {
      for (const cid in channels) {
        const msg = channels[cid]?.find((m) => m.id === data.post_id);
        if (msg) updateReactions(msg);
      }
    })
  );

  setRepliesByThread(
    produce((threads) => {
      for (const rootId in threads) {
        const msg = threads[rootId]?.find((m) => m.id === data.post_id);
        if (msg) updateReactions(msg);
      }
    })
  );
}

// ============================================
// Message Actions
// ============================================

export async function pinMessage(messageId: string): Promise<void> {
  const token = authStore.token;
  const response = await fetch(`/api/v1/posts/${encodeURIComponent(messageId)}/pin`, {
    method: 'POST',
    headers: token ? { Authorization: `Bearer ${token}` } : {},
  });

  if (!response.ok) throw new Error('Failed to pin message');

  // Optimistic update
  handleMessageUpdate({ id: messageId, isPinned: true });
}

export async function unpinMessage(messageId: string): Promise<void> {
  const token = authStore.token;
  const response = await fetch(`/api/v1/posts/${encodeURIComponent(messageId)}/pin`, {
    method: 'DELETE',
    headers: token ? { Authorization: `Bearer ${token}` } : {},
  });

  if (!response.ok) throw new Error('Failed to unpin message');

  // Optimistic update
  handleMessageUpdate({ id: messageId, isPinned: false });
}

export async function saveMessage(messageId: string): Promise<void> {
  const token = authStore.token;
  const response = await fetch(`/api/v1/posts/${encodeURIComponent(messageId)}/save`, {
    method: 'POST',
    headers: token ? { Authorization: `Bearer ${token}` } : {},
  });

  if (!response.ok) throw new Error('Failed to save message');
}

export async function unsaveMessage(messageId: string): Promise<void> {
  const token = authStore.token;
  const response = await fetch(`/api/v1/posts/${encodeURIComponent(messageId)}/save`, {
    method: 'DELETE',
    headers: token ? { Authorization: `Bearer ${token}` } : {},
  });

  if (!response.ok) throw new Error('Failed to unsave message');
}

export async function editMessage(messageId: string, newContent: string): Promise<void> {
  const token = authStore.token;
  const response = await fetch(`/api/v1/posts/${encodeURIComponent(messageId)}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ message: newContent }),
  });

  if (!response.ok) throw new Error('Failed to edit message');

  const post: Post = await response.json();
  handleMessageUpdate({ id: messageId, content: post.message, editedAt: toOptionalIsoTimestamp(post.edited_at ?? post.edit_at) });
}

export async function deleteMessage(messageId: string): Promise<void> {
  const token = authStore.token;
  const response = await fetch(`/api/v1/posts/${encodeURIComponent(messageId)}`, {
    method: 'DELETE',
    headers: token ? { Authorization: `Bearer ${token}` } : {},
  });

  if (!response.ok) throw new Error('Failed to delete message');

  handleMessageDelete(messageId);
}

export async function addReaction(messageId: string, emoji: string): Promise<void> {
  const token = authStore.token;
  const response = await fetch(`/api/v1/posts/${encodeURIComponent(messageId)}/reactions`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ emoji_name: emoji }),
  });

  if (!response.ok) throw new Error('Failed to add reaction');
}

export async function removeReaction(messageId: string, emoji: string): Promise<void> {
  const token = authStore.token;
  const response = await fetch(
    `/api/v1/posts/${encodeURIComponent(messageId)}/reactions/${encodeURIComponent(emoji)}`,
    {
      method: 'DELETE',
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    }
  );

  if (!response.ok) throw new Error('Failed to remove reaction');
}

// ============================================
// Clear State
// ============================================

export function clearMessages(channelId?: string) {
  if (channelId) {
    setMessagesByChannel(produce((channels) => { delete channels[channelId]; }));
  } else {
    setMessagesByChannel({});
  }
}

export function resetSessionState() {
  setMessagesByChannel({});
  setRepliesByThread({});
  setHasMoreOlderByChannel({});
  setIsLoading(false);
  setIsLoadingOlder(false);
  setError(null);
}

// ============================================
// Exports
// ============================================

export const messageStore = {
  // State
  messagesByChannel,
  repliesByThread,
  hasMoreOlderByChannel,
  readStatesByChannel,
  isLoading,
  isLoadingOlder,
  error,

  // Getters
  getMessages,
  getReplies,
  hasMoreOlder,

  // Actions
  fetchMessages,
  fetchOlderMessages,
  fetchThread,
  sendMessage,
  addOptimisticMessage,
  updateOptimisticMessage,
  handleNewMessage,
  handleMessageUpdate,
  handleMessageDelete,
  handleReactionAdded,
  handleReactionRemoved,
  pinMessage,
  unpinMessage,
  saveMessage,
  unsaveMessage,
  editMessage,
  deleteMessage,
  addReaction,
  removeReaction,
  clearMessages,
  resetSessionState,
};

export default messageStore;
