// ============================================
// API Type Definitions for RustChat
// ============================================

// ============================================
// Common Types
// ============================================

export interface ApiError {
  id: string;
  message: string;
  detailed_error?: string;
  request_id?: string;
  status_code: number;
}

export interface ApiResponse<T> {
  data: T;
  status: number;
  headers: Record<string, string>;
}

export interface PaginatedResponse<T> {
  data: T[];
  total_count: number;
  page: number;
  per_page: number;
  has_next: boolean;
}

export interface PaginationParams {
  page?: number;
  per_page?: number;
  q?: string;
}

// ============================================
// Auth Types
// ============================================

export interface LoginRequest {
  email: string;
  password: string;
  remember?: boolean;
}

export interface LoginResponse {
  token: string;
  user: User;
}

export interface AuthPolicy {
  enable_email_password: boolean;
  enable_sign_in_with_email?: boolean;
  enable_sign_in_with_username?: boolean;
  enable_sso: boolean;
  require_sso: boolean;
  allow_registration: boolean;
  password_min_length: number;
  password_require_uppercase: boolean;
  password_require_number: boolean;
  password_require_symbol: boolean;
  session_length_hours: number;
}

// ============================================
// User Types
// ============================================

export interface User {
  id: string;
  username: string;
  email: string;
  first_name?: string;
  last_name?: string;
  display_name?: string;
  nickname?: string;
  position?: string;
  avatar_url?: string;
  role: string;
  presence: 'online' | 'away' | 'dnd' | 'offline';
  custom_status?: {
    text?: string;
    emoji?: string;
    expires_at?: string;
  };
  notify_props?: Record<string, string>;
  props?: Record<string, unknown>;
  timezone?: {
    automaticTimezone: string;
    manualTimezone: string;
    useAutomaticTimezone: string;
  };
  create_at: number;
  update_at: number;
  delete_at: number;
  last_password_update?: number;
  last_picture_update?: number;
  locale: string;
}

export interface UpdateUserRequest {
  username?: string;
  first_name?: string;
  last_name?: string;
  display_name?: string;
  nickname?: string;
  position?: string;
  avatar_url?: string;
  email?: string;
  props?: Record<string, unknown>;
  notify_props?: Record<string, string>;
}

export interface ChangePasswordRequest {
  current_password?: string;
  new_password: string;
}

export interface UserStatus {
  status?: 'online' | 'away' | 'dnd' | 'offline';
  presence?: 'online' | 'away' | 'dnd' | 'offline';
  text?: string;
  emoji?: string;
  expires_at?: string;
}

export interface UpdateStatusRequest {
  status?: 'online' | 'away' | 'dnd' | 'offline';
  presence?: 'online' | 'away' | 'dnd' | 'offline';
  text?: string;
  emoji?: string;
  duration?: string;
  duration_minutes?: number;
  dnd_end_time?: number;
}

export interface UserStatusResponse {
  user_id: string;
  status: string;
  manual: boolean;
  last_activity_at: number;
}

// ============================================
// Channel Types
// ============================================

export type ChannelType = 'public' | 'private' | 'direct' | 'group';

export interface Channel {
  id: string;
  team_id: string;
  name: string;
  display_name: string;
  channel_type: ChannelType;
  header?: string;
  purpose?: string;
  unreadCount?: number;
  mentionCount?: number;
  created_at: string;
  creator_id: string;
}

export interface ChannelMember {
  channel_id: string;
  user_id: string;
  roles: string;
  last_viewed_at: number;
  msg_count: number;
  mention_count: number;
  mention_count_root: number;
  urgent_mention_count: number;
  msg_count_root: number;
  notify_props: ChannelNotifyProps;
  last_update_at: number;
}

export interface ChannelNotifyProps {
  desktop?: 'default' | 'all' | 'mention' | 'none';
  mobile?: 'default' | 'all' | 'mention' | 'none';
  mark_unread?: 'all' | 'mention';
  ignore_channel_mentions?: 'default' | 'off' | 'on';
}

