// ============================================
// Header - Global Application Header
// ============================================

import { createSignal, Show, For, createEffect, createMemo } from 'solid-js';
import { A, useNavigate } from '@solidjs/router';
import { authStore, logout } from '@/stores/auth';
import { uiStore } from '@/stores/ui';
import { unreadStore } from '@/stores/unreads';
import { channelStore } from '@/stores/channels';
import { useIsMobile } from '@/hooks/useMediaQuery';
import { ConnectionIndicator } from '@/components/ConnectionStatus';
import { isAdminRole } from '@/utils/roles';

// Icons
import {
  HiOutlineBars3,
  HiOutlineBell,
  HiOutlineMagnifyingGlass,
  HiOutlineCommandLine,
  HiOutlineUserCircle,
  HiOutlineCog6Tooth,
  HiOutlineArrowRightOnRectangle,
  HiOutlineCheck,
} from 'solid-icons/hi';

// ============================================
// Header Component
// ============================================

export function Header() {
  const navigate = useNavigate();
  const isMobile = useIsMobile();
  const [userMenuOpen, setUserMenuOpen] = createSignal(false);
  const [notificationsOpen, setNotificationsOpen] = createSignal(false);

  const user = () => authStore.user();
  const displayName = () => {
    const u = user();
    return u?.display_name || u?.username || 'User';
  };
  const isAdmin = () => isAdminRole(user()?.role);

  const saveSettingsReturnTarget = () => {
    try {
      const current = `${window.location.pathname}${window.location.search}${window.location.hash}`;
      sessionStorage.setItem('rustchat_settings_return_to', current);
    } catch {
      // noop
    }
  };

  const initials = () => {
    const u = user();
    if (!u) return '?';
    if (u.first_name && u.last_name) {
      return `${u.first_name[0]}${u.last_name[0]}`.toUpperCase();
    }
    return (u.username || u.display_name || '?').slice(0, 2).toUpperCase();
  };

  const handleLogout = async () => {
    setUserMenuOpen(false);
    await logout('manual');
  };

  const openCommandPalette = () => {
    // Dispatch custom event for command palette
    window.dispatchEvent(new CustomEvent('rustchat:open-command-palette'));
  };

  const unreadCount = () => unreadStore.totalMentionCount() || unreadStore.totalUnreadCount();
  const notifications = createMemo(() => {
    const allChannelIds = new Set<string>([
      ...Object.keys(unreadStore.channelUnreads),
      ...Object.keys(unreadStore.channelMentions),
    ]);

    return Array.from(allChannelIds)
      .map((channelId) => {
        const unread = unreadStore.channelUnreads[channelId] || 0;
        const mentions = unreadStore.channelMentions[channelId] || 0;
        const channel = channelStore.getChannel(channelId);
        const channelName = channel?.display_name || channel?.name || channelId.slice(0, 8);
        const isMention = mentions > 0;
        return {
          id: channelId,
          channelId,
          unread,
          mentions,
          read: unread === 0 && mentions === 0,
          title: isMention ? 'New mention' : 'Unread messages',
          message: isMention
            ? `${mentions} mention${mentions > 1 ? 's' : ''} in #${channelName}`
            : `${unread} unread message${unread > 1 ? 's' : ''} in #${channelName}`,
          time: 'recent',
        };
      })
      .filter((item) => item.unread > 0 || item.mentions > 0)
      .sort((a, b) => b.mentions - a.mentions || b.unread - a.unread);
  });

  const openNotificationChannel = async (channelId: string) => {
    setNotificationsOpen(false);
    navigate(`/channels/${channelId}`);
    await unreadStore.markAsRead(channelId);
  };

  createEffect(() => {
    if (authStore.isAuthenticated) {
      void unreadStore.fetchOverview();
    }
  });

  return (
    <header class="h-14 bg-bg-surface-1 border-b border-border-1 flex items-center px-4 shrink-0 z-30">
      {/* Left Section: Menu + Logo */}
      <div class="flex items-center gap-3 flex-shrink-0">
        {/* Mobile Menu Button */}
        <Show when={isMobile()}>
          <button
            type="button"
            class="p-2 rounded-lg text-text-2 hover:bg-bg-surface-2 hover:text-text-1 transition-colors"
            onClick={() => uiStore.toggleMobileSidebar()}
            aria-label="Open sidebar"
          >
            <HiOutlineBars3 size={24} />
          </button>
        </Show>

        {/* Logo */}
        <A href="/" class="flex items-center gap-2 text-text-1 hover:text-brand transition-colors">
          <div class="w-8 h-8 rounded-lg bg-brand flex items-center justify-center">
            <span class="text-white font-bold text-lg">R</span>
          </div>
          <Show when={!isMobile()}>
            <span class="font-semibold text-lg hidden sm:block">RustChat</span>
          </Show>
        </A>
      </div>

      {/* Center Section: Search */}
      <div class="flex-1 flex justify-center px-4 max-w-2xl mx-auto">
        <button
          type="button"
          class="w-full max-w-md flex items-center gap-2 px-4 py-2 bg-bg-app border border-border-1 rounded-lg text-text-3 hover:border-border-2 hover:text-text-2 transition-all group"
          onClick={openCommandPalette}
          aria-label="Search or jump to..."
        >
          <HiOutlineMagnifyingGlass size={18} class="group-hover:text-text-1 transition-colors" />
          <span class="flex-1 text-left text-sm hidden sm:block">Search or jump to...</span>
          <Show when={!isMobile()}>
            <kbd class="hidden sm:flex items-center gap-1 px-1.5 py-0.5 bg-bg-surface-2 rounded text-xs text-text-3">
              <HiOutlineCommandLine size={12} />
              <span>K</span>
            </kbd>
          </Show>
        </button>
      </div>

      {/* Right Section: Actions + User */}
      <div class="flex items-center gap-1 sm:gap-2 flex-shrink-0">
        {/* Connection Status */}
        <ConnectionIndicator />

        {/* Command Palette Trigger (Desktop) */}
        <Show when={!isMobile()}>
          <button
            type="button"
            class="p-2 rounded-lg text-text-2 hover:bg-bg-surface-2 hover:text-text-1 transition-colors"
            onClick={openCommandPalette}
            aria-label="Command palette (Ctrl+K)"
            title="Command palette (Ctrl+K)"
          >
            <HiOutlineCommandLine size={20} />
          </button>
        </Show>

        {/* Notifications */}
        <div class="relative">
          <button
            type="button"
            class="p-2 rounded-lg text-text-2 hover:bg-bg-surface-2 hover:text-text-1 transition-colors relative"
            onClick={() => setNotificationsOpen((prev) => !prev)}
            aria-label={`Notifications ${unreadCount() > 0 ? `(${unreadCount()} unread)` : ''}`}
            aria-expanded={notificationsOpen()}
            aria-haspopup="menu"
          >
            <HiOutlineBell size={20} />
            <Show when={unreadCount() > 0}>
              <span class="absolute top-1 right-1 w-4 h-4 bg-danger text-white text-[10px] font-medium rounded-full flex items-center justify-center">
                {unreadCount()}
              </span>
            </Show>
          </button>

          {/* Notifications Dropdown */}
          <Show when={notificationsOpen()}>
            <>
              <div
                class="fixed inset-0 z-40"
                onClick={() => setNotificationsOpen(false)}
                aria-hidden="true"
              />
              <div class="absolute right-0 top-full mt-2 w-80 bg-bg-surface-1 border border-border-1 rounded-lg shadow-lg z-50 py-2">
                <div class="px-4 py-2 border-b border-border-1 flex items-center justify-between">
                  <h3 class="font-semibold text-text-1">Notifications</h3>
                  <button
                    type="button"
                    class="text-xs text-brand hover:underline"
                    onClick={() => {
                      void unreadStore.markAllAsRead();
                    }}
                  >
                    Mark all read
                  </button>
                </div>
                <div class="max-h-80 overflow-y-auto">
                  <Show
                    when={notifications().length > 0}
                    fallback={<p class="px-4 py-4 text-sm text-text-3">No unread notifications.</p>}
                  >
                    <For each={notifications()}>
                      {(notification) => (
                        <button
                          type="button"
                          class="w-full px-4 py-3 hover:bg-bg-surface-2 text-left cursor-pointer flex gap-3"
                          onClick={() => {
                            void openNotificationChannel(notification.channelId);
                          }}
                        >
                          <div class={`w-2 h-2 rounded-full mt-2 ${notification.read ? 'bg-transparent' : 'bg-brand'} flex-shrink-0`} />
                          <div class="flex-1">
                            <p class="text-sm font-medium text-text-1">{notification.title}</p>
                            <p class="text-xs text-text-3">{notification.message}</p>
                            <p class="text-xs text-text-3 mt-1">{notification.time}</p>
                          </div>
                        </button>
                      )}
                    </For>
                  </Show>
                </div>
                <div class="px-4 py-2 border-t border-border-1">
                  <button
                    type="button"
                    class="text-sm text-brand hover:underline block text-center w-full"
                    onClick={() => {
                      saveSettingsReturnTarget();
                      setNotificationsOpen(false);
                      navigate('/settings/notifications');
                    }}
                  >
                    View all notifications
                  </button>
                </div>
              </div>
            </>
          </Show>
        </div>

        {/* User Menu */}
        <div class="relative">
          <button
            type="button"
            class="flex items-center gap-2 p-1 rounded-lg hover:bg-bg-surface-2 transition-colors"
            onClick={() => setUserMenuOpen((prev) => !prev)}
            aria-label="User menu"
            aria-expanded={userMenuOpen()}
            aria-haspopup="menu"
          >
            <Show
              when={user()?.avatar_url}
              fallback={
                <div class="w-8 h-8 rounded-full bg-brand/10 flex items-center justify-center text-brand font-medium text-sm">
                  {initials()}
                </div>
              }
            >
              <img
                src={user()?.avatar_url}
                alt={displayName()}
                class="w-8 h-8 rounded-full object-cover"
              />
            </Show>
            <Show when={!isMobile()}>
              <span class="text-sm text-text-1 hidden md:block max-w-[120px] truncate">
                {displayName()}
              </span>
            </Show>
          </button>

          {/* User Dropdown */}
          <Show when={userMenuOpen()}>
            <>
              <div
                class="fixed inset-0 z-40"
                onClick={() => setUserMenuOpen(false)}
                aria-hidden="true"
              />
              <div class="absolute right-0 top-full mt-2 w-56 bg-bg-surface-1 border border-border-1 rounded-lg shadow-lg z-50 py-2">
                {/* User Info */}
                <div class="px-4 py-3 border-b border-border-1">
                  <p class="font-semibold text-text-1 truncate">{displayName()}</p>
                  <p class="text-xs text-text-3 truncate">@{user()?.username}</p>
                  <p class="text-xs text-text-3 mt-1 truncate">{user()?.email}</p>
                </div>

                {/* Menu Items */}
                <div class="py-1">
                  <button
                    type="button"
                    class="w-full px-4 py-2 text-left text-sm text-text-1 hover:bg-bg-surface-2 flex items-center gap-2"
                    onClick={() => {
                      saveSettingsReturnTarget();
                      setUserMenuOpen(false);
                      navigate('/settings/profile');
                    }}
                  >
                    <HiOutlineUserCircle size={18} class="text-text-3" />
                    Profile
                  </button>
                  <button
                    type="button"
                    class="w-full px-4 py-2 text-left text-sm text-text-1 hover:bg-bg-surface-2 flex items-center gap-2"
                    onClick={() => {
                      saveSettingsReturnTarget();
                      setUserMenuOpen(false);
                      navigate('/settings/profile');
                    }}
                  >
                    <HiOutlineCog6Tooth size={18} class="text-text-3" />
                    Settings
                  </button>
                  <Show when={isAdmin()}>
                    <button
                      type="button"
                      class="w-full px-4 py-2 text-left text-sm text-text-1 hover:bg-bg-surface-2 flex items-center gap-2"
                      onClick={() => {
                        setUserMenuOpen(false);
                        navigate('/admin');
                      }}
                    >
                      <HiOutlineCog6Tooth size={18} class="text-text-3" />
                      Admin Console
                    </button>
                  </Show>
                </div>

                {/* Status Section */}
                <div class="py-1 border-t border-border-1">
                  <p class="px-4 py-1 text-xs text-text-3 uppercase tracking-wider">Status</p>
                  <For each={['online', 'away', 'dnd', 'offline'] as const}>
                    {(status) => (
                      <button
                        type="button"
                        class="w-full px-4 py-2 text-left text-sm text-text-1 hover:bg-bg-surface-2 flex items-center gap-2"
                        onClick={() => {
                          authStore.updateStatus({ status });
                          setUserMenuOpen(false);
                        }}
                      >
                        <span
                          class={`w-2 h-2 rounded-full ${
                            status === 'online' ? 'bg-green-500' :
                            status === 'away' ? 'bg-yellow-500' :
                            status === 'dnd' ? 'bg-red-500' :
                            'bg-gray-400'
                          }`}
                        />
                        <span class="capitalize">{status === 'dnd' ? 'Do not disturb' : status}</span>
                        <Show when={user()?.presence === status}>
                          <HiOutlineCheck size={16} class="ml-auto text-brand" />
                        </Show>
                      </button>
                    )}
                  </For>
                </div>

                {/* Logout */}
                <div class="py-1 border-t border-border-1">
                  <button
                    type="button"
                    class="w-full px-4 py-2 text-left text-sm text-danger hover:bg-danger/10 flex items-center gap-2"
                    onClick={handleLogout}
                  >
                    <HiOutlineArrowRightOnRectangle size={18} />
                    Logout
                  </button>
                </div>
              </div>
            </>
          </Show>
        </div>
      </div>
    </header>
  );
}

export default Header;
