import type { UserId } from './User'

export type ChannelId = string
export type TeamId = string

export type ChannelType = 'public' | 'private' | 'direct' | 'group'

export interface Channel {
  id: ChannelId
  teamId?: TeamId
  name: string
  displayName: string
  type: ChannelType
  purpose?: string
  header?: string
  creatorId: UserId
  createdAt: Date
  updatedAt: Date
  
  // Membership
  memberCount?: number
  isArchived: boolean
  
  // For DM/Group channels
  participantIds?: UserId[]
  
  // Unread state (client-side only)
  unreadCount?: number
  mentionCount?: number
}

export interface ChannelMember {
  channelId: ChannelId
  userId: UserId
  roles: string[]
  joinedAt: Date
  lastViewedAt?: Date
  notifyProps: {
    desktop: 'default' | 'all' | 'mention' | 'none'
    mobile: 'default' | 'all' | 'mention' | 'none'
    markUnread: 'all' | 'mention'
  }
}
