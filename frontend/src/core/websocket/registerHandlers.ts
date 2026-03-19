// WebSocket Handler Registration
// Central registration of all feature-specific WebSocket handlers

import { wsManager, type WebSocketEvent } from './WebSocketManager'
import { handleMessageWebSocketEvent, registerThreadHandlers } from '../../features/messages'
import { handleCallWebSocketEvent } from '../../features/calls'
import { handleChannelWebSocketEvent } from '../../features/channels'
import { handleActivityCreated, handleActivityRead } from '../../features/activity/handlers/activitySocketHandlers'
import type { Post } from '../../api/posts'

/**
 * Register all WebSocket event handlers
 * Call this once during app initialization
 */
export function registerWebSocketHandlers(): void {
  // Initialize thread handlers
  const threadHandlers = registerThreadHandlers()

  // Message events
  wsManager.on('posted', (event: WebSocketEvent) => {
    handleMessageWebSocketEvent(event as any)
    // Also handle for thread updates
    const data = JSON.parse((event as any).data)
    const post: Post = JSON.parse(data.post)
    threadHandlers.handleNewPost(post)
  })
  wsManager.on('post_edited', (event: WebSocketEvent) => {
    handleMessageWebSocketEvent(event as any)
    // Also handle for thread updates
    const data = JSON.parse((event as any).data)
    const post: Post = JSON.parse(data.post)
    threadHandlers.handlePostUpdated(post)
  })
  wsManager.on('post_deleted', (event: WebSocketEvent) => {
    handleMessageWebSocketEvent(event as any)
    // Also handle for thread updates
    const data = JSON.parse((event as any).data)
    threadHandlers.handlePostDeleted(data.post_id)
  })
  wsManager.on('reaction_added', (event: WebSocketEvent) => handleMessageWebSocketEvent(event as any))
  wsManager.on('reaction_removed', (event: WebSocketEvent) => handleMessageWebSocketEvent(event as any))

  // Call events
  wsManager.on('custom_com.mattermost.calls_call_start', (event: WebSocketEvent) => handleCallWebSocketEvent(event as any))
  wsManager.on('custom_com.mattermost.calls_call_end', (event: WebSocketEvent) => handleCallWebSocketEvent(event as any))
  wsManager.on('custom_com.mattermost.calls_user_joined', (event: WebSocketEvent) => handleCallWebSocketEvent(event as any))
  wsManager.on('custom_com.mattermost.calls_user_left', (event: WebSocketEvent) => handleCallWebSocketEvent(event as any))
  wsManager.on('custom_com.mattermost.calls_user_muted', (event: WebSocketEvent) => handleCallWebSocketEvent(event as any))
  wsManager.on('custom_com.mattermost.calls_user_unmuted', (event: WebSocketEvent) => handleCallWebSocketEvent(event as any))
  wsManager.on('custom_com.mattermost.calls_raise_hand', (event: WebSocketEvent) => handleCallWebSocketEvent(event as any))
  wsManager.on('custom_com.mattermost.calls_lower_hand', (event: WebSocketEvent) => handleCallWebSocketEvent(event as any))
  wsManager.on('custom_com.mattermost.calls_user_voice_on', (event: WebSocketEvent) => handleCallWebSocketEvent(event as any))
  wsManager.on('custom_com.mattermost.calls_user_voice_off', (event: WebSocketEvent) => handleCallWebSocketEvent(event as any))
  wsManager.on('custom_com.mattermost.calls_host_mute', (event: WebSocketEvent) => handleCallWebSocketEvent(event as any))
  wsManager.on('custom_com.mattermost.calls_host_removed', (event: WebSocketEvent) => handleCallWebSocketEvent(event as any))
  wsManager.on('custom_com.mattermost.calls_host_changed', (event: WebSocketEvent) => handleCallWebSocketEvent(event as any))
  wsManager.on('custom_com.mattermost.calls_ringing', (event: WebSocketEvent) => handleCallWebSocketEvent(event as any))
  wsManager.on('custom_com.mattermost.calls_screen_on', (event: WebSocketEvent) => handleCallWebSocketEvent(event as any))
  wsManager.on('custom_com.mattermost.calls_screen_off', (event: WebSocketEvent) => handleCallWebSocketEvent(event as any))
  wsManager.on('custom_com.mattermost.calls_signal', (event: WebSocketEvent) => handleCallWebSocketEvent(event as any))

  // Channel events
  wsManager.on('channel_created', (event: WebSocketEvent) => handleChannelWebSocketEvent(event as any))
  wsManager.on('channel_updated', (event: WebSocketEvent) => handleChannelWebSocketEvent(event as any))
  wsManager.on('channel_deleted', (event: WebSocketEvent) => handleChannelWebSocketEvent(event as any))
  wsManager.on('user_added', (event: WebSocketEvent) => handleChannelWebSocketEvent(event as any))
  wsManager.on('user_removed', (event: WebSocketEvent) => handleChannelWebSocketEvent(event as any))
  wsManager.on('channel_viewed', (event: WebSocketEvent) => handleChannelWebSocketEvent(event as any))

  // Activity feed events
  wsManager.on('activity_created', (event: WebSocketEvent) => {
    const data = JSON.parse((event as any).data)
    handleActivityCreated(data)
  })
  wsManager.on('activity_read', (event: WebSocketEvent) => {
    const data = JSON.parse((event as any).data)
    handleActivityRead(data)
  })

  console.log('[WebSocket] All handlers registered')
}

/**
 * Unregister all handlers (useful for testing or hot reload)
 */
export function unregisterWebSocketHandlers(): void {
  // Note: Current implementation doesn't support unregistering
  // Would need to store unsubscribe functions from wsManager.on()
  console.log('[WebSocket] Handler unregistration not implemented')
}
