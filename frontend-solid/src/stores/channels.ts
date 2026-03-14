import { createStore, produce } from 'solid-js/store';
import { createSignal, createEffect, batch, createMemo } from 'solid-js';
import { authStore } from './auth';

// ============================================
// Types
// ============================================

export type ChannelType = 'public' | 'private' | 'direct' | 'group';

export interface Channel {
  id: string;
  team_id: string;
  name: string;
  display_name: string;
  channel_type: ChannelType;
  header?: string;
  purpose?: string;
  unreadCount?: number;
  mentionCount?: number;
  created_at: string;
  creator_id: string;
}

export interface ChannelMember {
  channel_id: string;
  user_id: string;
  roles: string;
  last_viewed_at: number;
  msg_count: number;
  mention_count: number;
  mention_count_root: number;
  urgent_mention_count: number;
  msg_count_root: number;
  notify_props: ChannelNotifyProps;
  last_update_at: number;
}

export interface ChannelNotifyProps {
  desktop?: string;
  mobile?: string;
  mark_unread?: string;
  ignore_channel_mentions?: string;
}

export interface CreateChannelRequest {
  team_id: string;
  name: string;
  display_name: string;
  channel_type: ChannelType;
  header?: string;
  purpose?: string;
  target_user_id?: string;
}

// ============================================
// Store State
// ============================================

const STORAGE_KEY = 'rustchat_last_channels';

const [channels, setChannels] = createStore<Channel[]>([]);
const [joinableChannels, setJoinableChannels] = createStore<Channel[]>([]);
const [currentChannelId, setCurrentChannelId] = createSignal<string | null>(null);
const [membersByChannel, setMembersByChannel] = createStore<Record<string, ChannelMember[]>>({});
const [isLoading, setIsLoading] = createSignal(false);
const [error, setError] = createSignal<string | null>(null);

// Persist last channel per team
const getStoredLastChannels = (): Record<string, string> => {
  if (typeof window === 'undefined') return {};
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored ? JSON.parse(stored) : {};
  } catch {
    return {};
  }
};

const [lastChannelByTeam, setLastChannelByTeam] = createStore<Record<string, string>>(getStoredLastChannels());

createEffect(() => {
  if (typeof window !== 'undefined') {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ ...lastChannelByTeam }));
  }
});

// ============================================
// Computed
// ============================================

export const currentChannel = createMemo(() => {
  const id = currentChannelId();
  return channels.find((c) => c.id === id) || null;
});

export const publicChannels = createMemo(() =>
  channels.filter((c) => c.channel_type === 'public')
);

export const privateChannels = createMemo(() =>
  channels.filter((c) => c.channel_type === 'private')
);

export const directMessages = createMemo(() =>
  channels.filter((c) => c.channel_type === 'direct' || c.channel_type === 'group')
);

export const favoriteChannels = createMemo(() =>
  channels.filter((c) => c.channel_type === 'public' || c.channel_type === 'private')
);

// ============================================
// Actions
// ============================================

export async function fetchChannels(teamId: string): Promise<void> {
  batch(() => {
    setIsLoading(true);
    setError(null);
  });

  try {
    const token = authStore.token;
    const response = await fetch(`/api/v1/channels?team_id=${encodeURIComponent(teamId)}`, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    });

    if (!response.ok) throw new Error('Failed to fetch channels');

    const data: Channel[] = await response.json();
    setChannels(data);

    // Try to restore last selected channel for this team
    const lastId = lastChannelByTeam[teamId];
    if (lastId && data.some((c) => c.id === lastId)) {
      setCurrentChannelId(lastId);
    } else {
      // Auto-select general channel if none selected or last not found
      const general = data.find((c) => c.name === 'general');
      const defaultId = general?.id || data[0]?.id || null;
      setCurrentChannelId(defaultId);

      // Save this default selection
      if (defaultId) {
        setLastChannelByTeam(teamId, defaultId);
      }
    }
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to fetch channels');
    throw err;
  } finally {
    setIsLoading(false);
  }
}

export async function fetchJoinableChannels(teamId: string): Promise<void> {
  setIsLoading(true);

  try {
    const token = authStore.token;
    const response = await fetch(
      `/api/v1/channels?team_id=${encodeURIComponent(teamId)}&available_to_join=true`,
      {
        headers: token ? { Authorization: `Bearer ${token}` } : {},
      }
    );

    if (!response.ok) throw new Error('Failed to fetch joinable channels');

    const data: Channel[] = await response.json();
    setJoinableChannels(data);
  } catch (err) {
    console.error('Failed to fetch joinable channels', err);
  } finally {
    setIsLoading(false);
  }
}

export async function fetchChannel(channelId: string): Promise<Channel> {
  const token = authStore.token;
  const response = await fetch(`/api/v1/channels/${encodeURIComponent(channelId)}`, {
    headers: token ? { Authorization: `Bearer ${token}` } : {},
  });

  if (!response.ok) throw new Error('Failed to fetch channel');

  const data: Channel = await response.json();

  setChannels(
    produce((list) => {
      const index = list.findIndex((c) => c.id === data.id);
      if (index !== -1) {
        list[index] = data;
      } else {
        list.push(data);
      }
    })
  );

  return data;
}

