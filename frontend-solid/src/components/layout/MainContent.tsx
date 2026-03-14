// ============================================
// MainContent - Flexible Content Area
// ============================================

import { JSX, Show, createEffect } from 'solid-js';
import { uiStore } from '@/stores/ui';
import { channelStore } from '@/stores/channels';
import { cn } from '@/utils/cn';

// Icons
import {
  HiOutlineUsers,
  HiOutlinePaperClip,
  HiOutlineBookmark,
  HiOutlineInformationCircle,
} from 'solid-icons/hi';

// ============================================
// Props Interface
// ============================================

export interface MainContentProps {
  children: JSX.Element;
}

// ============================================
// MainContent Component
// ============================================

export function MainContent(props: MainContentProps) {
  // Track current channel for title updates
  createEffect(() => {
    const channel = channelStore.currentChannel();
    if (channel && typeof document !== 'undefined') {
      document.title = `${channel.display_name} - RustChat`;
    } else if (typeof document !== 'undefined') {
      document.title = 'RustChat';
    }
  });

  return (
    <main id="main-content" class="flex-1 flex flex-col min-w-0 bg-bg-app" tabIndex={-1}>
      {/* Channel Header (when in a channel) */}
      <Show when={channelStore.currentChannel()}>
        {(channel) => <ChannelHeader channel={channel()} />}
      </Show>

      {/* Scrollable Content Area */}
      <div
        class={cn(
          'flex-1 overflow-y-auto custom-scrollbar',
          !channelStore.currentChannel() && 'p-6'
        )}
      >
        {props.children}
      </div>
    </main>
  );
}

// ============================================
// Channel Header Component
// ============================================

import type { Channel } from '@/stores/channels';

interface ChannelHeaderProps {
  channel: Channel;
}

function ChannelHeader(props: ChannelHeaderProps) {
  const isRightPanelOpen = () => uiStore.preferences.rightPanelOpen;

  const channelIcon = () => {
    switch (props.channel.channel_type) {
      case 'direct':
        return <HiOutlineUsers size={20} class="text-text-3" />;
      case 'private':
        return <span class="text-text-3 font-bold text-lg">🔒</span>;
      default:
        return <span class="text-text-3 font-bold text-lg">#</span>;
    }
  };

  return (
    <header class="h-14 border-b border-border-1 flex items-center px-4 bg-bg-surface-1 shrink-0">
      {/* Left: Channel Info */}
      <div class="flex items-center gap-3 flex-1 min-w-0">
        {channelIcon()}
        <div class="min-w-0">
          <h1 class="font-semibold text-text-1 truncate">{props.channel.display_name}</h1>
          <Show when={props.channel.header}>
            <p class="text-xs text-text-3 truncate">{props.channel.header}</p>
          </Show>
        </div>
      </div>

      {/* Right: Actions */}
      <div class="flex items-center gap-1 ml-4">
        {/* Members Button */}
        <button
          type="button"
          class={cn(
            'flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm transition-colors',
            isRightPanelOpen() && uiStore.activePanelTab() === 'members'
              ? 'bg-brand/10 text-brand'
              : 'text-text-2 hover:bg-bg-surface-2 hover:text-text-1'
          )}
          onClick={() => uiStore.setActivePanel('members')}
          aria-pressed={isRightPanelOpen() && uiStore.activePanelTab() === 'members'}
        >
          <HiOutlineUsers size={18} />
          <span class="hidden sm:inline">Members</span>
        </button>

        {/* Pinned Messages Button */}
        <button
          type="button"
          class={cn(
            'flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm transition-colors',
            isRightPanelOpen() && uiStore.activePanelTab() === 'pinned'
              ? 'bg-brand/10 text-brand'
              : 'text-text-2 hover:bg-bg-surface-2 hover:text-text-1'
          )}
          onClick={() => uiStore.setActivePanel('pinned')}
          aria-pressed={isRightPanelOpen() && uiStore.activePanelTab() === 'pinned'}
        >
          <HiOutlineBookmark size={18} />
          <span class="hidden sm:inline">Pinned</span>
        </button>

        {/* Files Button */}
        <button
          type="button"
          class={cn(
            'flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm transition-colors',
            isRightPanelOpen() && uiStore.activePanelTab() === 'files'
              ? 'bg-brand/10 text-brand'
              : 'text-text-2 hover:bg-bg-surface-2 hover:text-text-1'
          )}
          onClick={() => uiStore.setActivePanel('files')}
          aria-pressed={isRightPanelOpen() && uiStore.activePanelTab() === 'files'}
        >
          <HiOutlinePaperClip size={18} />
          <span class="hidden sm:inline">Files</span>
        </button>

        {/* Channel Info Button */}
        <button
          type="button"
          class={cn(
            'flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-sm transition-colors',
            isRightPanelOpen() && uiStore.activePanelTab() === 'info'
              ? 'bg-brand/10 text-brand'
              : 'text-text-2 hover:bg-bg-surface-2 hover:text-text-1'
          )}
          onClick={() => uiStore.setActivePanel('info')}
          aria-pressed={isRightPanelOpen() && uiStore.activePanelTab() === 'info'}
        >
          <HiOutlineInformationCircle size={18} />
        </button>
      </div>
    </header>
  );
}

export default MainContent;
