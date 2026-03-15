// ============================================
// Settings Route - User Settings Framework
// ============================================

import { Show, For, createSignal, createMemo, createEffect, onMount, onCleanup } from 'solid-js';
import { useNavigate, useLocation } from '@solidjs/router';
import { logout, authStore } from '../stores/auth';
import {
  userStore,
  updatePreferences,
  resetPreferences,
  updateProfile,
  changePassword,
  uploadAvatar,
  removeAvatar,
} from '../stores/user';
import { useTheme, AVAILABLE_THEMES } from '../stores/theme';
import { resolveDefaultChannelPath, channelStore } from '../stores/channels';
import { isAdminRole } from '../utils/roles';
import { avatarSizedUrl } from '../utils/avatar';

import { requestPermission } from '../hooks/useDesktopNotifications';
import { SoundSettings } from '../utils/sounds';
import Button from '../components/ui/Button';

// ============================================
// Settings Sections Configuration
// ============================================

interface SettingsSection {
  id: string;
  label: string;
  icon: string;
  adminOnly?: boolean;
}

const sections: SettingsSection[] = [
  { id: 'profile', label: 'Profile', icon: '👤' },
  { id: 'security', label: 'Security', icon: '🔒' },
  { id: 'notifications', label: 'Notifications', icon: '🔔' },
  { id: 'display', label: 'Display', icon: '🎨' },
  { id: 'sidebar', label: 'Sidebar', icon: '📑' },
  { id: 'sounds', label: 'Sounds', icon: '🔊' },
  { id: 'advanced', label: 'Advanced', icon: '⚙️' },
];
const SETTINGS_RETURN_KEY = 'rustchat_settings_return_to';
const CLOSE_RESOLUTION_TIMEOUT_MS = 1200;

function normalizeReturnPath(path: string | null | undefined): string | null {
  if (!path || !path.startsWith('/') || path.startsWith('//')) {
    return null;
  }

  try {
    const parsed = new URL(path, window.location.origin);
    const normalized = `${parsed.pathname}${parsed.search}${parsed.hash}`;
    if (parsed.pathname.startsWith('/settings') || parsed.pathname.startsWith('/login')) {
      return null;
    }
    return normalized;
  } catch {
    return null;
  }
}

// ============================================
// Main Settings Component
// ============================================

