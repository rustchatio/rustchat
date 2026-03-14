// ============================================
// WebSocket Event Type Definitions
// ============================================

// ============================================
// Envelope Types
// ============================================

export type EnvelopeType = 'event' | 'response' | 'error' | 'ack' | 'command';

export interface WsEnvelope {
  type: EnvelopeType;
  event: string;
  seq?: number;
  channel_id?: string;
  broadcast?: {
    channel_id?: string;
    team_id?: string;
    user_id?: string;
    omit_users?: Record<string, boolean>;
  };
  data: unknown;
}

export interface ClientEnvelope {
  type: 'command';
  event: string;
  data: Record<string, unknown>;
  channel_id?: string;
  client_msg_id?: string;
  seq?: number;
}

// ============================================
// Event Types
// ============================================

export type ServerEvent =
  | 'hello'
  | 'initial_load'
  | 'posted'
  | 'message_created'
  | 'post_created'
  | 'message_posted'
  | 'thread_reply_created'
  | 'message_updated'
  | 'post_edited'
  | 'thread_reply_updated'
  | 'message_deleted'
  | 'post_deleted'
  | 'thread_reply_deleted'
  | 'reaction_added'
  | 'reaction_removed'
  | 'user_typing'
  | 'typing'
  | 'user_typing_stop'
  | 'stop_typing'
  | 'user_presence'
  | 'status_change'
  | 'channel_created'
  | 'channel_updated'
  | 'channel_deleted'
  | 'user_added'
  | 'user_removed'
  | 'unread_counts_updated'
  | 'post_unread'
  | 'error';

export type ClientEvent =
  | 'subscribe_channel'
  | 'unsubscribe_channel'
  | 'user_typing'
  | 'user_typing_stop'
  | 'presence'
  | 'reconnect';

// ============================================
// Event Data Types
// ============================================

export interface HelloEvent {
  server_version: string;
  connection_id: string;
}

export interface InitialLoadEvent {
  channels: unknown[];
  channel_unreads: Array<{
    channel_id: string;
    msg_count: number;
    mention_count: number;
  }>;
  statuses: Array<{
    user_id: string;
    status: string;
  }>;
  teams?: unknown[];
  preferences?: unknown[];
}

export interface PostedEvent {
  post?: string | Record<string, unknown>;
  channel_id?: string;
  sender_name?: string;
  team_id?: string;
  mentions?: string[];
}

export interface MessageUpdateEvent {
  id: string;
  channel_id?: string;
  message?: string;
  is_pinned?: boolean;
  reply_count?: number;
  reply_count_inc?: number;
  edited_at?: string | number;
  edit_at?: string | number;
  last_reply_at?: string | number;
}

export interface ReactionEvent {
  reaction?: string | Record<string, unknown>;
  post_id?: string;
  user_id?: string;
  emoji_name?: string;
  emoji?: string;
}

export interface TypingEvent {
  user_id: string;
  username?: string;
  display_name?: string;
  channel_id?: string;
  thread_root_id?: string;
  parent_id?: string;
}

export interface PresenceEvent {
  user_id: string;
  status: 'online' | 'away' | 'dnd' | 'offline';
  manual?: boolean;
  last_activity_at?: number;
}

export interface ChannelEvent {
  id: string;
  team_id: string;
  name?: string;
  display_name?: string;
  channel_type?: string;
  type?: string;
  header?: string;
  purpose?: string;
  created_at?: string | number;
  create_at?: string | number;
  creator_id?: string;
}

export interface UnreadCountsEvent {
  channel_id: string;
  team_id: string;
  unread_count: number;
  mention_count: number;
}

export interface PostUnreadEvent {
  team_id: string;
  user_id: string;
  channel_id: string;
  msg_count: number;
  mention_count: number;
  mention_count_root: number;
  urgent_mention_count: number;
  msg_count_root: number;
  last_viewed_at: number;
}

// ============================================
// Event Handler Type
// ============================================

export type EventHandler<T = unknown> = (data: T, envelope: WsEnvelope) => void;

// ============================================
// Connection Status
// ============================================

export type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'reconnecting' | 'error';

export interface ConnectionState {
  status: ConnectionStatus;
  reconnectAttempts: number;
  lastError?: string;
  connectedAt?: Date;
}
