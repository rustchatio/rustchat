import { createStore, produce } from 'solid-js/store';
import { createSignal, batch, createMemo } from 'solid-js';
import { authStore } from './auth';
import { API_BASE_URL } from '@/api/client';

// ============================================
// Types
// ============================================

export interface ChannelUnread {
  channel_id: string;
  team_id: string;
  unread_count: number;
  mention_count: number;
}

export interface TeamUnread {
  team_id: string;
  unread_count: number;
}

export interface UnreadOverview {
  channels: ChannelUnread[];
  teams: TeamUnread[];
}

export interface ReadState {
  last_read_message_id: number | null;
  first_unread_message_id: number | null;
}

export interface ChannelUnreadAt {
  team_id: string;
  user_id: string;
  channel_id: string;
  msg_count: number;
  mention_count: number;
  mention_count_root: number;
  urgent_mention_count: number;
  msg_count_root: number;
  last_viewed_at: number;
}

// ============================================
// Store State
// ============================================

const [channelUnreads, setChannelUnreads] = createStore<Record<string, number>>({});
const [teamUnreads, setTeamUnreads] = createStore<Record<string, number>>({});
const [channelMentions, setChannelMentions] = createStore<Record<string, number>>({});
const [channelReadStates, setChannelReadStates] = createStore<Record<string, ReadState>>({});
const [isLoading, setIsLoading] = createSignal(false);
const [error, setError] = createSignal<string | null>(null);
const API_V1_BASE = API_BASE_URL.replace(/\/+$/, '');
const API_V4_BASE = API_V1_BASE.includes('/api/v1')
  ? API_V1_BASE.replace('/api/v1', '/api/v4')
  : '/api/v4';

function apiUrl(base: string, path: string): string {
  const normalizedPath = path.startsWith('/') ? path : `/${path}`;
  return `${base}${normalizedPath}`;
}

// ============================================
// Computed
// ============================================

export const totalUnreadCount = createMemo(() =>
  Object.values(channelUnreads).reduce((a, b) => a + b, 0)
);

export const totalMentionCount = createMemo(() =>
  Object.values(channelMentions).reduce((a, b) => a + b, 0)
);

export function getChannelUnreadCount(channelId: string) {
  return createMemo(() => channelUnreads[channelId] || 0);
}

export function getTeamUnreadCount(teamId: string) {
  return createMemo(() => teamUnreads[teamId] || 0);
}

export function getChannelMentionCount(channelId: string) {
  return createMemo(() => channelMentions[channelId] || 0);
}

export function getChannelReadState(channelId: string) {
  return createMemo(() => channelReadStates[channelId]);
}

export function hasUnread(channelId: string) {
  return createMemo(() => (channelUnreads[channelId] || 0) > 0);
}

export function hasMentions(channelId: string) {
  return createMemo(() => (channelMentions[channelId] || 0) > 0);
}

// ============================================
// Actions
// ============================================

export async function fetchOverview(): Promise<void> {
  setIsLoading(true);

  try {
    const token = authStore.token;
    const response = await fetch(apiUrl(API_V1_BASE, '/unreads/overview'), {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    });

    if (!response.ok) throw new Error('Failed to fetch unread overview');

    const data: UnreadOverview = await response.json();

    batch(() => {
      // Reset and populate
      const newChannelUnreads: Record<string, number> = {};
      const newTeamUnreads: Record<string, number> = {};
      const newChannelMentions: Record<string, number> = {};

      data.channels.forEach((c) => {
        newChannelUnreads[c.channel_id] = c.unread_count;
        newChannelMentions[c.channel_id] = c.mention_count || 0;
      });

      data.teams.forEach((t) => {
        newTeamUnreads[t.team_id] = t.unread_count;
      });

      setChannelUnreads(newChannelUnreads);
      setTeamUnreads(newTeamUnreads);
      setChannelMentions(newChannelMentions);
    });
  } catch (err) {
    console.error('Failed to fetch unread overview:', err);
    setError(err instanceof Error ? err.message : 'Failed to fetch unread overview');
  } finally {
    setIsLoading(false);
  }
}