export interface CreateChannelRequest {
  team_id: string;
  name: string;
  display_name: string;
  channel_type: ChannelType;
  header?: string;
  purpose?: string;
  target_user_id?: string;
}

export interface UpdateChannelRequest {
  name?: string;
  display_name?: string;
  header?: string;
  purpose?: string;
}

export interface SidebarCategory {
  id: string;
  team_id: string;
  user_id: string;
  type: string;
  display_name: string;
  sorting: string;
  muted: boolean;
  collapsed: boolean;
  channel_ids: string[];
  sort_order: number;
  create_at: number;
  update_at: number;
  delete_at: number;
}

export interface SidebarCategories {
  categories: SidebarCategory[];
  order: string[];
}

// ============================================
// Message/Post Types
// ============================================

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
  reactions?: Reaction[];
  is_saved?: boolean;
  client_msg_id?: string;
  seq: number | string;
}

export interface PostMetadata {
  embeds?: Embed[];
  emojis?: Emoji[];
  files?: FileInfo[];
  images?: Record<string, PostImage>;
  reactions?: Reaction[];
}

export interface Embed {
  type: string;
  url?: string;
  data?: Record<string, unknown>;
}

export interface Emoji {
  name: string;
  unified?: string;
  custom?: boolean;
}

export interface FileInfo {
  id: string;
  user_id: string;
  post_id?: string;
  name: string;
  extension: string;
  size: number;
  mime_type: string;
  width?: number;
  height?: number;
  has_preview_image?: boolean;
  mini_preview?: string;
}

export interface PostImage {
  format: string;
  frame_count: number;
  width: number;
  height: number;
}

export interface Reaction {
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

export interface ReadState {
  last_read_message_id: number | null;
  first_unread_message_id: number | null;
}

export interface PostListResponse {
  messages: Post[];
  read_state: ReadState | null;
}

export interface CreatePostRequest {
  channel_id: string;
  message: string;
  root_post_id?: string;
  parent_id?: string;
  file_ids?: string[];
  client_msg_id?: string;
  props?: Record<string, unknown>;
}

export interface UpdatePostRequest {
  message?: string;
  is_pinned?: boolean;
  has_reactions?: boolean;
  props?: Record<string, unknown>;
}

export interface ChannelUnreadAt {
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
// Unread Types
// ============================================

export interface ChannelUnread {
  channel_id: string;
  team_id: string;
  mention_count: number;
  mention_count_root: number;
  msg_count: number;
  msg_count_root: number;
  urgent_mention_count: number;
  last_viewed_at: number;
  notify_props?: ChannelNotifyProps;
}

export interface TeamUnread {
  team_id: string;
  mention_count: number;
  msg_count: number;
}

export interface UnreadOverview {
  channels: ChannelUnread[];
  teams: TeamUnread[];
}

// ============================================
// WebSocket Types
// ============================================

export interface WsEnvelope {
  type: 'event' | 'response' | 'error' | 'ack';
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
// Team Types
// ============================================

export interface Team {
  id: string;
  name: string;
  display_name: string;
  description?: string;
  company_name?: string;
  allowed_domains?: string;
  invite_id?: string;
  allow_open_invite: boolean;
  delete_at: number;
  create_at: number;
  update_at: number;
  scheme_id?: string;
  policy_id?: string;
}

export interface TeamMember {
  team_id: string;
  user_id: string;
  roles: string;
  delete_at: number;
  scheme_user?: boolean;
  scheme_admin?: boolean;
}

// ============================================
// Search Types
// ============================================

export interface SearchParams {
  terms: string;
  is_or_search?: boolean;
  time_zone_offset?: number;
  page?: number;
  per_page?: number;
  include_deleted_channels?: boolean;
}

export interface SearchResult {
  posts: Post[];
  order: string[];
  matches?: Record<string, string[]>;
}
