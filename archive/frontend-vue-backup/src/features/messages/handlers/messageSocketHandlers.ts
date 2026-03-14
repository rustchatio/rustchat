// Message WebSocket Handlers - Feature-specific WebSocket event handling
// Replaces the centralized useWebSocket.ts message handling logic

import { messageService } from '../services/messageService'
import type { Message, MessageId } from '../../../core/entities/Message'
import type { ChannelId } from '../../../core/entities/Channel'

interface WebSocketMessageEvent {
  event: 'posted' | 'post_edited' | 'post_deleted' | 'reaction_added' | 'reaction_removed'
  data: string // JSON stringified data
  broadcast: {
    channel_id: string
    user_id: string
  }
}

interface PostData {
  post: string // JSON stringified Message
}

export function handleWebSocketEvent(event: WebSocketMessageEvent) {
  switch (event.event) {
    case 'posted':
      handlePost(event)
      break
    case 'post_edited':
      handlePostEdit(event)
      break
    case 'post_deleted':
      handlePostDelete(event)
      break
    case 'reaction_added':
      handleReactionAdded(event)
      break
    case 'reaction_removed':
      handleReactionRemoved(event)
      break
  }
}

function handlePost(event: WebSocketMessageEvent) {
  try {
    const data: PostData = JSON.parse(event.data)
    const post: Message = JSON.parse(data.post)

    // Normalize the post
    const normalizedPost = normalizePost(post)
    messageService.handleIncomingMessage(normalizedPost)
  } catch (err) {
    console.error('Failed to handle post:', err)
  }
}

function handlePostEdit(event: WebSocketMessageEvent) {
  try {
    const data: PostData = JSON.parse(event.data)
    const post: Message = JSON.parse(data.post)

    const normalizedPost = normalizePost(post)
    messageService.handleMessageUpdate(normalizedPost.id, normalizedPost)
  } catch (err) {
    console.error('Failed to handle post edit:', err)
  }
}

function handlePostDelete(event: WebSocketMessageEvent) {
  try {
    const data = JSON.parse(event.data)
    const channelId = data.channel_id as ChannelId
    const messageId = data.post_id as MessageId

    messageService.handleMessageDelete(messageId, channelId)
  } catch (err) {
    console.error('Failed to handle post delete:', err)
  }
}

function handleReactionAdded(event: WebSocketMessageEvent) {
  try {
    const data = JSON.parse(event.data)
    const reaction = JSON.parse(data.reaction)

    messageService.handleReactionAdded(
      reaction.post_id as MessageId,
      reaction.emoji_name,
      reaction.user_id
    )
  } catch (err) {
    console.error('Failed to handle reaction added:', err)
  }
}

function handleReactionRemoved(event: WebSocketMessageEvent) {
  try {
    const data = JSON.parse(event.data)
    const reaction = JSON.parse(data.reaction)

    messageService.handleReactionRemoved(
      reaction.post_id as MessageId,
      reaction.emoji_name,
      reaction.user_id
    )
  } catch (err) {
    console.error('Failed to handle reaction removed:', err)
  }
}

// Normalize WebSocket post format to our Message entity
function normalizePost(post: any): Message {
  return {
    id: post.id,
    channelId: post.channel_id,
    userId: post.user_id,
    content: post.message,
    rootId: post.root_id,
    replyCount: post.reply_count ?? 0,
    reactions: normalizeReactions(post.reactions),
    files: normalizeFiles(post.metadata?.files || post.files || []),
    isPinned: post.is_pinned ?? false,
    isSaved: post.is_saved ?? false,
    status: 'delivered',
    clientId: post.props?.client_msg_id,
    createdAt: new Date(post.create_at),
    updatedAt: post.update_at ? new Date(post.update_at) : undefined,
    props: post.props
  }
}

function normalizeReactions(reactions: any): { emoji: string; count: number; users: string[] }[] {
  if (!reactions) return []
  
  if (Array.isArray(reactions)) {
    return reactions.map(r => ({
      emoji: r.emoji_name,
      count: r.count,
      users: r.users || []
    }))
  }

  // Handle object format
  return Object.entries(reactions).map(([emoji, data]: [string, any]) => ({
    emoji,
    count: data.count || 0,
    users: data.users || []
  }))
}

function normalizeFiles(files: any[]): any[] {
  return files.map(f => ({
    id: f.id,
    name: f.name,
    url: f.url,
    size: f.size,
    mimeType: f.mime_type || f.mimeType
  }))
}
