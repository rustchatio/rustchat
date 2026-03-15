import { createStore, produce } from 'solid-js/store';
import { createSignal, createEffect, batch, createMemo } from 'solid-js';
import { authStore } from './auth';
import { channelsApi } from '@/api/channels';
import { client, getErrorMessage } from '@/api/client';

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
  role?: string;
  roles?: string;
  username?: string;
  display_name?: string;
  avatar_url?: string;
  presence?: string;
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
  desktop?: 'default' | 'all' | 'mention' | 'none';
  mobile?: 'default' | 'all' | 'mention' | 'none';
  mark_unread?: 'all' | 'mention';
  ignore_channel_mentions?: 'default' | 'on' | 'off';
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

interface TeamSummary {
  id: string;
  name?: string;
  display_name?: string | null;
}

// ============================================
// Store State
// ============================================

const STORAGE_KEY = 'rustchat_last_channels';
const DEFAULT_TEAM_NAME = 'rustchat';
const DEFAULT_TEAM_DISPLAY_NAME = 'RustChat';

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
    const data = await channelsApi.list(teamId);
    setChannels(data);

    // Try to restore last selected channel for this team
    const lastId = lastChannelByTeam[teamId];
    if (lastId && data.some((c) => c.id === lastId)) {
      setCurrentChannelId(lastId);
    } else {
      // Prefer Mattermost-style defaults before generic fallbacks.
      const preferred =
        data.find((c) => c.name === 'town-square') ||
        data.find((c) => c.name === 'off-topic') ||
        data.find((c) => c.name === 'general');
      const defaultId = preferred?.id || data[0]?.id || null;
      setCurrentChannelId(defaultId);

      // Save this default selection
      if (defaultId) {
        setLastChannelByTeam(teamId, defaultId);
      }
    }
  } catch (err) {
    setError(getErrorMessage(err) || 'Failed to fetch channels');
    throw err;
  } finally {
    setIsLoading(false);
  }
}

function normalizeTeamSummaries(payload: unknown): TeamSummary[] {
  if (!Array.isArray(payload)) return [];
  return payload.filter((team): team is TeamSummary => {
    const candidate = team as Partial<TeamSummary> | null;
    return typeof candidate?.id === 'string';
  });
}

async function fetchUserTeams(): Promise<TeamSummary[]> {
  if (!authStore.token) return [];

  const requestTeams = async () => {
    const response = await client.get<TeamSummary[]>('/teams');
    return normalizeTeamSummaries(response.data);
  };

  let payload = await requestTeams();
  if (payload.length > 0) {
    return payload;
  }

  // Trigger auth workspace bootstrap for older sessions/users, then retry.
  await client.get('/auth/me');
  payload = await requestTeams();
  if (payload.length > 0) {
    return payload;
  }

  // Recovery path: auto-join an open public team first (prefer rustchat).
  try {
    const publicResponse = await client.get<TeamSummary[]>('/teams/public');
    const publicTeams = normalizeTeamSummaries(publicResponse.data);
    const joinTarget =
      publicTeams.find((team) => team.name?.toLowerCase() === DEFAULT_TEAM_NAME) || publicTeams[0];

    if (joinTarget) {
      await client.post(`/teams/${joinTarget.id}/join`);
      payload = await requestTeams();
      if (payload.length > 0) {
        return payload;
      }
    }
  } catch (error) {
    console.warn('Failed to auto-join public team during bootstrap', error);
  }

  // Final recovery: create a default workspace team for the signed-in user.
  try {
    await client.post('/teams', {
      name: DEFAULT_TEAM_NAME,
      display_name: DEFAULT_TEAM_DISPLAY_NAME,
      description: 'Default RustChat workspace',
    });
  } catch (error) {
    const conflictName = `${DEFAULT_TEAM_NAME}-${Date.now().toString(36)}`;
    try {
      await client.post('/teams', {
        name: conflictName,
        display_name: DEFAULT_TEAM_DISPLAY_NAME,
        description: 'Default RustChat workspace',
      });
    } catch (fallbackError) {
      console.warn('Failed to auto-create default team during bootstrap', error, fallbackError);
    }
  }

  return requestTeams();
}

export async function resolveDefaultChannelPath(): Promise<string | null> {
  try {
    const teams = await fetchUserTeams();
    if (teams.length === 0) return null;

    const knownTeamIds = new Set(teams.map((team) => team.id));
    const preferredTeamIds = Object.keys(lastChannelByTeam).filter((teamId) => knownTeamIds.has(teamId));
    const fallbackTeamIds = teams.map((team) => team.id).filter((teamId) => !preferredTeamIds.includes(teamId));
    const orderedTeamIds = [...preferredTeamIds, ...fallbackTeamIds];

    for (const teamId of orderedTeamIds) {
      try {
        await fetchChannels(teamId);
      } catch {
        continue;
      }

      let channelId = currentChannelId();
      if (!channelId) {
        try {
          await fetchJoinableChannels(teamId);
          const firstJoinable = channelStore.joinableChannels[0];
          if (firstJoinable) {
            await joinChannel(firstJoinable.id);
            await fetchChannels(teamId);
            channelId = currentChannelId();
          }
        } catch {
          // Ignore join recovery errors and continue trying remaining teams.
        }
      }

      if (channelId) {
        return `/channels/${channelId}`;
      }
    }
  } catch (error) {
    console.error('Failed to resolve default channel path', error);
  }

  return null;
}

export async function fetchJoinableChannels(teamId: string): Promise<void> {
  setIsLoading(true);

  try {
    const data = await channelsApi.list(teamId, true);
    setJoinableChannels(data);
  } catch (err) {
    console.error('Failed to fetch joinable channels', err);
  } finally {
    setIsLoading(false);
  }
}

export async function fetchChannel(channelId: string): Promise<Channel> {
  const data = await channelsApi.get(channelId);

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
    const channel = await channelsApi.create(data);

    setChannels((prev) => [...prev, channel]);
    setCurrentChannelId(channel.id);
    setLastChannelByTeam(channel.team_id, channel.id);

    return channel;
  } catch (err) {
    const message = getErrorMessage(err) || 'Failed to create channel';
    setError(message);
    throw new Error(message);
  } finally {
    setIsLoading(false);
  }
}

export async function joinChannel(channelId: string): Promise<void> {
  const userId = authStore.user()?.id;
  if (!userId) throw new Error('User not authenticated');

  await channelsApi.join(channelId, userId);
}

export async function leaveChannel(channelId: string): Promise<void> {
  const userId = authStore.user()?.id;
  if (!userId) throw new Error('User not authenticated');

  await channelsApi.leave(channelId, userId);

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
  const data = await channelsApi.getMembers(channelId);

  setMembersByChannel(channelId, data);
  return data;
}

export async function updateNotifyProps(
  channelId: string,
  userId: string,
  props: ChannelNotifyProps
): Promise<void> {
  await channelsApi.updateNotifyProps(channelId, userId, props);
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
  resolveDefaultChannelPath,
  fetchChannel,
  fetchJoinableChannels,
  createChannel,
  joinChannel,
  leaveChannel,
  selectChannel,
  fetchChannelMembers,
  updateNotifyProps,

  // Getters
  getChannel: (id: string) => channels.find((c) => c.id === id),

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
