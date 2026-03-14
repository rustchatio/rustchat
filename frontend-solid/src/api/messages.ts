// ============================================
// Messages/Posts API Methods
// ============================================

import { client } from './client';
import type {
  Post,
  PostListResponse,
  CreatePostRequest,
  UpdatePostRequest,
  Reaction,
  ChannelUnreadAt,
  FileInfo,
} from './types';

// ============================================
// Post CRUD
// ============================================

export interface ListPostsParams {
  before?: string;
  after?: string;
  limit?: number;
  is_pinned?: boolean;
  q?: string;
}

export async function listPosts(channelId: string, params?: ListPostsParams): Promise<PostListResponse> {
  const response = await client.get<PostListResponse>(`/channels/${channelId}/posts`, { params });
  return response.data;
}

export async function getPost(id: string): Promise<Post> {
  const response = await client.get<Post>(`/posts/${id}`);
  return response.data;
}

export async function createPost(data: CreatePostRequest): Promise<Post> {
  const response = await client.post<Post>(`/channels/${data.channel_id}/posts`, data);
  return response.data;
}

export async function updatePost(id: string, data: UpdatePostRequest): Promise<Post> {
  const response = await client.put<Post>(`/posts/${id}`, data);
  return response.data;
}

export async function patchPost(id: string, data: Partial<UpdatePostRequest>): Promise<Post> {
  const response = await client.patch<Post>(`/posts/${id}`, data);
  return response.data;
}

export async function deletePost(id: string): Promise<void> {
  await client.delete(`/posts/${id}`);
}

// ============================================
// Threads
// ============================================

export async function getThread(rootId: string): Promise<Post[]> {
  const response = await client.get<Post[]>(`/posts/${rootId}/thread`);
  return response.data;
}

export async function getThreadsForUser(
  userId: string,
  params?: { since?: number; before?: string; after?: string; limit?: number; extended?: boolean }
): Promise<{ threads: unknown[]; total_unread_threads: number; total_unread_mentions: number }> {
  const response = await client.get(`/users/${userId}/threads`, { params });
  return response.data;
}

export async function updateThreadRead(userId: string, teamId: string, threadId: string, timestamp: number): Promise<void> {
  await client.put(`/users/${userId}/teams/${teamId}/threads/${threadId}/read`, { timestamp });
}

export async function followThread(userId: string, teamId: string, threadId: string): Promise<void> {
  await client.put(`/users/${userId}/teams/${teamId}/threads/${threadId}/following`);
}

export async function unfollowThread(userId: string, teamId: string, threadId: string): Promise<void> {
  await client.delete(`/users/${userId}/teams/${teamId}/threads/${threadId}/following`);
}

// ============================================
// Pin/Save
// ============================================

export async function pinPost(id: string): Promise<void> {
  await client.post(`/posts/${id}/pin`);
}

export async function unpinPost(id: string): Promise<void> {
  await client.delete(`/posts/${id}/pin`);
}

export async function savePost(id: string): Promise<void> {
  await client.post(`/posts/${id}/save`);
}

export async function unsavePost(id: string): Promise<void> {
  await client.delete(`/posts/${id}/save`);
}

export async function getSavedPosts(userId?: string): Promise<Post[]> {
  const response = await client.get<Post[]>(userId ? `/users/${userId}/posts/saved` : '/active_user/saved_posts');
  return response.data;
}

export async function getPinnedPosts(channelId: string): Promise<Post[]> {
  const response = await listPosts(channelId, { is_pinned: true });
  return response.messages;
}

// ============================================
// Reactions
// ============================================

export async function addReaction(postId: string, emoji: string): Promise<Reaction> {
  const response = await client.post<Reaction>(`/posts/${postId}/reactions`, { emoji_name: emoji });
  return response.data;
}

export async function removeReaction(postId: string, emoji: string): Promise<void> {
  await client.delete(`/posts/${postId}/reactions/${encodeURIComponent(emoji)}`);
}

export async function getReactions(postId: string): Promise<Reaction[]> {
  const response = await client.get<Reaction[]>(`/posts/${postId}/reactions`);
  return response.data;
}

// ============================================
// Files
// ============================================

export async function getFile(fileId: string): Promise<Blob> {
  const response = await client.get<Blob>(`/files/${fileId}`, { responseType: 'blob' });
  return response.data;
}

export async function getFileInfo(fileId: string): Promise<FileInfo> {
  const response = await client.get<FileInfo>(`/files/${fileId}/info`);
  return response.data;
}

export async function getFilePreview(fileId: string): Promise<Blob> {
  const response = await client.get<Blob>(`/files/${fileId}/preview`, { responseType: 'blob' });
  return response.data;
}

export async function getFileThumbnail(fileId: string): Promise<Blob> {
  const response = await client.get<Blob>(`/files/${fileId}/thumbnail`, { responseType: 'blob' });
  return response.data;
}

export async function uploadFiles(
  channelId: string,
  files: File[],
  clientIds?: string[]
): Promise<{ file_infos: FileInfo[]; client_ids: string[] }> {
  const formData = new FormData();
  files.forEach((file) => formData.append('files', file));
  if (clientIds) {
    clientIds.forEach((id) => formData.append('client_ids', id));
  }

  const response = await client.post<{ file_infos: FileInfo[]; client_ids: string[] }>(
    `/files/channels/${channelId}`,
    formData,
    {
      headers: { 'Content-Type': 'multipart/form-data' },
    }
  );
  return response.data;
}

// ============================================
// Unread
// ============================================

export async function setUnreadFromPost(
  userId: string,
  postId: string,
  collapsedThreadsSupported = true
): Promise<ChannelUnreadAt> {
  const response = await client.post<ChannelUnreadAt>(`/users/${userId}/posts/${postId}/set_unread`, {
    collapsed_threads_supported: collapsedThreadsSupported,
  });
  return response.data;
}

// ============================================
// Search
// ============================================

export async function searchPosts(
  teamId: string,
  terms: string,
  params?: { is_or_search?: boolean; time_zone_offset?: number; page?: number; per_page?: number }
): Promise<{ posts: Post[]; order: string[] }> {
  const response = await client.post<{ posts: Post[]; order: string[] }>(`/teams/${teamId}/posts/search`, {
    terms,
    ...params,
  });
  return response.data;
}

// ============================================
// Posts API Object
// ============================================

export const postsApi = {
  list: listPosts,
  get: getPost,
  create: createPost,
  update: updatePost,
  patch: patchPost,
  delete: deletePost,
  getThread,
  getThreadsForUser,
  updateThreadRead,
  followThread,
  unfollowThread,
  pin: pinPost,
  unpin: unpinPost,
  save: savePost,
  unsave: unsavePost,
  getSaved: getSavedPosts,
  getPinned: getPinnedPosts,
  addReaction,
  removeReaction,
  getReactions,
  getFile,
  getFileInfo,
  getFilePreview,
  getFileThumbnail,
  uploadFiles,
  setUnreadFromPost,
  search: searchPosts,
};

export default postsApi;
