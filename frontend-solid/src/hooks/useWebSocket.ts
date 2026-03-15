// ============================================
// useWebSocket Hook for Solid.js
// ============================================

import { createEffect, onCleanup, createSignal } from 'solid-js';
import { authStore } from '../stores/auth';
import { channelStore } from '../stores/channels';
import { messageStore } from '../stores/messages';
import { presenceStore } from '../stores/presence';
import { unreadStore } from '../stores/unreads';
import {
  websocket,
  on,
  onAny,
  subscribe,
  unsubscribe,
  sendTyping as wsSendTyping,
  sendStopTyping as wsSendStopTyping,
  sendPresence as wsSendPresence,
} from '../realtime/websocket';
import { handleCallWebsocketEvent } from '../stores/calls';
import {
  playNewMessageSound,
  playMentionSound,
  playDirectMessageSound,
} from '../utils/sounds';
import type {
  ServerEvent,
  WsEnvelope,
  PostedEvent,
  MessageUpdateEvent,
  ReactionEvent,
  TypingEvent,
  PresenceEvent,
  ChannelEvent,
  UnreadCountsEvent,
  PostUnreadEvent,
  InitialLoadEvent,
} from '../realtime/events';
import type { ConnectionState } from '../realtime/events';
import type { Post } from '../api/types';

// ============================================
// Hook State
// ============================================

const [isReady, setIsReady] = createSignal(false);

// ============================================
// Event Handlers
// ============================================

function handlePosted(data: PostedEvent, envelope: WsEnvelope) {
  const postData = data.post;
  if (!postData) return;

  let post: Post;
  if (typeof postData === 'string') {
    try {
      post = JSON.parse(postData) as Post;
    } catch {
      return;
    }
  } else {
    post = postData as unknown as Post;
  }

  // Normalize channel_id from envelope if not in post
  if (!post.channel_id && envelope.channel_id) {
    post.channel_id = envelope.channel_id;
  }

  messageStore.handleNewMessage(post as unknown as import('../stores/messages').Post);

  // Handle notifications and sounds for messages from others
  if (post.user_id !== authStore.user()?.id) {
    const isCurrentChannel = post.channel_id === channelStore.currentChannelId();
    const currentUser = authStore.user();
    const isMention = currentUser && post.message?.includes(`@${currentUser.username}`);
    
    // Check if this is a DM by looking up the channel
    const channel = channelStore.getChannel(post.channel_id);
    const isDirectMessage = channel?.channel_type === 'direct';

    // Play sounds based on message type
    if (!isCurrentChannel) {
      // Update unread counts
      unreadStore.incrementChannelUnread(post.channel_id);

      if (isMention) {
        unreadStore.incrementChannelMention(post.channel_id);
        playMentionSound();
      } else if (isDirectMessage) {
        playDirectMessageSound();
      } else {
        playNewMessageSound();
      }
    } else if (isMention) {
      // Mention in current channel - still play sound
      playMentionSound();
    }
  }
}

function handleMessageUpdated(data: MessageUpdateEvent) {
  messageStore.handleMessageUpdate(data);
}

function handleMessageDeleted(data: { post_id?: string; id?: string }) {
  const messageId = data.post_id || data.id;
  if (messageId) {
    messageStore.handleMessageDelete(messageId);
  }
}

function handleReactionAdded(data: ReactionEvent) {
  let reaction: { post_id: string; user_id: string; emoji_name: string };

  if (data.reaction) {
    if (typeof data.reaction === 'string') {
      try {
        reaction = JSON.parse(data.reaction);
      } catch {
        return;
      }
    } else {
      reaction = data.reaction as { post_id: string; user_id: string; emoji_name: string };
    }
  } else {
    reaction = {
      post_id: data.post_id!,
      user_id: data.user_id!,
      emoji_name: data.emoji_name || data.emoji!,
    };
  }

  if (reaction.post_id && reaction.user_id && reaction.emoji_name) {
    messageStore.handleReactionAdded(reaction);
  }
}

function handleReactionRemoved(data: ReactionEvent) {
  let reaction: { post_id: string; user_id: string; emoji_name: string };

  if (data.reaction) {
    if (typeof data.reaction === 'string') {
      try {
        reaction = JSON.parse(data.reaction);
      } catch {
        return;
      }
    } else {
      reaction = data.reaction as { post_id: string; user_id: string; emoji_name: string };
    }
  } else {
    reaction = {
      post_id: data.post_id!,
      user_id: data.user_id!,
      emoji_name: data.emoji_name || data.emoji!,
    };
  }

  if (reaction.post_id && reaction.user_id && reaction.emoji_name) {
    messageStore.handleReactionRemoved(reaction);
  }
}

