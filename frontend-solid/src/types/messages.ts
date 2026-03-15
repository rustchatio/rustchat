// ============================================
// Message Types & Interfaces for RustChat
// ============================================

// ============================================
// Message (Frontend Model)
// ============================================

/**
 * File attachment interface for messages
 */
export interface FileAttachment {
  id: string;
  name: string;
  url: string;
  thumbnail_url?: string;
  size: number;
  mime_type: string;
  width?: number;
  height?: number;
  has_preview_image?: boolean;
  mini_preview?: string;
}

/**
 * Reaction interface for messages
 */
export interface Reaction {
  emoji: string;
  count: number;
  users: string[];
}

/**
 * Main Message interface used in the UI
 */
export interface Message {
  id: string;
  channelId: string;
  userId: string;
  username: string;
  avatarUrl?: string;
  email?: string;
  content: string;
  timestamp: string;
  editedAt?: string;
  reactions: Reaction[];
  threadCount?: number;
  lastReplyAt?: string;
  rootId?: string;
  files?: FileAttachment[];
  isPinned: boolean;
  isSaved: boolean;
  status?: 'sending' | 'delivered' | 'failed';
  clientMsgId?: string;
  props?: Record<string, unknown>;
  seq: number | string;
}

// ============================================
// Post (Mattermost-compatible API Model)
// ============================================

/**
 * Post metadata for embeds, files, reactions
 */
export interface PostMetadata {
  embeds?: Embed[];
  emojis?: Emoji[];
  files?: FileInfo[];
  images?: Record<string, PostImage>;
  reactions?: PostReaction[];
}

/**
 * Embed data for link previews
 */
export interface Embed {
  type: string;
  url?: string;
  data?: Record<string, unknown>;
}

/**
 * Emoji reference in post metadata
 */
export interface Emoji {
  name: string;
  unified?: string;
  custom?: boolean;
}

/**
 * File info for post attachments
 */
export interface FileInfo {
  id: string;
  user_id: string;
  post_id?: string;
  name: string;
  extension: string;
  url?: string;
  thumbnail_url?: string;
  size: number;
  mime_type: string;
  width?: number;
  height?: number;
  has_preview_image?: boolean;
  mini_preview?: string;
}

/**
 * Image dimensions in post metadata
 */
export interface PostImage {
  format: string;
  frame_count: number;
  width: number;
  height: number;
}

/**
 * Post reaction from API
 */
export interface PostReaction {
  user_id: string;
  post_id: string;
  emoji_name: string;
  create_at?: number;
  update_at?: number;
  delete_at?: number;
  // For display
  emoji?: string;
  count?: number;
  users?: string[];
}

/**
 * Post interface - Mattermost-compatible API response
 */
export interface Post {
  id: string;
  channel_id: string;
  user_id: string;
  message: string;
  root_post_id?: string;
  parent_id?: string;
  original_id?: string;
  created_at: string;
  updated_at: string;
  edited_at?: string | number | null;
  edit_at?: string | number | null;
  deleted_at?: string | number | null;
  is_pinned: boolean;
  props?: Record<string, unknown>;
  type?: string;
  has_reactions?: boolean;
  pending_post_id?: string;
  reply_count?: number;
  last_reply_at?: string | number | null;
  participants?: string[];
  is_following?: boolean;
  metadata?: PostMetadata;
  // Populated fields
  username?: string;
  avatar_url?: string;
  email?: string;
  files?: FileInfo[];
  reactions?: PostReaction[];
  is_saved?: boolean;
  client_msg_id?: string;
  seq: number | string;
}

// ============================================
// Thread
// ============================================

/**
 * Thread participant info
 */
export interface ThreadParticipant {
  id: string;
  username: string;
  avatar_url?: string;
}

/**
 * Thread info for collapsed thread view
 */
export interface ThreadInfo {
  id: string;
  channel_id: string;
  reply_count: number;
  last_reply_at: string;
  participants: ThreadParticipant[];
  is_following: boolean;
  unread_replies?: number;
  unread_mentions?: number;
}

/**
 * Full thread with posts
 */
export interface Thread {
  id: string;
  channel_id: string;
  posts: Post[];
  order: string[];
}

// ============================================
// API Request/Response Types
// ============================================

/**
 * Read state for a channel
 */
export interface ReadState {
  last_read_message_id: number | null;
  first_unread_message_id: number | null;
}

/**
 * Create post request body
 */
export interface CreatePostRequest {
  channel_id: string;
  message: string;
  root_post_id?: string;
  parent_id?: string;
  file_ids?: string[];
  client_msg_id?: string;
  props?: Record<string, unknown>;
}

/**
 * Update post request body
 */
export interface UpdatePostRequest {
  message?: string;
  is_pinned?: boolean;
  has_reactions?: boolean;
  props?: Record<string, unknown>;
}

/**
 * Post list response from API
 */
export interface PostListResponse {
  messages: Post[];
  read_state: ReadState | null;
}

// ============================================
// Display/Format Types
// ============================================

/**
 * Message grouping for display
 */
export interface MessageGroup {
  date: Date;
  dateLabel: string;
  messages: Message[];
  hasUnreadSeparator?: boolean;
}

/**
 * File upload progress
 */
export interface FileUploadProgress {
  fileId: string;
  fileName: string;
  progress: number;
  status: 'pending' | 'uploading' | 'completed' | 'error';
  error?: string;
}

// ============================================
// Event Types
// ============================================

/**
 * Message action types
 */
export type MessageAction = 
  | 'react'
  | 'reply'
  | 'edit'
  | 'delete'
  | 'pin'
  | 'unpin'
  | 'save'
  | 'unsave'
  | 'copy_link'
  | 'mark_unread'
  | 'follow_thread'
  | 'unfollow_thread'
  | 'copy_text'
  | 'report';

/**
 * Message filter types
 */
export type MessageFilter = 'all' | 'unread' | 'pinned' | 'saved';
