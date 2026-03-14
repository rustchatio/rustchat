// ============================================
// Desktop Notifications Hook
// ============================================

import { createSignal, createEffect, onCleanup } from 'solid-js';
import { authStore } from '../stores/auth';

// ============================================
// Types
// ============================================

export interface NotificationPermissionState {
  permission: NotificationPermission;
  supported: boolean;
  enabled: boolean;
}

export interface NotificationOptions {
  title: string;
  body?: string;
  icon?: string;
  badge?: string;
  tag?: string;
  requireInteraction?: boolean;
  silent?: boolean;
  data?: Record<string, unknown>;
  onClick?: () => void;
}

// ============================================
// State
// ============================================

const [permissionState, setPermissionState] = createSignal<NotificationPermissionState>({
  permission: 'default',
  supported: false,
  enabled: false,
});

// Track active notifications for cleanup
const activeNotifications = new Map<string, Notification>();

// ============================================
// Helpers
// ============================================

function isSupported(): boolean {
  return typeof window !== 'undefined' && 'Notification' in window;
}

function updatePermissionState() {
  if (!isSupported()) {
    setPermissionState({ permission: 'denied', supported: false, enabled: false });
    return;
  }

  const permission = Notification.permission;
  const enabled = permission === 'granted' && localStorage.getItem('notifications_enabled') !== 'false';

  setPermissionState({
    permission,
    supported: true,
    enabled,
  });
}

// ============================================
// Actions
// ============================================

export async function requestPermission(): Promise<NotificationPermission> {
  if (!isSupported()) {
    return 'denied';
  }

  try {
    const permission = await Notification.requestPermission();
    updatePermissionState();

    if (permission === 'granted') {
      localStorage.setItem('notifications_enabled', 'true');
    }

    return permission;
  } catch (err) {
    console.error('[Notifications] Failed to request permission:', err);
    return 'denied';
  }
}

export function enableNotifications(): boolean {
  if (!isSupported() || Notification.permission !== 'granted') {
    return false;
  }

  localStorage.setItem('notifications_enabled', 'true');
  updatePermissionState();
  return true;
}

export function disableNotifications(): void {
  localStorage.setItem('notifications_enabled', 'false');
  updatePermissionState();
}

export function showNotification(options: NotificationOptions): Notification | null {
  const state = permissionState();

  if (!state.supported || !state.enabled || state.permission !== 'granted') {
    return null;
  }

  // Don't show notifications when user is active on the page
  if (document.visibilityState === 'visible' && document.hasFocus()) {
    // Still create notification but don't require interaction
    options = { ...options, requireInteraction: false };
  }

  try {
    const notification = new Notification(options.title, {
      body: options.body,
      icon: options.icon || '/icon-192x192.png',
      badge: options.badge || '/icon-72x72.png',
      tag: options.tag,
      requireInteraction: options.requireInteraction ?? false,
      silent: options.silent ?? false,
      data: options.data,
    });

    // Handle click
    notification.onclick = () => {
      window.focus();
      options.onClick?.();
      notification.close();
    };

    // Track notification
    if (options.tag) {
      // Close existing notification with same tag
      activeNotifications.get(options.tag)?.close();
      activeNotifications.set(options.tag, notification);
    }

    // Auto-cleanup after 10 seconds if not requiring interaction
    if (!options.requireInteraction) {
      setTimeout(() => {
        notification.close();
        if (options.tag) {
          activeNotifications.delete(options.tag);
        }
      }, 10000);
    }

    return notification;
  } catch (err) {
    console.error('[Notifications] Failed to show notification:', err);
    return null;
  }
}

export function closeNotification(tag: string): void {
  activeNotifications.get(tag)?.close();
  activeNotifications.delete(tag);
}

export function closeAllNotifications(): void {
  activeNotifications.forEach((notification) => notification.close());
  activeNotifications.clear();
}

// ============================================
// Message Notification Helper
// ============================================

export function notifyNewMessage(params: {
  channelName: string;
  senderName: string;
  message: string;
  channelId: string;
  onClick?: () => void;
}): Notification | null {
  const currentUser = authStore.user();
  const isMention = params.message.includes(`@${currentUser?.username}`);

  // Check if we should notify based on preferences
  const shouldNotify = localStorage.getItem('notify_messages') !== 'false';
  const shouldMention = isMention && localStorage.getItem('notify_mentions') !== 'false';

  if (!shouldNotify && !shouldMention) {
    return null;
  }

  return showNotification({
    title: isMention ? `Mention in ${params.channelName}` : params.channelName,
    body: `${params.senderName}: ${params.message.slice(0, 100)}${params.message.length > 100 ? '...' : ''}`,
    tag: `message-${params.channelId}`,
    requireInteraction: isMention,
    onClick: params.onClick,
  });
}

// ============================================
// Hook
// ============================================

export function useDesktopNotifications() {
  // Initialize permission state
  createEffect(() => {
    updatePermissionState();
  });

  // Handle permission changes
  createEffect(() => {
    if (!isSupported()) return;

    // Listen for visibility changes to manage notification state
    const handleVisibilityChange = () => {
      if (document.visibilityState === 'visible') {
        // Close non-mention notifications when user returns
        activeNotifications.forEach((notification, tag) => {
          if (!tag.includes('mention')) {
            notification.close();
            activeNotifications.delete(tag);
          }
        });
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);

    onCleanup(() => {
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    });
  });

  return {
    permissionState,
    requestPermission,
    enableNotifications,
    disableNotifications,
    showNotification,
    closeNotification,
    closeAllNotifications,
    notifyNewMessage,
  };
}

export default useDesktopNotifications;