function handleTyping(data: TypingEvent, envelope: WsEnvelope) {
  const channelId = data.channel_id || envelope.channel_id || envelope.broadcast?.channel_id;
  if (!channelId) return;

  presenceStore.addTypingUser(
    data.user_id,
    data.display_name || data.username || 'Someone',
    channelId,
    data.thread_root_id || data.parent_id
  );
}

function handleStopTyping(data: TypingEvent, envelope: WsEnvelope) {
  const channelId = data.channel_id || envelope.channel_id || envelope.broadcast?.channel_id;
  if (!channelId) return;

  presenceStore.removeTypingUser(data.user_id, channelId, data.thread_root_id || data.parent_id);
}

function handlePresence(data: PresenceEvent) {
  const status = data.status.toLowerCase() as 'online' | 'away' | 'dnd' | 'offline';
  presenceStore.updatePresenceFromEvent(data.user_id, status);
}

function handleStatusChange(data: PresenceEvent) {
  const status = data.status.toLowerCase() as 'online' | 'away' | 'dnd' | 'offline';
  presenceStore.updatePresenceFromEvent(data.user_id, status);
}

function handleChannelCreated(data: ChannelEvent) {
  if (data.id && data.team_id) {
    channelStore.addChannel({
      id: data.id,
      team_id: data.team_id,
      name: data.name || '',
      display_name: data.display_name || data.name || '',
      channel_type: normalizeChannelType(data.channel_type || data.type),
      header: data.header || '',
      purpose: data.purpose || '',
      created_at: normalizeTimestamp(data.created_at ?? data.create_at),
      creator_id: data.creator_id || '',
    });
  }
}

function handleUnreadCountsUpdated(data: UnreadCountsEvent) {
  unreadStore.handleUnreadUpdate(data);
}

function handlePostUnread(data: PostUnreadEvent) {
  unreadStore.applyPostUnread(data);
}

function handleInitialLoad(data: InitialLoadEvent) {
  // Handle channels
  if (Array.isArray(data.channels)) {
    data.channels.forEach((rawChannel: unknown) => {
      const channel = rawChannel as ChannelEvent;
      if (channel.id && channel.team_id) {
        channelStore.addChannel({
          id: channel.id,
          team_id: channel.team_id,
          name: channel.name || '',
          display_name: channel.display_name || channel.name || '',
          channel_type: normalizeChannelType(channel.channel_type || channel.type),
          header: channel.header || '',
          purpose: channel.purpose || '',
          created_at: normalizeTimestamp(channel.created_at ?? channel.create_at),
          creator_id: channel.creator_id || '',
        });
      }
    });
  }

  // Handle unreads
  if (Array.isArray(data.channel_unreads)) {
    data.channel_unreads.forEach((rawUnread: unknown) => {
      const unread = rawUnread as { channel_id: string; msg_count?: number; mention_count?: number };
      if (unread.channel_id) {
        unreadStore.channelUnreads[unread.channel_id] = unread.msg_count ?? 0;
        unreadStore.channelMentions[unread.channel_id] = unread.mention_count ?? 0;
      }
    });
  }

  // Handle statuses
  if (Array.isArray(data.statuses)) {
    data.statuses.forEach((rawStatus: unknown) => {
      const status = rawStatus as { user_id: string; status: string };
      if (status.user_id && status.status) {
        const presenceStatus = status.status.toLowerCase() as 'online' | 'away' | 'dnd' | 'offline';
        presenceStore.updatePresenceFromEvent(status.user_id, presenceStatus);
      }
    });
  }
}

// ============================================
// Helpers
// ============================================

function normalizeChannelType(type: string | undefined): 'public' | 'private' | 'direct' | 'group' {
  const raw = String(type || '').toLowerCase();
  if (raw === 'o' || raw === 'public') return 'public';
  if (raw === 'p' || raw === 'private') return 'private';
  if (raw === 'd' || raw === 'direct') return 'direct';
  if (raw === 'g' || raw === 'group') return 'group';
  return 'public';
}

function normalizeTimestamp(value: string | number | undefined): string {
  if (typeof value === 'number') {
    return new Date(value).toISOString();
  }
  if (typeof value === 'string') {
    return value;
  }
  return new Date().toISOString();
}

// ============================================
// Hook
// ============================================

