/**
 * Activity Feed Types
 */

export enum ActivityType {
  MENTION = 'mention',
  REPLY = 'reply',
  REACTION = 'reaction',
  DM = 'dm',
  THREAD_REPLY = 'thread_reply'
}

export interface Activity {
  id: string
  type: ActivityType
  actorId: string
  actorUsername: string
  actorAvatarUrl?: string
  channelId: string
  channelName: string
  teamId: string
  teamName: string
  postId: string
  rootId?: string
  message?: string
  reaction?: string
  read: boolean
  createdAt: Date
}

export interface ActivityFeedResponse {
  order: string[]
  activities: Record<string, Activity>
  unreadCount: number
  nextCursor?: string
}

export interface ActivityQueryParams {
  cursor?: string
  limit?: number
  type?: ActivityType | string
  unreadOnly?: boolean
}