export async function createChannel(data: CreateChannelRequest): Promise<Channel> {
  batch(() => {
    setIsLoading(true);
    setError(null);
  });

  try {
    const token = authStore.token;
    const response = await fetch('/api/v1/channels', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.message || 'Failed to create channel');
    }

    const channel: Channel = await response.json();

    setChannels((prev) => [...prev, channel]);
    setCurrentChannelId(channel.id);
    setLastChannelByTeam(channel.team_id, channel.id);

    return channel;
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to create channel');
    throw err;
  } finally {
    setIsLoading(false);
  }
}

export async function joinChannel(channelId: string): Promise<void> {
  const userId = authStore.user()?.id;
  if (!userId) throw new Error('User not authenticated');

  const token = authStore.token;
  const response = await fetch(`/api/v1/channels/${encodeURIComponent(channelId)}/members`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${token}`,
    },
    body: JSON.stringify({ user_id: userId }),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.message || 'Failed to join channel');
  }
}

export async function leaveChannel(channelId: string): Promise<void> {
  const userId = authStore.user()?.id;
  if (!userId) throw new Error('User not authenticated');

  const token = authStore.token;
  const response = await fetch(
    `/api/v1/channels/${encodeURIComponent(channelId)}/members/${encodeURIComponent(userId)}`,
    {
      method: 'DELETE',
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    }
  );

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.message || 'Failed to leave channel');
  }

  setChannels((prev) => prev.filter((c) => c.id !== channelId));

  if (currentChannelId() === channelId) {
    setCurrentChannelId(channels[0]?.id || null);
  }
}

export function selectChannel(channelId: string) {
  const channel = channels.find((c) => c.id === channelId);
  if (channel) {
    setCurrentChannelId(channelId);
    setLastChannelByTeam(channel.team_id, channelId);
  }
}

export async function fetchChannelMembers(channelId: string): Promise<ChannelMember[]> {
  const token = authStore.token;
  const response = await fetch(`/api/v1/channels/${encodeURIComponent(channelId)}/members`, {
    headers: token ? { Authorization: `Bearer ${token}` } : {},
  });

  if (!response.ok) throw new Error('Failed to fetch channel members');

  const data: ChannelMember[] = await response.json();

  setMembersByChannel(channelId, data);
  return data;
}

export async function updateNotifyProps(
  channelId: string,
  userId: string,
  props: ChannelNotifyProps
): Promise<void> {
  const token = authStore.token;
  const response = await fetch(
    `/api/v1/channels/${encodeURIComponent(channelId)}/members/${encodeURIComponent(userId)}/notify_props`,
    {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify(props),
    }
  );

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.message || 'Failed to update notification settings');
  }
}

// ============================================
// Update Helpers (for WebSocket events)
// ============================================

export function addChannel(channel: Channel) {
  setChannels(
    produce((list) => {
      const index = list.findIndex((c) => c.id === channel.id);
      if (index !== -1) {
        list[index] = channel;
      } else {
        list.push(channel);
      }
    })
  );
}

export function updateChannel(updated: Channel) {
  setChannels(
    produce((list) => {
      const index = list.findIndex((c) => c.id === updated.id);
      if (index !== -1) {
        list[index] = updated;
      }
    })
  );
}

export function removeChannel(channelId: string) {
  setChannels((prev) => prev.filter((c) => c.id !== channelId));
  if (currentChannelId() === channelId) {
    setCurrentChannelId(channels[0]?.id || null);
  }
}

export function clearChannels() {
  setChannels([]);
  setCurrentChannelId(null);
}

export function incrementUnread(channelId: string) {
  setChannels(
    produce((list) => {
      const channel = list.find((c) => c.id === channelId);
      if (channel) {
        channel.unreadCount = (channel.unreadCount || 0) + 1;
      }
    })
  );
}

export function incrementMention(channelId: string) {
  setChannels(
    produce((list) => {
      const channel = list.find((c) => c.id === channelId);
      if (channel) {
        channel.mentionCount = (channel.mentionCount || 0) + 1;
      }
    })
  );
}

export function clearCounts(channelId: string) {
  setChannels(
    produce((list) => {
      const channel = list.find((c) => c.id === channelId);
      if (channel) {
        channel.unreadCount = 0;
        channel.mentionCount = 0;
      }
    })
  );
}

// ============================================
// Exports
// ============================================

export const channelStore = {
  // State
  channels,
  joinableChannels,
  currentChannelId,
  currentChannel,
  membersByChannel,
  isLoading,
  error,

  // Computed
  publicChannels,
  privateChannels,
  directMessages,
  favoriteChannels,

  // Actions
  fetchChannels,
  fetchChannel,
  fetchJoinableChannels,
  createChannel,
  joinChannel,
  leaveChannel,
  selectChannel,
  fetchChannelMembers,
  updateNotifyProps,

  // Update helpers
  addChannel,
  updateChannel,
  removeChannel,
  clearChannels,
  incrementUnread,
  incrementMention,
  clearCounts,
};

export default channelStore;
