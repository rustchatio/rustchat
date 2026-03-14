import { createStore, produce } from 'solid-js/store';
import { createSignal, createEffect } from 'solid-js';
import { authStore } from './auth';

// ============================================
// Types
// ============================================

export interface UserPreferences {
  theme: string;
  sidebar_collapsed: boolean;
  message_display: 'clean' | 'compact' | 'standard';
  clock_display: '12hour' | '24hour';
  teammate_name_display: 'username' | 'full_name' | 'nickname';
  enable_unread_scrollbar: boolean;
  channel_switcher_mode: 'recent' | 'alpha' | 'manual';
  language: string;
  timezone: string;
  // Notification preferences
  notify_dm: boolean;
  notify_mention: boolean;
  notify_channel_mention: boolean;
  notify_thread: boolean;
  email_digest: boolean;
  // Display preferences
  name_format: 'username' | 'full_name' | 'nickname';
  reduced_motion: boolean;
  larger_text: boolean;
  show_badges: boolean;
}

export interface UserProfile {
  id: string;
  username: string;
  email: string;
  first_name?: string;
  last_name?: string;
  display_name?: string;
  nickname?: string;
  position?: string;
  avatar_url?: string;
  notify_props?: Record<string, string>;
  props?: Record<string, unknown>;
}

// ============================================
// Store State
// ============================================

const STORAGE_KEY = 'rustchat_user_prefs';

const defaultPreferences: UserPreferences = {
  theme: 'light',
  sidebar_collapsed: false,
  message_display: 'clean',
  clock_display: '24hour',
  teammate_name_display: 'username',
  enable_unread_scrollbar: true,
  channel_switcher_mode: 'recent',
  language: 'en',
  timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
  // Notification defaults
  notify_dm: true,
  notify_mention: true,
  notify_channel_mention: false,
  notify_thread: true,
  email_digest: false,
  // Display defaults
  name_format: 'username',
  reduced_motion: false,
  larger_text: false,
  show_badges: true,
};

const getStoredPreferences = (): Partial<UserPreferences> => {
  if (typeof window === 'undefined') return {};
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored ? JSON.parse(stored) : {};
  } catch {
    return {};
  }
};

const [preferences, setPreferences] = createStore<UserPreferences>({
  ...defaultPreferences,
  ...getStoredPreferences(),
});

const [profile, setProfile] = createSignal<UserProfile | null>(null);
const [isLoading, setIsLoading] = createSignal(false);
const [error, setError] = createSignal<string | null>(null);

// ============================================
// Persistence Effects
// ============================================

createEffect(() => {
  if (typeof window !== 'undefined') {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(preferences));
  }
});

// ============================================
// Actions
// ============================================

export function updatePreferences(updates: Partial<UserPreferences>) {
  setPreferences(produce((draft) => {
    Object.assign(draft, updates);
  }));
}

export function resetPreferences() {
  setPreferences(defaultPreferences);
}

export async function fetchUserProfile(userId?: string): Promise<void> {
  const targetId = userId || authStore.user()?.id;
  if (!targetId) return;

  setIsLoading(true);
  setError(null);

  try {
    const token = authStore.token;
    const response = await fetch(`/api/v1/users/${targetId}`, {
      headers: token ? { Authorization: `Bearer ${token}` } : {},
    });

    if (!response.ok) throw new Error('Failed to fetch profile');

    const data: UserProfile = await response.json();
    setProfile(data);
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to fetch profile');
    throw err;
  } finally {
    setIsLoading(false);
  }
}

export async function updateProfile(updates: Partial<UserProfile>): Promise<void> {
  const userId = authStore.user()?.id;
  if (!userId) throw new Error('Not authenticated');

  setIsLoading(true);
  setError(null);

  try {
    const token = authStore.token;
    const response = await fetch(`/api/v1/users/${userId}`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify(updates),
    });

    if (!response.ok) {
      const payload = (await response.json().catch(() => null)) as
        | { message?: string; detailed_error?: string; error?: string }
        | null;
      throw new Error(
        payload?.message ||
        payload?.detailed_error ||
        payload?.error ||
        'Failed to update profile'
      );
    }

    const data: UserProfile = await response.json();
    setProfile(data);

    // Also update auth store user
    await authStore.fetchMe();
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to update profile');
    throw err;
  } finally {
    setIsLoading(false);
  }
}

export async function changePassword(newPassword: string): Promise<void> {
  const userId = authStore.user()?.id;
  if (!userId) throw new Error('Not authenticated');

  setIsLoading(true);
  setError(null);

  try {
    const token = authStore.token;
    const response = await fetch(`/api/v1/users/${userId}/password`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify({ new_password: newPassword }),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.message || 'Failed to change password');
    }
  } catch (err) {
    setError(err instanceof Error ? err.message : 'Failed to change password');
    throw err;
  } finally {
    setIsLoading(false);
  }
}

// ============================================
// Computed
// ============================================

export function getDisplayName(user?: { username?: string; display_name?: string; first_name?: string; last_name?: string } | null): string {
  if (!user) return 'Unknown';

  const mode = preferences.teammate_name_display;

  switch (mode) {
    case 'full_name':
      if (user.first_name || user.last_name) {
        return `${user.first_name || ''} ${user.last_name || ''}`.trim();
      }
      return user.display_name || user.username || 'Unknown';
    case 'nickname':
      return user.display_name || user.username || 'Unknown';
    case 'username':
    default:
      return user.username || 'Unknown';
  }
}

export function getInitials(user?: { username?: string; first_name?: string; last_name?: string; display_name?: string } | null): string {
  if (!user) return '?';

  if (user.first_name || user.last_name) {
    return `${user.first_name?.[0] || ''}${user.last_name?.[0] || ''}`.toUpperCase();
  }

  if (user.display_name) {
    const parts = user.display_name.split(' ');
    if (parts.length > 1) {
      return `${parts[0][0]}${parts[parts.length - 1][0]}`.toUpperCase();
    }
    return user.display_name.slice(0, 2).toUpperCase();
  }

  return user.username?.slice(0, 2).toUpperCase() || '?';
}

// ============================================
// Exports
// ============================================

export const userStore = {
  // State
  preferences,
  profile,
  isLoading,
  error,

  // Actions
  updatePreferences,
  resetPreferences,
  fetchUserProfile,
  updateProfile,
  changePassword,

  // Helpers
  getDisplayName,
  getInitials,
};

export default userStore;
