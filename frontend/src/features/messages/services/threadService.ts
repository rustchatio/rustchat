import { api } from '@/api/client'
import type { Post } from '@/types'

export interface ThreadQueryParams {
  cursor?: string
  limit?: number
}

export interface ThreadResponse {
  order: string[]
  posts: Record<string, Post>
  next_cursor?: string
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