export async function markAsRead(channelId: string, userId: string = 'me'): Promise<void> {
  try {
    const token = authStore.token;
    const response = await fetch(
      apiUrl(API_V4_BASE, `/channels/${encodeURIComponent(channelId)}/members/${encodeURIComponent(userId)}/read`),
      {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify({}),
      }
    );

    if (!response.ok) throw new Error('Failed to mark as read');

    // Optimistic update
    batch(() => {
      setChannelUnreads(channelId, 0);
      setChannelMentions(channelId, 0);
    });

    // Clear the "new messages" line state locally too
    setChannelReadStates(
      produce((states) => {
        if (states[channelId]) {
          states[channelId] = {
            last_read_message_id: null,
            first_unread_message_id: null,
          };
        }
      })
    );
  } catch (err) {
    console.error('Failed to mark channel as read:', err);
  }
}

export async function markAsUnread(channelId: string, userId: string = 'me'): Promise<void> {
  try {
    const token = authStore.token;
    const response = await fetch(
      apiUrl(API_V4_BASE, `/channels/${encodeURIComponent(channelId)}/members/${encodeURIComponent(userId)}/set_unread`),
      {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify({}),
      }
    );

    if (!response.ok) throw new Error('Failed to mark as unread');

    // Optimistic update
    setChannelUnreads(channelId, 1);

    // Refresh overview to get accurate counts
    await fetchOverview();
  } catch (err) {
    console.error('Failed to mark channel as unread:', err);
  }
}

export async function markAsUnreadFromPost(postId: string, userId: string = 'me'): Promise<void> {
  try {
    const token = authStore.token;
    const response = await fetch(
      apiUrl(API_V4_BASE, `/users/${encodeURIComponent(userId)}/posts/${encodeURIComponent(postId)}/set_unread`),
      {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify({ collapsed_threads_supported: true }),
      }
    );

    if (!response.ok) throw new Error('Failed to mark post as unread');

    const data: ChannelUnreadAt = await response.json();
    applyPostUnread(data);
  } catch (err) {
    console.error('Failed to mark post as unread:', err);
    throw err;
  }
}

export async function markAllAsRead(): Promise<void> {
  try {
    const token = authStore.token;
    const response = await fetch(apiUrl(API_V1_BASE, '/unreads/mark_all_read'), {
      method: 'POST',
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    });

    if (!response.ok) throw new Error('Failed to mark all as read');

    batch(() => {
      setChannelUnreads({});
      setTeamUnreads({});
      setChannelMentions({});
    });
  } catch (err) {
    console.error('Failed to mark all as read:', err);
  }
}

// ============================================
// State Setters (for WebSocket events)
// ============================================

export function setReadState(channelId: string, state: ReadState) {
  setChannelReadStates(channelId, state);
}

export function handleUnreadUpdate(data: { channel_id: string; team_id: string; unread_count: number }) {
  setChannelUnreads(data.channel_id, data.unread_count);
  // Team unread count update: if we want to be accurate we should probably re-fetch or track team mappings
}

export function applyPostUnread(data: ChannelUnreadAt) {
  const mentionCount = Number.isFinite(data.mention_count) ? data.mention_count : 0;
  setChannelMentions(data.channel_id, mentionCount);
  // Refresh overview to get accurate counts
  void fetchOverview();
}

export function incrementChannelUnread(channelId: string) {
  setChannelUnreads(
    produce((unreads) => {
      unreads[channelId] = (unreads[channelId] || 0) + 1;
    })
  );
}

export function incrementChannelMention(channelId: string) {
  setChannelMentions(
    produce((mentions) => {
      mentions[channelId] = (mentions[channelId] || 0) + 1;
    })
  );
}

// ============================================
// Clear State
// ============================================

export function clearAllState() {
  batch(() => {
    setChannelUnreads({});
    setTeamUnreads({});
    setChannelMentions({});
    setChannelReadStates({});
  });
}

// ============================================
// Exports
// ============================================

export const unreadStore = {
  // State
  channelUnreads,
  teamUnreads,
  channelMentions,
  channelReadStates,
  isLoading,
  error,

  // Computed
  totalUnreadCount,
  totalMentionCount,
  getChannelUnreadCount,
  getTeamUnreadCount,
  getChannelMentionCount,
  getChannelReadState,
  hasUnread,
  hasMentions,

  // Actions
  fetchOverview,
  markAsRead,
  markAsUnread,
  markAsUnreadFromPost,
  markAllAsRead,

  // WebSocket handlers
  setReadState,
  handleUnreadUpdate,
  applyPostUnread,
  incrementChannelUnread,
  incrementChannelMention,

  // Clear
  clearAllState,
};

export default unreadStore;
