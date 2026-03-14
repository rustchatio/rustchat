// ============================================
// MobileNav - Bottom Navigation for Mobile
// ============================================

import { Show, For, createMemo } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { uiStore } from '@/stores/ui';
import { authStore } from '@/stores/auth';
import { unreadStore } from '@/stores/unreads';
import { isAdminRole } from '@/utils/roles';

// Icons
import {
  HiOutlineHome,
  HiOutlineHashtag,
  HiOutlineBell,
  HiOutlineUserCircle,
  HiOutlineCog6Tooth,
  HiOutlineBars3,
} from 'solid-icons/hi';

// ============================================
// MobileNav Component
// ============================================

export function MobileNav() {
  const navigate = useNavigate();
  const isAdmin = () => isAdminRole(authStore.user()?.role);
  const mentionsBadgeCount = () => unreadStore.totalMentionCount() || unreadStore.totalUnreadCount();

  const saveSettingsReturnTarget = () => {
    try {
      const current = `${window.location.pathname}${window.location.search}${window.location.hash}`;
      sessionStorage.setItem('rustchat_settings_return_to', current);
    } catch {
      // noop
    }
  };

  const navItems = createMemo(() => {
    const items = [
      {
        id: 'home',
        label: 'Home',
        icon: HiOutlineHome,
        onClick: () => navigate('/'),
      },
      {
        id: 'channels',
        label: 'Channels',
        icon: HiOutlineHashtag,
        onClick: () => {
          uiStore.toggleMobileSidebar();
        },
      },
      {
        id: 'mentions',
        label: 'Mentions',
        icon: HiOutlineBell,
        badge: mentionsBadgeCount,
        onClick: () => {
          uiStore.toggleMobileSidebar();
        },
      },
      {
        id: 'menu',
        label: 'Menu',
        icon: HiOutlineBars3,
        onClick: () => uiStore.toggleMobileSidebar(),
      },
      {
        id: 'profile',
        label: 'You',
        icon: HiOutlineUserCircle,
        onClick: () => {
          saveSettingsReturnTarget();
          navigate('/settings/profile');
        },
      },
    ];

    if (isAdmin()) {
      items.splice(items.length - 1, 0, {
        id: 'admin',
        label: 'Admin',
        icon: HiOutlineCog6Tooth,
        onClick: () => navigate('/admin'),
      });
    }

    return items;
  });

  return (
    <nav
      class="h-16 bg-bg-surface-1 border-t border-border-1 flex items-center justify-around px-2 shrink-0 z-30"
      role="navigation"
      aria-label="Mobile navigation"
    >
      <For each={navItems()}>
        {(item) => (
          <button
            type="button"
            class="flex flex-col items-center justify-center gap-1 flex-1 h-full min-w-[64px] rounded-lg text-text-3 hover:text-text-2 transition-colors"
            onClick={item.onClick}
            aria-label={item.label}
          >
            <div class="relative">
              <item.icon size={24} />
              <Show when={item.badge && item.badge() > 0}>
                <span class="absolute -top-1 -right-1 w-4 h-4 bg-danger text-white text-[10px] font-semibold rounded-full flex items-center justify-center">
                  {item.badge!()}
                </span>
              </Show>
            </div>
            <span class="text-[10px] font-medium">{item.label}</span>
          </button>
        )}
      </For>
    </nav>
  );
}

export default MobileNav;
