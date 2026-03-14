// ============================================
// RustChat Solid.js Stores
// ============================================

export {
  authStore,
  login,
  logout,
  fetchMe,
  updateStatus,
  getAuthPolicy,
  loginWithToken,
  refreshToken,
} from './auth';
export type {
  AuthState,
  LoginCredentials,
  User,
  LogoutReason,
  SessionWarning,
} from './auth';

export { userStore } from './user';
export type { UserPreferences, UserProfile } from './user';

export { channelStore } from './channels';
export type { Channel, ChannelType, ChannelMember, ChannelNotifyProps, CreateChannelRequest } from './channels';

export { messageStore, postToMessage } from './messages';
export type { Message, Post, FileAttachment, Reaction, ReadState, CreatePostRequest } from './messages';

export { presenceStore } from './presence';
export type { Presence, PresenceUser, TypingUser } from './presence';

export { unreadStore } from './unreads';
export type { ChannelUnread, TeamUnread, UnreadOverview, ChannelUnreadAt } from './unreads';

export { useTheme, ThemeProvider, AVAILABLE_THEMES } from './theme';
export type { Theme } from '../types';

export { uiStore } from './ui';
export type { UIPreferences, PanelTab } from './ui';

// ============================================
// Store Reset on Logout
// ============================================

import { messageStore } from './messages';
import { channelStore } from './channels';
import { unreadStore } from './unreads';
import { presenceStore } from './presence';

// Listen for logout events and reset stores
if (typeof window !== 'undefined') {
  window.addEventListener('rustchat:logout', () => {
    messageStore.resetSessionState?.();
    channelStore.clearChannels?.();
    unreadStore.clearAllState?.();
    presenceStore.clear?.();
  });
}
