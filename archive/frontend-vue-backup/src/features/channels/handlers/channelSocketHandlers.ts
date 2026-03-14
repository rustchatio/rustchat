// Channel WebSocket Handlers - Feature-specific channel event handling

import { channelService } from '../services/channelService'
import type { ChannelId } from '../../../core/entities/Channel'
import type { UserId } from '../../../core/entities/User'

interface WebSocketChannelEvent {
  event: string
  data: string
  broadcast: {
    channel_id: string
    user_id: string
  }
}

export function handleChannelWebSocketEvent(event: WebSocketChannelEvent) {
  switch (event.event) {
    case 'channel_created':
      handleChannelCreated(event)
      break
    case 'channel_updated':
      handleChannelUpdated(event)
      break
    case 'channel_deleted':
      handleChannelDeleted(event)
      break
    case 'user_added':
      handleUserAdded(event)
      break
    case 'user_removed':
      handleUserRemoved(event)
      break
    case 'channel_viewed':
      handleChannelViewed(event)
      break
    case 'channel_updated':
      handleChannelUpdated(event)
      break
  }
}

// Helper to read event data safely
function readEventData(event: WebSocketChannelEvent): any {
  try {
    return JSON.parse(event.data)
  } catch {
    return {}
  }
}

function readEventChannelId(data: any): ChannelId | undefined {
  return (data?.channel_id || data?.channel_id_raw) as ChannelId | undefined
}

function readEventUserId(data: any): UserId | undefined {
  return (data?.user_id || data?.user_id_raw) as UserId | undefined
}

// Event handlers
function handleChannelCreated(event: WebSocketChannelEvent) {
  console.log('Channel created:', event)
  const data = readEventData(event)
  
  if (!data.channel_id) return

  const channel = normalizeChannel(data)
  channelService.handleChannelCreated(channel)
}

function handleChannelUpdated(event: WebSocketChannelEvent) {
  console.log('Channel updated:', event)
  const data = readEventData(event)
  
  if (!data.channel_id) return

  const channel = normalizeChannel(data)
  channelService.handleChannelUpdated(channel)
}

function handleChannelDeleted(event: WebSocketChannelEvent) {
  console.log('Channel deleted:', event)
  const data = readEventData(event)
  const channelId = readEventChannelId(data)
  
  if (channelId) {
    channelService.handleChannelDeleted(channelId)
  }
}

function handleUserAdded(event: WebSocketChannelEvent) {
  const data = readEventData(event)
  const channelId = readEventChannelId(data)
  const userId = readEventUserId(data)
  
  if (channelId && userId) {
    channelService.handleUserJoined(channelId, userId)
    // Refresh channel to get updated member count
    void channelService.loadChannels(data.team_id)
  }
}

function handleUserRemoved(event: WebSocketChannelEvent) {
  const data = readEventData(event)
  const channelId = readEventChannelId(data)
  const userId = readEventUserId(data)
  
  if (channelId && userId) {
    channelService.handleUserLeft(channelId, userId)
    // Refresh channel to get updated member count
    void channelService.loadChannels(data.team_id)
  }
}

function handleChannelViewed(event: WebSocketChannelEvent) {
  const data = readEventData(event)
  const channelId = readEventChannelId(data)
  
  if (channelId) {
    // Clear unread counts for this channel
    // This happens when the user views the channel on another device
    channelService.handleNewMessage(channelId, false)
  }
}

// Normalize WebSocket channel data to domain entity
function normalizeChannel(data: any): any {
  return {
    id: data.channel_id || data.id,
    teamId: data.team_id,
    name: data.name || data.channel_name,
    displayName: data.display_name || data.channel_display_name,
    type: data.channel_type || data.type,
    purpose: data.purpose,
    header: data.header,
    creatorId: data.creator_id,
    createdAt: data.create_at ? new Date(data.create_at) : new Date(),
    updatedAt: data.update_at ? new Date(data.update_at) : new Date(),
    isArchived: data.delete_at ? true : false,
    memberCount: data.member_count
  }
}
