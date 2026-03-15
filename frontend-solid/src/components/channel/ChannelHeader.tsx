// ============================================
// ChannelHeader - Enhanced Channel Header Component
// ============================================

import { Show, createMemo, createSignal } from 'solid-js';
import { channelStore, currentChannel, fetchChannelMembers } from '@/stores/channels';
import { authStore } from '@/stores/auth';
import { callsStore } from '@/stores/calls';
import { uiStore, setActivePanel } from '@/stores/ui';
import { cn } from '@/utils/cn';
import { toast } from '@/hooks/useToast';

// Icons
import {
  HiOutlineHashtag,
  HiOutlineLockClosed,
  HiOutlineUser,
  HiOutlineUsers,
  HiOutlineMagnifyingGlass,
  HiOutlineBookmark,
  HiOutlineCog6Tooth,
  HiOutlineInformationCircle,
  HiOutlinePhone,
  HiOutlineVideoCamera,
  HiOutlineChevronDown,
} from 'solid-icons/hi';

// ============================================
// Types
// ============================================

interface ChannelHeaderProps {
  onSearchClick?: () => void;
}

// ============================================
// Channel Header Component
// ============================================

export function ChannelHeader(props: ChannelHeaderProps) {
  const channel = currentChannel;
  const [showInfoMenu, setShowInfoMenu] = createSignal(false);
  const [startingCallMode, setStartingCallMode] = createSignal<'voice' | 'video' | null>(null);

  // Get member count
  const memberCount = createMemo(() => {
    const id = channel()?.id;
    return id ? channelStore.membersByChannel[id]?.length || 0 : 0;
  });

  // Check if user is admin/owner
  const canManageChannel = createMemo(() => {
    const user = authStore.user();
    if (!user) return false;
    // In real app, check channel roles
    return user.role === 'system_admin' || user.role === 'team_admin';
  });

  // Channel icon based on type
  const channelIcon = createMemo(() => {
    const type = channel()?.channel_type;
    if (type === 'private') return <HiOutlineLockClosed size={20} />;
    if (type === 'direct') return <HiOutlineUser size={20} />;
    return <HiOutlineHashtag size={20} />;
  });

  // Open members panel
  const openMembersPanel = () => {
    setActivePanel('members');
    uiStore.setRightPanelOpen(true);
    // Fetch members if needed
    const channelId = channel()?.id;
    if (channelId && !channelStore.membersByChannel[channelId]?.length) {
      fetchChannelMembers(channelId);
    }
  };

  // Open pinned messages
  const openPinnedPanel = () => {
    setActivePanel('pinned');
    uiStore.setRightPanelOpen(true);
  };

  const openSavedPanel = () => {
    setActivePanel('saved');
    uiStore.setRightPanelOpen(true);
  };

  const openSearchPanel = () => {
    if (typeof window !== 'undefined' && window.innerWidth >= 1024) {
      setActivePanel('search');
      uiStore.setRightPanelOpen(true);
      return;
    }
    props.onSearchClick?.();
  };

  // Open channel info
  const openInfoPanel = () => {
    setActivePanel('info');
    uiStore.setRightPanelOpen(true);
  };

  // Open files panel
  const openFilesPanel = () => {
    setActivePanel('files');
    uiStore.setRightPanelOpen(true);
  };

  // Start call
  const startCall = async (mode: 'voice' | 'video') => {
    const channelId = channel()?.id;
    if (!channelId || startingCallMode()) {
      return;
    }

    setStartingCallMode(mode);
    try {
      await callsStore.startOrJoinCall(channelId, mode);
    } catch (error) {
      toast.error(
        'Unable to start call',
        error instanceof Error ? error.message : 'Unexpected error while starting call.'
      );
    } finally {
      setStartingCallMode(null);
    }
  };

  return (
    <header class="h-14 flex items-center justify-between px-4 border-b border-border-1 bg-bg-surface-1 shrink-0">
      {/* Left: Channel Info */}
      <div class="flex items-center gap-3 min-w-0">
        {/* Channel Icon & Name */}
        <button
          type="button"
          class="flex items-center gap-2 text-text-2 hover:text-text-1 transition-colors"
          onClick={() => setShowInfoMenu(!showInfoMenu())}
        >
          <span class="text-text-2">{channelIcon()}</span>
          <h1 class="font-semibold text-text-1 truncate">{channel()?.display_name}</h1>
          <HiOutlineChevronDown size={16} class="text-text-3" />
        </button>

        {/* Topic (hidden on mobile) */}
        <Show when={channel()?.header}>
          <span class="text-text-3 hidden md:inline">|</span>
          <p class="text-text-3 text-sm hidden md:block truncate max-w-xs lg:max-w-md">
            {channel()?.header}
          </p>
        </Show>
      </div>

      {/* Right: Actions */}
      <div class="flex items-center gap-1">
        {/* Search in Channel */}
        <button
          type="button"
          class={cn(
            'p-2 rounded-lg transition-colors',
            'text-text-2 hover:text-text-1 hover:bg-bg-surface-2'
          )}
          onClick={openSearchPanel}
          title="Search in channel (Ctrl+F)"
        >
          <HiOutlineMagnifyingGlass size={20} />
        </button>

        {/* Pinned Messages */}
        <button
          type="button"
          class={cn(
            'p-2 rounded-lg transition-colors',
            'text-text-2 hover:text-text-1 hover:bg-bg-surface-2'
          )}
          onClick={openPinnedPanel}
          title="Pinned messages"
        >
          <HiOutlineBookmark size={20} />
        </button>

        {/* Files */}
        <button
          type="button"
          class={cn(
            'p-2 rounded-lg transition-colors hidden sm:flex',
            'text-text-2 hover:text-text-1 hover:bg-bg-surface-2'
          )}
          onClick={openFilesPanel}
          title="Recent files"
        >
          <HiOutlineInformationCircle size={20} />
        </button>

        {/* Divider */}
        <div class="w-px h-6 bg-border-1 mx-1 hidden md:block" />

        {/* Call Buttons */}
        <button
          type="button"
          class={cn(
            'p-2 rounded-lg transition-colors hidden md:flex',
            'text-text-2 hover:text-brand hover:bg-brand/10',
            startingCallMode() === 'voice' && 'opacity-60 cursor-not-allowed'
          )}
          onClick={() => {
            void startCall('voice');
          }}
          title="Start voice call"
          disabled={Boolean(startingCallMode())}
        >
          <HiOutlinePhone size={20} />
        </button>

        <button
          type="button"
          class={cn(
            'p-2 rounded-lg transition-colors hidden md:flex',
            'text-text-2 hover:text-brand hover:bg-brand/10',
            startingCallMode() === 'video' && 'opacity-60 cursor-not-allowed'
          )}
          onClick={() => {
            void startCall('video');
          }}
          title="Start video call"
          disabled={Boolean(startingCallMode())}
        >
          <HiOutlineVideoCamera size={20} />
        </button>

        {/* Member Count */}
        <button
          type="button"
          class={cn(
            'flex items-center gap-2 px-3 py-1.5 rounded-lg transition-colors ml-1',
            'text-text-2 hover:text-text-1 hover:bg-bg-surface-2'
          )}
          onClick={openMembersPanel}
          title="View members"
        >
          <HiOutlineUsers size={18} />
          <span class="text-sm font-medium">{memberCount()}</span>
        </button>

        {/* Channel Settings (admin only) */}
        <Show when={canManageChannel()}>
          <button
            type="button"
            class={cn(
              'p-2 rounded-lg transition-colors',
              'text-text-2 hover:text-text-1 hover:bg-bg-surface-2'
            )}
            onClick={openInfoPanel}
            title="Channel settings"
          >
            <HiOutlineCog6Tooth size={20} />
          </button>
        </Show>
      </div>

      {/* Channel Info Dropdown Menu */}
      <Show when={showInfoMenu()}>
        <>
          <div
            class="fixed inset-0 z-40"
            onClick={() => setShowInfoMenu(false)}
          />
          <div class="absolute top-14 left-4 w-72 bg-bg-surface-1 border border-border-1 rounded-lg shadow-lg z-50 py-2">
            <div class="px-4 py-2 border-b border-border-1">
              <h3 class="font-semibold text-text-1">{channel()?.display_name}</h3>
              <p class="text-xs text-text-3 mt-1 capitalize">
                {channel()?.channel_type} channel
              </p>
            </div>

            <Show when={channel()?.header}>
              <div class="px-4 py-3 border-b border-border-1">
                <p class="text-sm text-text-2">{channel()?.header}</p>
              </div>
            </Show>

            <Show when={channel()?.purpose}>
              <div class="px-4 py-3 border-b border-border-1">
                <p class="text-xs text-text-3 uppercase mb-1">Purpose</p>
                <p class="text-sm text-text-2">{channel()?.purpose}</p>
              </div>
            </Show>

            <div class="p-2">
              <button
                type="button"
                class="w-full flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-bg-surface-2 transition-colors text-left"
                onClick={() => {
                  openInfoPanel();
                  setShowInfoMenu(false);
                }}
              >
                <HiOutlineInformationCircle size={18} class="text-text-2" />
                <span class="text-sm text-text-1">Channel details</span>
              </button>

              <button
                type="button"
                class="w-full flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-bg-surface-2 transition-colors text-left"
                onClick={() => {
                  openMembersPanel();
                  setShowInfoMenu(false);
                }}
              >
                <HiOutlineUsers size={18} class="text-text-2" />
                <span class="text-sm text-text-1">View members</span>
                <span class="ml-auto text-xs text-text-3">{memberCount()}</span>
              </button>

              <button
                type="button"
                class="w-full flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-bg-surface-2 transition-colors text-left"
                onClick={() => {
                  openSavedPanel();
                  setShowInfoMenu(false);
                }}
              >
                <HiOutlineBookmark size={18} class="text-text-2" />
                <span class="text-sm text-text-1">Saved messages</span>
              </button>

              <button
                type="button"
                class="w-full flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-bg-surface-2 transition-colors text-left"
                onClick={() => {
                  openPinnedPanel();
                  setShowInfoMenu(false);
                }}
              >
                <HiOutlineBookmark size={18} class="text-text-2" />
                <span class="text-sm text-text-1">Pinned messages</span>
              </button>
            </div>
          </div>
        </>
      </Show>
    </header>
  );
}

export default ChannelHeader;