export interface UseWebSocketReturn {
  isConnected: () => boolean;
  isConnecting: () => boolean;
  isReconnecting: () => boolean;
  connectionState: () => ConnectionState;
  subscribe: (channelId: string) => void;
  unsubscribe: (channelId: string) => void;
  sendTyping: (channelId: string, threadRootId?: string) => boolean;
  sendStopTyping: (channelId: string, threadRootId?: string) => boolean;
  sendPresence: (status: string) => boolean;
  reconnect: () => void;
}

export function useWebSocket(): UseWebSocketReturn {
  // Auto-connect when authenticated
  createEffect(() => {
    if (authStore.isAuthenticated) {
      websocket.connect();
    } else {
      websocket.disconnect();
    }
  });

  // Register event handlers
  createEffect(() => {
    if (!authStore.isAuthenticated) return;

    const handlers: Array<{ event: ServerEvent; handler: (data: unknown, envelope: WsEnvelope) => void }> = [
      { event: 'posted', handler: handlePosted as (data: unknown, envelope: WsEnvelope) => void },
      { event: 'message_created', handler: handlePosted as (data: unknown, envelope: WsEnvelope) => void },
      { event: 'post_created', handler: handlePosted as (data: unknown, envelope: WsEnvelope) => void },
      { event: 'message_posted', handler: handlePosted as (data: unknown, envelope: WsEnvelope) => void },
      { event: 'thread_reply_created', handler: handlePosted as (data: unknown, envelope: WsEnvelope) => void },
      { event: 'message_updated', handler: handleMessageUpdated as (data: unknown) => void },
      { event: 'post_edited', handler: handleMessageUpdated as (data: unknown) => void },
      { event: 'thread_reply_updated', handler: handleMessageUpdated as (data: unknown) => void },
      { event: 'message_deleted', handler: handleMessageDeleted as (data: unknown) => void },
      { event: 'post_deleted', handler: handleMessageDeleted as (data: unknown) => void },
      { event: 'thread_reply_deleted', handler: handleMessageDeleted as (data: unknown) => void },
      { event: 'reaction_added', handler: handleReactionAdded as (data: unknown) => void },
      { event: 'reaction_removed', handler: handleReactionRemoved as (data: unknown) => void },
      { event: 'user_typing', handler: handleTyping as (data: unknown, envelope: WsEnvelope) => void },
      { event: 'typing', handler: handleTyping as (data: unknown, envelope: WsEnvelope) => void },
      { event: 'user_typing_stop', handler: handleStopTyping as (data: unknown, envelope: WsEnvelope) => void },
      { event: 'stop_typing', handler: handleStopTyping as (data: unknown, envelope: WsEnvelope) => void },
      { event: 'user_presence', handler: handlePresence as (data: unknown) => void },
      { event: 'status_change', handler: handleStatusChange as (data: unknown) => void },
      { event: 'channel_created', handler: handleChannelCreated as (data: unknown) => void },
      { event: 'unread_counts_updated', handler: handleUnreadCountsUpdated as (data: unknown) => void },
      { event: 'post_unread', handler: handlePostUnread as (data: unknown) => void },
      { event: 'initial_load', handler: handleInitialLoad as (data: unknown) => void },
    ];

    // Subscribe to all handlers
    const unsubscribers = handlers.map(({ event, handler }) => on(event, handler));
    const unsubscribeCalls = onAny((data, envelope) => {
      const eventName = envelope?.event;
      if (typeof eventName !== 'string' || !eventName.startsWith('custom_com.mattermost.calls_')) return;
      handleCallWebsocketEvent(eventName, data, envelope.channel_id);
    });

    setIsReady(true);

    onCleanup(() => {
      unsubscribers.forEach((unsub) => unsub());
      unsubscribeCalls();
      setIsReady(false);
    });
  });

  // Subscribe to current channel
  createEffect(() => {
    const channelId = channelStore.currentChannelId();
    if (!channelId || !isReady()) return;

    subscribe(channelId);

    onCleanup(() => {
      unsubscribe(channelId);
    });
  });

  return {
    isConnected: websocket.isConnected,
    isConnecting: websocket.isConnecting,
    isReconnecting: websocket.isReconnecting,
    connectionState: websocket.connectionState,
    subscribe,
    unsubscribe,
    sendTyping: wsSendTyping,
    sendStopTyping: wsSendStopTyping,
    sendPresence: wsSendPresence,
    reconnect: websocket.reconnect,
  };
}

export default useWebSocket;
