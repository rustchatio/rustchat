// ============================================
// Channels API Methods
// ============================================

import { client } from './client';
import type {
  Channel,
  ChannelMember,
  ChannelNotifyProps,
  CreateChannelRequest,
  UpdateChannelRequest,
  SidebarCategories,
  SidebarCategory,
} from './types';

// ============================================
// Channel CRUD
// ============================================

export async function listChannels(teamId: string, availableToJoin = false): Promise<Channel[]> {
  const response = await client.get<Channel[]>('/channels', {
    params: { team_id: teamId, available_to_join: availableToJoin },
  });
  return response.data;
}

export async function getChannel(id: string): Promise<Channel> {
  const response = await client.get<Channel>(`/channels/${id}`);
  return response.data;
}

export async function getChannelByName(teamId: string, name: string): Promise<Channel> {
  const response = await client.get<Channel>(`/teams/${teamId}/channels/name/${name}`);
  return response.data;
}

export async function createChannel(data: CreateChannelRequest): Promise<Channel> {
  const response = await client.post<Channel>('/channels', data);
  return response.data;
}

export async function updateChannel(id: string, data: UpdateChannelRequest): Promise<Channel> {
  const response = await client.put<Channel>(`/channels/${id}`, data);
  return response.data;
}

export async function patchChannel(id: string, data: Partial<UpdateChannelRequest>): Promise<Channel> {
  const response = await client.patch<Channel>(`/channels/${id}`, data);
  return response.data;
}

export async function deleteChannel(id: string): Promise<void> {
  await client.delete(`/channels/${id}`);
}

export async function restoreChannel(id: string): Promise<Channel> {
  const response = await client.post<Channel>(`/channels/${id}/restore`);
  return response.data;
}

// ============================================
// Channel Members
// ============================================

export async function getChannelMembers(id: string): Promise<ChannelMember[]> {
  const response = await client.get<ChannelMember[]>(`/channels/${id}/members`);
  return response.data;
}

export async function getChannelMember(channelId: string, userId: string): Promise<ChannelMember> {
  const response = await client.get<ChannelMember>(`/channels/${channelId}/members/${userId}`);
  return response.data;
}

export async function joinChannel(channelId: string, userId: string): Promise<ChannelMember> {
  const response = await client.post<ChannelMember>(`/channels/${channelId}/members`, {
    user_id: userId,
  });
  return response.data;
}

export async function leaveChannel(channelId: string, userId: string): Promise<void> {
  await client.delete(`/channels/${channelId}/members/${userId}`);
}

export async function addChannelMember(channelId: string, userId: string): Promise<ChannelMember> {
  const response = await client.post<ChannelMember>(`/channels/${channelId}/members`, {
    user_id: userId,
  });
  return response.data;
}

export async function removeChannelMember(channelId: string, userId: string): Promise<void> {
  await client.delete(`/channels/${channelId}/members/${userId}`);
}

// ============================================
// Read/Unread
// ============================================

export async function markAsRead(channelId: string, userId: string = 'me'): Promise<void> {
  await client.post(`/channels/${channelId}/members/${userId}/read`, {});
}

export async function markAsUnread(channelId: string, userId: string = 'me'): Promise<void> {
  await client.post(`/channels/${channelId}/members/${userId}/set_unread`, {});
}

export async function viewChannel(
  channelId: string,
  prevChannelId?: string
): Promise<{ status: string; last_viewed_at: number }> {
  const response = await client.post<{ status: string; last_viewed_at: number }>(
    `/channels/members/me/view`,
    { channel_id: channelId, prev_channel_id: prevChannelId }
  );
  return response.data;
}

// ============================================
// Notifications
// ============================================

export async function updateNotifyProps(
  channelId: string,
  userId: string,
  props: ChannelNotifyProps
): Promise<void> {
  await client.put(`/channels/${channelId}/members/${userId}/notify_props`, props);
}

export async function getChannelUnread(channelId: string, userId: string): Promise<{
  channel_id: string;
  msg_count: number;
  mention_count: number;
}> {
  const response = await client.get(`/channels/${channelId}/members/${userId}/unread`);
  return response.data;
}

// ============================================
// Sidebar Categories
// ============================================

export async function getSidebarCategories(userId: string, teamId: string): Promise<SidebarCategories> {
  const response = await client.get<SidebarCategories>(
    `/users/${userId}/teams/${teamId}/channels/categories`
  );
  return response.data;
}

export async function updateSidebarCategories(
  userId: string,
  teamId: string,
  categories: SidebarCategory[]
): Promise<SidebarCategory[]> {
  const response = await client.put<SidebarCategory[]>(
    `/users/${userId}/teams/${teamId}/channels/categories`,
    categories
  );
  return response.data;
}

export async function updateSidebarCategory(
  userId: string,
  teamId: string,
  categoryId: string,
  category: Partial<SidebarCategory>
): Promise<SidebarCategory> {
  const response = await client.put<SidebarCategory>(
    `/users/${userId}/teams/${teamId}/channels/categories/${categoryId}`,
    category
  );
  return response.data;
}

// ============================================
// Channels API Object
// ============================================

export const channelsApi = {
  list: listChannels,
  get: getChannel,
  getByName: getChannelByName,
  create: createChannel,
  update: updateChannel,
  patch: patchChannel,
  delete: deleteChannel,
  restore: restoreChannel,
  getMembers: getChannelMembers,
  getMember: getChannelMember,
  join: joinChannel,
  leave: leaveChannel,
  addMember: addChannelMember,
  removeMember: removeChannelMember,
  markAsRead,
  markAsUnread,
  view: viewChannel,
  updateNotifyProps,
  getUnread: getChannelUnread,
  getSidebarCategories,
  updateSidebarCategories,
  updateSidebarCategory,
};

export default channelsApi;