export default function Settings() {
  const navigate = useNavigate();
  const location = useLocation();
  const isAdmin = () => isAdminRole(authStore.user()?.role);
  const visibleSections = createMemo(() =>
    sections.filter((section) => !section.adminOnly || isAdmin())
  );

  const currentSection = () => {
    const match = location.pathname.match(/^\/settings\/([^/?#]+)/);
    const section = match?.[1] || 'profile';
    return visibleSections().some((item) => item.id === section) ? section : 'profile';
  };

  const handleSectionClick = (sectionId: string) => {
    navigate(`/settings/${sectionId}`);
  };

  const closeSettings = async () => {
    let explicitReturnPath: string | null = null;
    try {
      explicitReturnPath = normalizeReturnPath(sessionStorage.getItem(SETTINGS_RETURN_KEY));
      sessionStorage.removeItem(SETTINGS_RETURN_KEY);
    } catch {
      explicitReturnPath = null;
    }

    if (explicitReturnPath && explicitReturnPath !== location.pathname) {
      navigate(explicitReturnPath, { replace: true });
      return;
    }

    const selectedChannelId = channelStore.currentChannelId();
    if (selectedChannelId) {
      navigate(`/channels/${selectedChannelId}`, { replace: true });
      return;
    }

    const defaultPath = await Promise.race<string | null>([
      resolveDefaultChannelPath(),
      new Promise((resolve) => {
        window.setTimeout(() => resolve(null), CLOSE_RESOLUTION_TIMEOUT_MS);
      }),
    ]);
    const normalizedDefaultPath = normalizeReturnPath(defaultPath);
    navigate(normalizedDefaultPath || '/', { replace: true });
  };

  onMount(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        void closeSettings();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    onCleanup(() => {
      window.removeEventListener('keydown', onKeyDown);
    });
  });

  const handleLogout = async () => {
    await logout('manual');
    navigate('/login');
  };

  return (
    <div class="fixed inset-0 z-[80] flex items-center justify-center p-3 sm:p-6" role="dialog" aria-modal="true">
      <div
        class="absolute inset-0 bg-black/55 backdrop-blur-[1px]"
        onClick={() => {
          void closeSettings();
        }}
        aria-hidden="true"
      />

      <div class="relative w-full max-w-6xl h-[92vh] bg-bg-surface-1 rounded-xl border border-border-1 shadow-2xl overflow-hidden flex flex-col">
        <div class="flex items-center justify-between px-4 sm:px-6 py-4 border-b border-border-1 shrink-0">
          <div>
            <h1 class="font-semibold text-text-1 text-lg">Settings</h1>
            <p class="text-sm text-text-3 mt-0.5">Manage your preferences</p>
          </div>
          <button
            type="button"
            class="p-2 rounded-lg text-text-3 hover:text-text-1 hover:bg-bg-surface-2 transition-colors"
            onClick={() => {
              void closeSettings();
            }}
            aria-label="Close settings"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M18 6 6 18" />
              <path d="m6 6 12 12" />
            </svg>
          </button>
        </div>

        <div class="flex-1 min-h-0 flex flex-col md:flex-row">
          {/* Settings Sidebar */}
          <aside class="w-full md:w-64 border-b md:border-b-0 md:border-r border-border-1 bg-bg-surface-1 flex flex-col shrink-0" data-testid="settings-sidebar">
            <nav class="flex md:flex-col gap-1 p-2 md:space-y-0.5 overflow-x-auto md:overflow-y-auto">
              <For each={visibleSections()}>
                {(section) => (
                  <button
                    type="button"
                    onClick={() => handleSectionClick(section.id)}
                    class={`
                  shrink-0 md:shrink
                  w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-left transition-all
                  ${currentSection() === section.id
                    ? 'bg-brand/10 text-brand font-medium border border-brand/20'
                    : 'text-text-2 hover:bg-bg-surface-2 hover:text-text-1'
                  }
                `}
                  >
                    <span class="text-lg">{section.icon}</span>
                    <span>{section.label}</span>
                  </button>
                )}
              </For>
            </nav>

            <div class="p-4 border-t border-border-1 space-y-3 hidden md:block">
              <Button
                variant="secondary"
                size="sm"
                class="w-full justify-center"
                onClick={() => {
                  void closeSettings();
                }}
              >
                Close
              </Button>
              <Button variant="danger" size="sm" class="w-full justify-center" onClick={handleLogout}>
                Sign Out
              </Button>
            </div>
          </aside>

          {/* Settings Content */}
          <main class="flex-1 overflow-y-auto p-4 sm:p-6 lg:p-8">
            <div class="max-w-2xl mx-auto">
              <Show when={currentSection() === 'profile'}>
                <ProfileSettings />
              </Show>

              <Show when={currentSection() === 'security'}>
                <SecuritySettings />
              </Show>

              <Show when={currentSection() === 'notifications'}>
                <NotificationSettings />
              </Show>

              <Show when={currentSection() === 'display'}>
                <DisplaySettings />
              </Show>

              <Show when={currentSection() === 'sidebar'}>
                <SidebarSettings />
              </Show>

              <Show when={currentSection() === 'sounds'}>
                <SoundSettingsPanel />
              </Show>

              <Show when={currentSection() === 'advanced'}>
                <AdvancedSettings />
              </Show>
            </div>
          </main>
        </div>
      </div>
    </div>
  );
}

// ============================================
// Profile Settings
// ============================================

function ProfileSettings() {
  const user = authStore.user;
  const [isEditing, setIsEditing] = createSignal(false);
  const [username, setUsername] = createSignal(user()?.username || '');
  const [displayName, setDisplayName] = createSignal(user()?.display_name || '');
  const [firstName, setFirstName] = createSignal(user()?.first_name || '');
  const [lastName, setLastName] = createSignal(user()?.last_name || '');
  const [position, setPosition] = createSignal(user()?.position || '');
  const [isSaving, setIsSaving] = createSignal(false);
  const [saveError, setSaveError] = createSignal('');
  const [usernameError, setUsernameError] = createSignal('');
  const [saveSuccess, setSaveSuccess] = createSignal(false);
  const [avatarError, setAvatarError] = createSignal('');
  const [isUploadingAvatar, setIsUploadingAvatar] = createSignal(false);
  let avatarInputRef: HTMLInputElement | undefined;
  const syncFromUser = () => {
    const current = user();
    if (!current) return;
    setUsername(current.username || '');
    setDisplayName(current.display_name || '');
    setFirstName(current.first_name || '');
    setLastName(current.last_name || '');
    setPosition(current.position || '');
  };

  createEffect(() => {
    user();
    if (!isEditing()) {
      syncFromUser();
    }
  });

  const initials = createMemo(() => {
    const u = user();
    if (!u) return '?';
    if (firstName() && lastName()) {
      return `${firstName()[0]}${lastName()[0]}`.toUpperCase();
    }
    return (u.username || u.display_name || '?').slice(0, 2).toUpperCase();
  });

  const handleSave = async () => {
    const normalizedUsername = username().trim();
    if (!normalizedUsername) {
      setUsernameError('Username is required');
      setSaveError('');
      return;
    }

    setIsSaving(true);
    setSaveError('');
    setUsernameError('');
    setSaveSuccess(false);

    try {
      await updateProfile({
        username: normalizedUsername,
        display_name: displayName().trim(),
        first_name: firstName().trim(),
        last_name: lastName().trim(),
        position: position().trim(),
      });
      syncFromUser();
      setUsernameError('');
      setSaveSuccess(true);
      setIsEditing(false);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to save profile';
      if (/username|users_username_key|duplicate/i.test(message)) {
        setUsernameError('Username is unavailable. Choose another one.');
      } else {
        setSaveError(message);
      }
    } finally {
      setIsSaving(false);
    }
  };

  const handleAvatarInput = async (event: Event) => {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    if (!file.type.startsWith('image/')) {
      setAvatarError('Please select a valid image file.');
      input.value = '';
      return;
    }

    if (file.size > 5 * 1024 * 1024) {
      setAvatarError('Image size must be less than 5MB.');
      input.value = '';
      return;
    }

    setAvatarError('');
    setSaveSuccess(false);
    setIsUploadingAvatar(true);
    try {
      await uploadAvatar(file);
      setSaveSuccess(true);
    } catch (error) {
      setAvatarError(error instanceof Error ? error.message : 'Failed to upload profile picture.');
    } finally {
      setIsUploadingAvatar(false);
      input.value = '';
    }
  };

  const handleAvatarRemove = async () => {
    setAvatarError('');
    setSaveSuccess(false);
    setIsUploadingAvatar(true);
    try {
      await removeAvatar();
      setSaveSuccess(true);
    } catch (error) {
      setAvatarError(error instanceof Error ? error.message : 'Failed to remove profile picture.');
    } finally {
      setIsUploadingAvatar(false);
    }
  };

  return (
    <div class="space-y-6">
      <div>
        <h2 class="text-2xl font-bold text-text-1">Profile</h2>
        <p class="text-text-3 mt-1">Manage your personal information and how others see you</p>
      </div>

      {/* Avatar Section */}
      <div class="p-6 bg-bg-surface-1 rounded-xl border border-border-1">
        <div class="flex items-center gap-6">
          <Show
            when={user()?.avatar_url}
            fallback={
              <div class="w-24 h-24 rounded-full bg-brand/10 flex items-center justify-center text-brand text-3xl font-bold">
                {initials()}
              </div>
            }
          >
            <img
              src={avatarSizedUrl(user()?.avatar_url, 'xl')}
              alt={user()?.username}
              class="w-24 h-24 rounded-full object-cover"
            />
          </Show>
          <div>
            <h3 class="font-semibold text-text-1 mb-1">Profile Picture</h3>
            <p class="text-sm text-text-3 mb-3">Supported: JPG, PNG, GIF (max 5MB)</p>
            <input
              ref={avatarInputRef}
              type="file"
              accept="image/*"
              class="hidden"
              onChange={(event) => {
                void handleAvatarInput(event);
              }}
            />
            <div class="flex gap-2">
              <Button
                variant="secondary"
                size="sm"
                disabled={!isEditing() || isUploadingAvatar()}
                onClick={() => avatarInputRef?.click()}
              >
                {isUploadingAvatar() ? 'Uploading...' : 'Upload New'}
              </Button>
              <Show when={user()?.avatar_url}>
                <Button
                  variant="ghost"
                  size="sm"
                  disabled={!isEditing() || isUploadingAvatar()}
                  onClick={() => {
                    void handleAvatarRemove();
                  }}
                >
                  Remove
                </Button>
              </Show>
            </div>
            <Show when={avatarError()}>
              <p class="text-xs text-danger mt-2">{avatarError()}</p>
            </Show>
            <Show when={!isEditing()}>
              <p class="text-xs text-text-3 mt-2">Click Edit to change your profile picture.</p>
            </Show>
          </div>
        </div>
      </div>

      {/* Profile Form */}
      <div class="p-6 bg-bg-surface-1 rounded-xl border border-border-1 space-y-4">
        <div class="flex items-center justify-between">
          <h3 class="font-semibold text-text-1">Personal Information</h3>
          <Button
            variant={isEditing() ? 'ghost' : 'secondary'}
            size="sm"
            onClick={() => {
              if (isEditing()) {
                setIsEditing(false);
                syncFromUser();
                setSaveError('');
                setUsernameError('');
                setSaveSuccess(false);
                return;
              }
              setSaveError('');
              setUsernameError('');
              setSaveSuccess(false);
              setIsEditing(true);
            }}
          >
            {isEditing() ? 'Cancel' : 'Edit'}
          </Button>
        </div>

        <Show when={saveError()}>
          <div class="p-3 bg-danger/10 border border-danger/20 rounded-lg text-danger text-sm">
            {saveError()}
          </div>
        </Show>
        <Show when={saveSuccess()}>
          <div class="p-3 bg-success/10 border border-success/20 rounded-lg text-success text-sm">
            Profile updated successfully.
          </div>
        </Show>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-text-2 mb-1.5">Username</label>
            <input
              type="text"
              value={username()}
              onInput={(e) => setUsername(e.currentTarget.value)}
              disabled={!isEditing()}
              aria-invalid={usernameError() ? 'true' : 'false'}
              class="w-full px-3 py-2 bg-bg-app border border-border-1 rounded-lg text-text-1 disabled:bg-bg-surface-2"
            />
            <Show when={usernameError()}>
              <p class="text-xs text-danger mt-1">{usernameError()}</p>
            </Show>
            <p class="text-xs text-text-3 mt-1">Username changes are saved immediately when you click Save Changes</p>
          </div>
          <div>
            <label class="block text-sm font-medium text-text-2 mb-1.5">Email</label>
            <input
              type="email"
              value={user()?.email || ''}
              disabled
              class="w-full px-3 py-2 bg-bg-app border border-border-1 rounded-lg text-text-1 disabled:text-text-3"
            />
            <p class="text-xs text-text-3 mt-1">Contact admin to change email</p>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-text-2 mb-1.5">First Name</label>
            <input
              type="text"
              value={firstName()}
              onInput={(e) => setFirstName(e.currentTarget.value)}
              disabled={!isEditing()}
              placeholder="Add first name"
              class="w-full px-3 py-2 bg-bg-app border border-border-1 rounded-lg text-text-1 disabled:bg-bg-surface-2"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-text-2 mb-1.5">Last Name</label>
            <input
              type="text"
              value={lastName()}
              onInput={(e) => setLastName(e.currentTarget.value)}
              disabled={!isEditing()}
              placeholder="Add last name"
              class="w-full px-3 py-2 bg-bg-app border border-border-1 rounded-lg text-text-1 disabled:bg-bg-surface-2"
            />
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium text-text-2 mb-1.5">Display Name</label>
          <input
            type="text"
            value={displayName()}
            onInput={(e) => setDisplayName(e.currentTarget.value)}
            disabled={!isEditing()}
            placeholder="How you want to be called"
            class="w-full px-3 py-2 bg-bg-app border border-border-1 rounded-lg text-text-1 disabled:bg-bg-surface-2"
          />
          <p class="text-xs text-text-3 mt-1">This is how your name appears in channels</p>
        </div>

        <div>
          <label class="block text-sm font-medium text-text-2 mb-1.5">Position / Title</label>
          <input
            type="text"
            value={position()}
            onInput={(e) => setPosition(e.currentTarget.value)}
            disabled={!isEditing()}
            placeholder="e.g. Software Engineer"
            class="w-full px-3 py-2 bg-bg-app border border-border-1 rounded-lg text-text-1 disabled:bg-bg-surface-2"
          />
        </div>

        <Show when={isEditing()}>
          <div class="pt-4 border-t border-border-1 flex gap-2">
            <Button variant="primary" onClick={handleSave} disabled={isSaving()}>
              {isSaving() ? 'Saving...' : 'Save Changes'}
            </Button>
            <Button
              variant="ghost"
              onClick={() => {
                setIsEditing(false);
                syncFromUser();
                setSaveError('');
                setUsernameError('');
                setSaveSuccess(false);
              }}
            >
              Cancel
            </Button>
          </div>
        </Show>
      </div>
    </div>
  );
}

// ============================================
// Security Settings
// ============================================

function SecuritySettings() {
  const [currentPassword, setCurrentPassword] = createSignal('');
  const [newPassword, setNewPassword] = createSignal('');
  const [confirmPassword, setConfirmPassword] = createSignal('');
  const [passwordError, setPasswordError] = createSignal('');
  const [passwordSuccess, setPasswordSuccess] = createSignal(false);
  const [isSubmitting, setIsSubmitting] = createSignal(false);
  const [isRevokingSessions, setIsRevokingSessions] = createSignal(false);
  const [sessionsNotice, setSessionsNotice] = createSignal('');
  const [sessionsNoticeType, setSessionsNoticeType] = createSignal<'success' | 'error'>('success');

  const handleChangePassword = async () => {
    setPasswordError('');
    setPasswordSuccess(false);

    if (!currentPassword() || !newPassword() || !confirmPassword()) {
      setPasswordError('All fields are required');
      return;
    }

    if (newPassword() !== confirmPassword()) {
      setPasswordError('New passwords do not match');
      return;
    }

    if (newPassword().length < 8) {
      setPasswordError('Password must be at least 8 characters');
      return;
    }

    setIsSubmitting(true);
    try {
      await changePassword(currentPassword(), newPassword());
      setPasswordSuccess(true);
      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');
    } catch (error) {
      setPasswordError(error instanceof Error ? error.message : 'Failed to change password');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleRevokeOtherSessions = async () => {
    const token = authStore.token;
    if (!token) return;

    setSessionsNotice('');
    setIsRevokingSessions(true);
    try {
      const response = await fetch('/api/v4/users/sessions/revoke/all', {
        method: 'POST',
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      if (!response.ok) {
        const payload = (await response.json().catch(() => null)) as
          | { message?: string; error?: string; detailed_error?: string }
          | null;
        throw new Error(
          payload?.message ||
          payload?.error ||
          payload?.detailed_error ||
          'Failed to revoke other sessions'
        );
      }

      setSessionsNotice('All other sessions have been revoked.');
      setSessionsNoticeType('success');
    } catch (error) {
      setSessionsNotice(error instanceof Error ? error.message : 'Failed to revoke other sessions');
      setSessionsNoticeType('error');
    } finally {
      setIsRevokingSessions(false);
    }
  };

  return (
    <div class="space-y-6">
      <div>
        <h2 class="text-2xl font-bold text-text-1">Security</h2>
        <p class="text-text-3 mt-1">Manage your password and security settings</p>
      </div>

      {/* Change Password */}
      <div class="p-6 bg-bg-surface-1 rounded-xl border border-border-1 space-y-4">
        <h3 class="font-semibold text-text-1">Change Password</h3>

        <Show when={passwordError()}>
          <div class="p-3 bg-danger/10 border border-danger/20 rounded-lg text-danger text-sm">
            {passwordError()}
          </div>
        </Show>

        <Show when={passwordSuccess()}>
          <div class="p-3 bg-success/10 border border-success/20 rounded-lg text-success text-sm">
            Password changed successfully!
          </div>
        </Show>

        <div class="space-y-3">
          <div>
            <label class="block text-sm font-medium text-text-2 mb-1.5">Current Password</label>
            <input
              type="password"
              value={currentPassword()}
              onInput={(e) => setCurrentPassword(e.currentTarget.value)}
              class="w-full px-3 py-2 bg-bg-app border border-border-1 rounded-lg text-text-1 focus:outline-none focus:border-brand"
              placeholder="Enter current password"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-text-2 mb-1.5">New Password</label>
            <input
              type="password"
              value={newPassword()}
              onInput={(e) => setNewPassword(e.currentTarget.value)}
              class="w-full px-3 py-2 bg-bg-app border border-border-1 rounded-lg text-text-1 focus:outline-none focus:border-brand"
              placeholder="Enter new password"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-text-2 mb-1.5">Confirm New Password</label>
            <input
              type="password"
              value={confirmPassword()}
              onInput={(e) => setConfirmPassword(e.currentTarget.value)}
              class="w-full px-3 py-2 bg-bg-app border border-border-1 rounded-lg text-text-1 focus:outline-none focus:border-brand"
              placeholder="Confirm new password"
            />
          </div>
        </div>

        <div class="pt-2">
          <Button variant="primary" onClick={handleChangePassword} disabled={isSubmitting()}>
            {isSubmitting() ? 'Changing...' : 'Change Password'}
          </Button>
        </div>
      </div>

      {/* Sessions */}
      <div class="p-6 bg-bg-surface-1 rounded-xl border border-border-1 space-y-4">
        <h3 class="font-semibold text-text-1">Active Sessions</h3>
        <p class="text-sm text-text-3">You're currently signed in on this device</p>
        <Show when={sessionsNotice()}>
          <div
            class={`p-3 rounded-lg text-sm ${
              sessionsNoticeType() === 'success'
                ? 'bg-success/10 border border-success/20 text-success'
                : 'bg-danger/10 border border-danger/20 text-danger'
            }`}
          >
            {sessionsNotice()}
          </div>
        </Show>

        <div class="flex items-center justify-between p-3 bg-bg-app rounded-lg border border-border-1">
          <div class="flex items-center gap-3">
            <div class="text-2xl">💻</div>
            <div>
              <p class="font-medium text-text-1">Current Session</p>
              <p class="text-xs text-text-3">Started just now • Web Browser</p>
            </div>
          </div>
          <span class="text-xs px-2 py-1 bg-success/10 text-success rounded-full">Active</span>
        </div>

        <Button variant="danger" size="sm" onClick={handleRevokeOtherSessions} disabled={isRevokingSessions()}>
          {isRevokingSessions() ? 'Revoking...' : 'Sign Out All Other Sessions'}
        </Button>
      </div>

      {/* Two-Factor Authentication */}
      <div class="p-6 bg-bg-surface-1 rounded-xl border border-border-1 space-y-4">
        <div class="flex items-center justify-between">
          <div>
            <h3 class="font-semibold text-text-1">Two-Factor Authentication</h3>
            <p class="text-sm text-text-3 mt-0.5">Add an extra layer of security to your account</p>
          </div>
          <Button variant="secondary" size="sm" disabled>
            Coming Soon
          </Button>
        </div>
      </div>
    </div>
  );
}

// ============================================
// Notification Settings
// ============================================

function NotificationSettings() {
  const [desktopEnabled, setDesktopEnabled] = createSignal(Notification.permission === 'granted');
  const prefs = () => userStore.preferences;

  const handleDesktopToggle = async () => {
    if (desktopEnabled()) {
      // Disable
      setDesktopEnabled(false);
    } else {
      // Request permission
      const permission = await requestPermission();
      setDesktopEnabled(permission === 'granted');
    }
  };

  return (
    <div class="space-y-6">
      <div>
        <h2 class="text-2xl font-bold text-text-1">Notifications</h2>
        <p class="text-text-3 mt-1">Configure how and when you receive notifications</p>
      </div>

      {/* Desktop Notifications */}
      <div class="p-6 bg-bg-surface-1 rounded-xl border border-border-1 space-y-4">
        <div class="flex items-center justify-between">
          <div>
            <h3 class="font-semibold text-text-1">Desktop Notifications</h3>
            <p class="text-sm text-text-3 mt-0.5">Show notifications on your desktop</p>
          </div>
          <label class="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={desktopEnabled()}
              onChange={handleDesktopToggle}
              class="sr-only peer"
            />
            <div class="w-11 h-6 bg-bg-hover peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-brand"></div>
          </label>
        </div>

        <Show when={!desktopEnabled() && Notification.permission === 'denied'}>
          <div class="p-3 bg-warning/10 border border-warning/20 rounded-lg text-warning text-sm">
            Desktop notifications are blocked. Please enable them in your browser settings.
          </div>
        </Show>
      </div>

      {/* Notification Triggers */}
      <div class="p-6 bg-bg-surface-1 rounded-xl border border-border-1 space-y-4">
        <h3 class="font-semibold text-text-1">Notify me about...</h3>

        <div class="space-y-3">
          <ToggleSetting
            label="Direct messages"
            description="When someone sends you a direct message"
            checked={prefs().notify_dm}
            onChange={(v) => updatePreferences({ notify_dm: v })}
          />

          <ToggleSetting
            label="Mentions"
            description="When someone mentions you with @username"
            checked={prefs().notify_mention}
            onChange={(v) => updatePreferences({ notify_mention: v })}
          />

          <ToggleSetting
            label="Channel mentions"
            description="When someone mentions @channel or @here"
            checked={prefs().notify_channel_mention}
            onChange={(v) => updatePreferences({ notify_channel_mention: v })}
          />

          <ToggleSetting
            label="Thread replies"
            description="When someone replies to a thread you're following"
            checked={prefs().notify_thread}
            onChange={(v) => updatePreferences({ notify_thread: v })}
          />
        </div>
      </div>

      {/* Email Notifications */}
      <div class="p-6 bg-bg-surface-1 rounded-xl border border-border-1 space-y-4">
        <h3 class="font-semibold text-text-1">Email Notifications</h3>

        <div class="space-y-3">
          <ToggleSetting
            label="Missed messages"
            description="Send daily digest of messages you missed"
            checked={prefs().email_digest}
            onChange={(v) => updatePreferences({ email_digest: v })}
          />
        </div>
      </div>
    </div>
  );
}

// ============================================
// Display Settings
// ============================================

function DisplaySettings() {
  const themeContext = useTheme();
  const prefs = () => userStore.preferences;

  return (
    <div class="space-y-6">
      <div>
        <h2 class="text-2xl font-bold text-text-1">Display</h2>
        <p class="text-text-3 mt-1">Customize your visual experience</p>
      </div>

      {/* Theme Selection */}
      <div class="p-6 bg-bg-surface-1 rounded-xl border border-border-1 space-y-4">
        <h3 class="font-semibold text-text-1">Theme</h3>
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
          <For each={AVAILABLE_THEMES}>
            {(themeOption) => (
              <button
                onClick={() => themeContext.setTheme(themeOption)}
                class={`
                  p-3 rounded-xl border-2 text-center transition-all
                  ${themeContext.theme() === themeOption
                    ? 'border-brand bg-brand/5'
                    : 'border-border-1 hover:border-border-2'
                  }
                `}
              >
                <div class="text-2xl mb-1">
                  {themeOption === 'light' && '☀️'}
                  {themeOption === 'dark' && '🌙'}
                  {themeOption === 'modern' && '✨'}
                  {themeOption === 'metallic' && '🔩'}
                  {themeOption === 'futuristic' && '🚀'}
                  {themeOption === 'high-contrast' && '👁️'}
                  {themeOption === 'simple' && '📝'}
                  {themeOption === 'dynamic' && '🎭'}
                </div>
                <div class="text-sm font-medium capitalize text-text-1">
                  {themeOption.replace('-', ' ')}
                </div>
              </button>
            )}
          </For>
        </div>
      </div>

      {/* Message Display */}
      <div class="p-6 bg-bg-surface-1 rounded-xl border border-border-1 space-y-4">
        <h3 class="font-semibold text-text-1">Message Display</h3>

        <div class="space-y-3">
          <SelectSetting
            label="Message Density"
            description="How compact messages appear in channels"
            value={prefs().message_display}
            options={[
              { value: 'clean', label: 'Clean' },
              { value: 'compact', label: 'Compact' },
              { value: 'standard', label: 'Standard' },
            ]}
            onChange={(v) => updatePreferences({ message_display: v as 'clean' | 'compact' | 'standard' })}
          />

          <SelectSetting
            label="Clock Display"
            description="Time format for timestamps"
            value={prefs().clock_display}
            options={[
              { value: '12hour', label: '12-hour (1:30 PM)' },
              { value: '24hour', label: '24-hour (13:30)' },
            ]}
            onChange={(v) => updatePreferences({ clock_display: v as '12hour' | '24hour' })}
          />

          <SelectSetting
            label="Name Display"
            description="How user names appear in messages"
            value={prefs().name_format || 'username'}
            options={[
              { value: 'username', label: 'Username' },
              { value: 'full_name', label: 'Full Name' },
              { value: 'nickname', label: 'Nickname' },
            ]}
            onChange={(v) => updatePreferences({ name_format: v as 'username' | 'full_name' | 'nickname' })}
          />
        </div>
      </div>

      {/* Accessibility */}
      <div class="p-6 bg-bg-surface-1 rounded-xl border border-border-1 space-y-4">
        <h3 class="font-semibold text-text-1">Accessibility</h3>

        <ToggleSetting
          label="Reduced Motion"
          description="Minimize animations throughout the interface"
          checked={prefs().reduced_motion}
          onChange={(v) => updatePreferences({ reduced_motion: v })}
        />

        <ToggleSetting
          label="Larger Text"
          description="Increase text size throughout the interface"
          checked={prefs().larger_text}
          onChange={(v) => updatePreferences({ larger_text: v })}
        />
      </div>
    </div>
  );
}

// ============================================
// Sidebar Settings
// ============================================

function SidebarSettings() {
  const prefs = () => userStore.preferences;

  return (
    <div class="space-y-6">
      <div>
        <h2 class="text-2xl font-bold text-text-1">Sidebar</h2>
        <p class="text-text-3 mt-1">Customize your sidebar layout and behavior</p>
      </div>

      <div class="p-6 bg-bg-surface-1 rounded-xl border border-border-1 space-y-4">
        <ToggleSetting
          label="Auto-collapse Categories"
          description="Collapse sidebar categories when not in use"
          checked={prefs().sidebar_collapsed}
          onChange={(v) => updatePreferences({ sidebar_collapsed: v })}
        />

        <SelectSetting
          label="Channel Sorting"
          description="How channels are ordered in the sidebar"
          value={prefs().channel_switcher_mode}
          options={[
            { value: 'recent', label: 'Recent Activity' },
            { value: 'alpha', label: 'Alphabetical' },
            { value: 'manual', label: 'Manual Order' },
          ]}
          onChange={(v) => updatePreferences({ channel_switcher_mode: v as 'recent' | 'alpha' | 'manual' })}
        />

        <ToggleSetting
          label="Unread Indicators"
          description="Show scrollbar indicators for unread messages"
          checked={prefs().enable_unread_scrollbar}
          onChange={(v) => updatePreferences({ enable_unread_scrollbar: v })}
        />

        <ToggleSetting
          label="Show Badges"
          description="Show unread count badges on channels"
          checked={prefs().show_badges !== false}
          onChange={(v) => updatePreferences({ show_badges: v })}
        />
      </div>
    </div>
  );
}

// ============================================
// Sound Settings Panel
// ============================================

function SoundSettingsPanel() {
  const [soundsEnabled, setSoundsEnabled] = createSignal(SoundSettings.isEnabled());

  const handleToggle = () => {
    const newValue = SoundSettings.toggle();
    setSoundsEnabled(newValue);
  };

  const playTestSound = (type: 'newMessage' | 'mention' | 'directMessage') => {
    import('../utils/sounds').then((module) => {
      switch (type) {
        case 'newMessage':
          module.Sounds.newMessage();
          break;
        case 'mention':
          module.Sounds.mention();
          break;
        case 'directMessage':
          module.Sounds.directMessage();
          break;
      }
    });
  };

  return (
    <div class="space-y-6">
      <div>
        <h2 class="text-2xl font-bold text-text-1">Sounds</h2>
        <p class="text-text-3 mt-1">Configure notification sounds</p>
      </div>

      {/* Master Toggle */}
      <div class="p-6 bg-bg-surface-1 rounded-xl border border-border-1">
        <div class="flex items-center justify-between">
          <div>
            <h3 class="font-semibold text-text-1">Enable Sounds</h3>
            <p class="text-sm text-text-3 mt-0.5">Play sounds for notifications</p>
          </div>
          <label class="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={soundsEnabled()}
              onChange={handleToggle}
              class="sr-only peer"
            />
            <div class="w-11 h-6 bg-bg-hover peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-brand"></div>
          </label>
        </div>
      </div>

      {/* Sound Types */}
      <div class="p-6 bg-bg-surface-1 rounded-xl border border-border-1 space-y-4">
        <h3 class="font-semibold text-text-1">Sound Types</h3>

        <div class="space-y-3">
          <div class="flex items-center justify-between p-3 bg-bg-app rounded-lg border border-border-1">
            <div>
              <p class="font-medium text-text-1">New Message</p>
              <p class="text-sm text-text-3">General message in any channel</p>
            </div>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => playTestSound('newMessage')}
              disabled={!soundsEnabled()}
            >
              Test
            </Button>
          </div>

          <div class="flex items-center justify-between p-3 bg-bg-app rounded-lg border border-border-1">
            <div>
              <p class="font-medium text-text-1">Mention</p>
              <p class="text-sm text-text-3">When someone mentions you with @username</p>
            </div>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => playTestSound('mention')}
              disabled={!soundsEnabled()}
            >
              Test
            </Button>
          </div>

          <div class="flex items-center justify-between p-3 bg-bg-app rounded-lg border border-border-1">
            <div>
              <p class="font-medium text-text-1">Direct Message</p>
              <p class="text-sm text-text-3">New direct message or DM group</p>
            </div>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => playTestSound('directMessage')}
              disabled={!soundsEnabled()}
            >
              Test
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}

// ============================================
// Advanced Settings
// ============================================

function AdvancedSettings() {
  const navigate = useNavigate();
  const [showResetConfirm, setShowResetConfirm] = createSignal(false);
  const isAdmin = () => isAdminRole(authStore.user()?.role);

  const handleClearCache = () => {
    localStorage.clear();
    window.location.reload();
  };

  const handleResetSettings = () => {
    resetPreferences();
    setShowResetConfirm(false);
  };

  return (
    <div class="space-y-6">
      <div>
        <h2 class="text-2xl font-bold text-text-1">Advanced</h2>
        <p class="text-text-3 mt-1">Advanced settings and troubleshooting</p>
      </div>

      {/* Data & Storage */}
      <div class="p-6 bg-bg-surface-1 rounded-xl border border-border-1 space-y-4">
        <h3 class="font-semibold text-text-1">Data & Storage</h3>

        <div class="space-y-3">
          <div class="flex items-center justify-between p-3 bg-bg-app rounded-lg border border-border-1">
            <div>
              <p class="font-medium text-text-1">Clear Cache</p>
              <p class="text-sm text-text-3">Clear local data and reload the app</p>
            </div>
            <Button variant="secondary" size="sm" onClick={handleClearCache}>
              Clear Cache
            </Button>
          </div>
        </div>
      </div>

      {/* Reset Settings */}
      <div class="p-6 bg-bg-surface-1 rounded-xl border border-border-1 space-y-4">
        <h3 class="font-semibold text-text-1">Reset Settings</h3>

        <Show when={!showResetConfirm()}>
          <p class="text-sm text-text-3">Reset all settings to their default values</p>
          <Button variant="danger" size="sm" onClick={() => setShowResetConfirm(true)}>
            Reset All Settings
          </Button>
        </Show>

        <Show when={showResetConfirm()}>
          <div class="p-3 bg-danger/10 border border-danger/20 rounded-lg">
            <p class="text-danger text-sm font-medium mb-2">
              Are you sure? This cannot be undone.
            </p>
            <div class="flex gap-2">
              <Button variant="danger" size="sm" onClick={handleResetSettings}>
                Yes, Reset Everything
              </Button>
              <Button variant="ghost" size="sm" onClick={() => setShowResetConfirm(false)}>
                Cancel
              </Button>
            </div>
          </div>
        </Show>
      </div>

      <Show when={isAdmin()}>
        <div class="p-6 bg-bg-surface-1 rounded-xl border border-border-1 space-y-4">
          <h3 class="font-semibold text-text-1">Administration</h3>
          <p class="text-sm text-text-3">
            Server configuration is managed in Admin Console.
          </p>
          <div class="flex flex-wrap gap-2">
            <Button
              variant="secondary"
              size="sm"
              onClick={() => navigate('/admin/settings')}
            >
              Open Server Configuration
            </Button>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => navigate('/admin/membership-policies')}
            >
              Open Membership Policies
            </Button>
            <Button
              variant="secondary"
              size="sm"
              onClick={() => navigate('/admin/email')}
            >
              Open Email Workflows
            </Button>
          </div>
        </div>
      </Show>

      {/* App Info */}
      <div class="p-6 bg-bg-surface-1 rounded-xl border border-border-1 space-y-4">
        <h3 class="font-semibold text-text-1">About</h3>

        <div class="grid grid-cols-2 gap-4 text-sm">
          <div>
            <span class="text-text-3">Version</span>
            <p class="text-text-1 font-medium">5.0.0-beta</p>
          </div>
          <div>
            <span class="text-text-3">Build</span>
            <p class="text-text-1 font-medium">{new Date().toISOString().split('T')[0]}</p>
          </div>
          <div>
            <span class="text-text-3">Framework</span>
            <p class="text-text-1 font-medium">Solid.js 1.9</p>
          </div>
          <div>
            <span class="text-text-3">API Version</span>
            <p class="text-text-1 font-medium">v4 (Mattermost compatible)</p>
          </div>
        </div>
      </div>
    </div>
  );
}

// ============================================
// Reusable Setting Components
// ============================================

interface ToggleSettingProps {
  label: string;
  description: string;
  checked: boolean;
  onChange: (value: boolean) => void;
  disabled?: boolean;
}

function ToggleSetting(props: ToggleSettingProps) {
  return (
    <div class="flex items-center justify-between py-2">
      <div>
        <p class="font-medium text-text-1">{props.label}</p>
        <p class="text-sm text-text-3">{props.description}</p>
      </div>
      <label class="relative inline-flex items-center cursor-pointer shrink-0 ml-4">
        <input
          type="checkbox"
          checked={props.checked}
          onChange={(e) => props.onChange(e.currentTarget.checked)}
          disabled={props.disabled}
          class="sr-only peer"
        />
        <div class="w-11 h-6 bg-bg-hover peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-brand disabled:opacity-50"></div>
      </label>
    </div>
  );
}

interface SelectSettingProps {
  label: string;
  description: string;
  value: string;
  options: { value: string; label: string }[];
  onChange: (value: string) => void;
}

function SelectSetting(props: SelectSettingProps) {
  return (
    <div class="flex items-center justify-between py-2">
      <div>
        <p class="font-medium text-text-1">{props.label}</p>
        <p class="text-sm text-text-3">{props.description}</p>
      </div>
      <select
        value={props.value}
        onChange={(e) => props.onChange(e.currentTarget.value)}
        class="px-3 py-1.5 bg-bg-app border border-border-1 rounded-lg text-text-1 text-sm focus:outline-none focus:border-brand shrink-0 ml-4"
      >
        <For each={props.options}>
          {(option) => (
            <option value={option.value}>{option.label}</option>
          )}
        </For>
      </select>
    </div>
  );
}
