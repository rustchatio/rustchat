// ============================================
// Sidebar - Left Navigation Sidebar
// ============================================

import { Show, createSignal, For } from 'solid-js';
import { A } from '@solidjs/router';
import { channelStore, selectChannel, type Channel } from '@/stores/channels';
import { uiStore } from '@/stores/ui';
import { cn } from '@/utils/cn';

// Icons
import {
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlinePlus,
  HiOutlineHashtag,
  HiOutlineLockClosed,
  HiOutlineUser,
  HiOutlineStar,
  HiOutlineBars3,
  HiOutlineXMark,
} from 'solid-icons/hi';

// ============================================
// Props Interface
// ============================================

export interface SidebarProps {
  isMobile?: boolean;
}

// ============================================
// Sidebar Component
// ============================================

export function Sidebar(props: SidebarProps) {
  const isCollapsed = () => uiStore.preferences.sidebarCollapsed && !props.isMobile;

  const currentChannelId = () => channelStore.currentChannelId();
  const prefs = () => uiStore.preferences;

  // Section expansion states
  const isExpanded = (section: 'favorites' | 'channels' | 'directMessages') =>
    prefs().channelSectionsExpanded[section];

  // Filter channels by type
  const favoriteChannels = () => channelStore.favoriteChannels();
  const publicChannels = () => channelStore.publicChannels();
  const privateChannels = () => channelStore.privateChannels();
  const directMessages = () => channelStore.directMessages();

  return (
    <aside
      class={cn(
        'h-full bg-bg-surface-1 border-r border-border-1 flex flex-col shrink-0 transition-all duration-200',
        props.isMobile ? 'w-[280px]' : isCollapsed() ? 'w-16' : 'w-[260px]'
      )}
    >
      {/* Sidebar Header */}
      <div class="h-14 border-b border-border-1 flex items-center px-3 shrink-0">
        <Show
          when={!isCollapsed()}
          fallback={
            <button
              type="button"
              class="p-2 rounded-lg text-text-2 hover:bg-bg-surface-2 hover:text-text-1 transition-colors mx-auto"
              onClick={() => uiStore.toggleSidebar()}
              aria-label="Expand sidebar"
              title="Expand sidebar (Ctrl+B)"
            >
              <HiOutlineBars3 size={20} />
            </button>
          }
        >
          <div class="flex items-center justify-between w-full">
            {/* Team Selector */}
            <TeamSelector />

            <Show when={!props.isMobile}>
              <button
                type="button"
                class="p-1.5 rounded-lg text-text-3 hover:bg-bg-surface-2 hover:text-text-1 transition-colors"
                onClick={() => uiStore.toggleSidebar()}
                aria-label="Collapse sidebar"
                title="Collapse sidebar (Ctrl+B)"
              >
                <HiOutlineXMark size={18} />
              </button>
            </Show>
          </div>
        </Show>
      </div>

      {/* Channel List */}
      <div class="flex-1 overflow-y-auto custom-scrollbar py-2">
        <Show when={!isCollapsed()}>
          {/* Favorites Section */}
          <Show when={favoriteChannels().length > 0}>
            <ChannelSection
              title="Favorites"
              icon={<HiOutlineStar size={16} />}
              isExpanded={isExpanded('favorites')}
              onToggle={() => uiStore.toggleChannelSection('favorites')}
            >
              <For each={favoriteChannels()}>
                {(channel) => (
                  <ChannelItem
                    channel={channel}
                    isActive={currentChannelId() === channel.id}
                    onClick={() => handleChannelClick(channel.id)}
                  />
                )}
              </For>
            </ChannelSection>
          </Show>

          {/* Channels Section */}
          <ChannelSection
            title="Channels"
            icon={<HiOutlineHashtag size={16} />}
            isExpanded={isExpanded('channels')}
            onToggle={() => uiStore.toggleChannelSection('channels')}
            action={
              <button
                type="button"
                class="p-1 rounded text-text-3 hover:bg-bg-surface-2 hover:text-text-1 opacity-0 group-hover:opacity-100 transition-opacity"
                aria-label="Create channel"
                title="Create channel"
              >
                <HiOutlinePlus size={14} />
              </button>
            }
          >
            {/* Public Channels */}
            <For each={publicChannels()}>
              {(channel) => (
                <ChannelItem
                  channel={channel}
                  isActive={currentChannelId() === channel.id}
                  onClick={() => handleChannelClick(channel.id)}
                />
              )}
            </For>

            {/* Private Channels */}
            <For each={privateChannels()}>
              {(channel) => (
                <ChannelItem
                  channel={channel}
                  isActive={currentChannelId() === channel.id}
                  onClick={() => handleChannelClick(channel.id)}
                  isPrivate
                />
              )}
            </For>
          </ChannelSection>

          {/* Direct Messages Section */}
          <ChannelSection
            title="Direct Messages"
            icon={<HiOutlineUser size={16} />}
            isExpanded={isExpanded('directMessages')}
            onToggle={() => uiStore.toggleChannelSection('directMessages')}
            action={
              <button
                type="button"
                class="p-1 rounded text-text-3 hover:bg-bg-surface-2 hover:text-text-1 opacity-0 group-hover:opacity-100 transition-opacity"
                aria-label="Start direct message"
                title="Start direct message"
              >
                <HiOutlinePlus size={14} />
              </button>
            }
          >
            <For each={directMessages()}>
              {(channel) => (
                <ChannelItem
                  channel={channel}
                  isActive={currentChannelId() === channel.id}
                  onClick={() => handleChannelClick(channel.id)}
                  isDirectMessage
                />
              )}
            </For>
          </ChannelSection>
        </Show>

        {/* Collapsed State - Just Icons */}
        <Show when={isCollapsed() && !props.isMobile}>
          <div class="flex flex-col items-center gap-1 py-2">
            <For each={publicChannels().slice(0, 5)}>
              {(channel) => (
                <A
                  href={`/channels/${channel.id}`}
                  class={cn(
                    'w-10 h-10 rounded-lg flex items-center justify-center text-text-2 transition-colors',
                    currentChannelId() === channel.id
                      ? 'bg-brand/10 text-brand'
                      : 'hover:bg-bg-surface-2 hover:text-text-1'
                  )}
                  title={channel.display_name}
                >
                  <HiOutlineHashtag size={18} />
                </A>
              )}
            </For>
          </div>
        </Show>
      </div>

      {/* Sidebar Footer */}
      <Show when={!isCollapsed()}>
        <div class="p-3 border-t border-border-1 shrink-0">
          <button
            type="button"
            class="w-full flex items-center gap-2 px-3 py-2 rounded-lg bg-bg-app border border-border-1 hover:border-border-2 transition-colors text-left"
          >
            <HiOutlinePlus size={18} class="text-text-3" />
            <span class="text-sm text-text-2">Add Channel</span>
          </button>
        </div>
      </Show>
    </aside>
  );
}

// ============================================
// Team Selector Component
// ============================================

function TeamSelector() {
  const [isOpen, setIsOpen] = createSignal(false);

  // Mock team data
  const currentTeam = () => ({
    id: 'default',
    name: 'Default Team',
    display_name: 'RustChat',
  });

  const teams = () => [
    { id: 'default', name: 'Default Team', display_name: 'RustChat' },
  ];

  return (
    <div class="relative flex-1">
      <button
        type="button"
        class="flex items-center gap-2 px-2 py-1.5 rounded-lg hover:bg-bg-surface-2 transition-colors w-full text-left"
        onClick={() => setIsOpen((prev) => !prev)}
        aria-expanded={isOpen()}
        aria-haspopup="listbox"
      >
        <div class="w-6 h-6 rounded bg-brand flex items-center justify-center text-white text-xs font-bold">
          {currentTeam().display_name.charAt(0)}
        </div>
        <span class="font-semibold text-text-1 truncate">{currentTeam().display_name}</span>
        <HiOutlineChevronDown size={16} class="ml-auto text-text-3" />
      </button>

      <Show when={isOpen()}>
        <>
          <div
            class="fixed inset-0 z-40"
            onClick={() => setIsOpen(false)}
            aria-hidden="true"
          />
          <div class="absolute left-0 top-full mt-1 w-56 bg-bg-surface-1 border border-border-1 rounded-lg shadow-lg z-50 py-2">
            <div class="px-3 py-2 border-b border-border-1">
              <p class="text-xs text-text-3 uppercase tracking-wider">Your Teams</p>
            </div>
            <For each={teams()}>
              {(team) => (
                <button
                  type="button"
                  class={cn(
                    'w-full px-3 py-2 flex items-center gap-2 hover:bg-bg-surface-2 transition-colors',
                    team.id === currentTeam().id ? 'bg-bg-surface-2' : ''
                  )}
                  onClick={() => setIsOpen(false)}
                >
                  <div class="w-6 h-6 rounded bg-brand flex items-center justify-center text-white text-xs font-bold">
                    {team.display_name.charAt(0)}
                  </div>
                  <span class="text-sm text-text-1 truncate">{team.display_name}</span>
                </button>
              )}
            </For>
            <div class="border-t border-border-1 mt-1 pt-1">
              <button
                type="button"
                class="w-full px-3 py-2 text-left text-sm text-text-2 hover:bg-bg-surface-2 hover:text-text-1 flex items-center gap-2"
                onClick={() => setIsOpen(false)}
              >
                <HiOutlinePlus size={16} />
                Create or Join Team
              </button>
            </div>
          </div>
        </>
      </Show>
    </div>
  );
}

// ============================================
// Channel Section Component
// ============================================

interface ChannelSectionProps {
  title: string;
  icon: ReturnType<typeof HiOutlineHashtag>;
  isExpanded: boolean;
  onToggle: () => void;
  children: ReturnType<typeof For>;
  action?: ReturnType<typeof Show>;
}

function ChannelSection(props: ChannelSectionProps) {
  return (
    <div class="mb-1">
      <button
        type="button"
        class="group w-full flex items-center gap-1 px-3 py-1.5 text-text-3 hover:text-text-2 transition-colors"
        onClick={props.onToggle}
        aria-expanded={props.isExpanded}
      >
        {props.isExpanded ? (
          <HiOutlineChevronDown size={14} />
        ) : (
          <HiOutlineChevronRight size={14} />
        )}
        <span class="flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider">
          {props.icon}
          {props.title}
        </span>
        <Show when={props.action}>
          <span class="ml-auto">{props.action}</span>
        </Show>
      </button>
      <Show when={props.isExpanded}>
        <div class="mt-0.5">{props.children}</div>
      </Show>
    </div>
  );
}

// ============================================
// Channel Item Component
// ============================================

interface ChannelItemProps {
  channel: Channel;
  isActive: boolean;
  onClick: () => void;
  isPrivate?: boolean;
  isDirectMessage?: boolean;
}

function ChannelItem(props: ChannelItemProps) {
  const href = () => `/channels/${props.channel.id}`;

  const handleClick = (e: MouseEvent) => {
    // Handle middle-click or Ctrl+click to open in new tab
    if (e.ctrlKey || e.metaKey || e.button === 1) {
      return; // Let default behavior happen
    }
    e.preventDefault();
    props.onClick();
    // Close mobile sidebar
    uiStore.setMobileSidebarOpen(false);
  };

  const icon = () => {
    if (props.isDirectMessage) {
      return <HiOutlineUser size={16} />;
    }
    if (props.isPrivate) {
      return <HiOutlineLockClosed size={16} />;
    }
    return <HiOutlineHashtag size={16} />;
  };

  const unreadCount = () => props.channel.unreadCount || 0;
  const mentionCount = () => props.channel.mentionCount || 0;

  return (
    <A
      href={href()}
      onClick={handleClick}
      class={cn(
        'group flex items-center gap-2 px-3 py-1.5 mx-2 rounded-lg text-sm transition-colors',
        props.isActive
          ? 'bg-brand/10 text-brand'
          : unreadCount() > 0
          ? 'text-text-1 hover:bg-bg-surface-2'
          : 'text-text-2 hover:bg-bg-surface-2 hover:text-text-1'
      )}
      aria-current={props.isActive ? 'page' : undefined}
    >
      <span class={cn('flex-shrink-0', props.isActive ? 'text-brand' : 'text-text-3')}>
        {icon()}
      </span>
      <span class="flex-1 truncate font-medium">
        {props.channel.display_name}
      </span>
      <Show when={mentionCount() > 0}>
        <span class="flex-shrink-0 min-w-[18px] h-[18px] px-1.5 bg-danger text-white text-[10px] font-semibold rounded-full flex items-center justify-center">
          {mentionCount() > 99 ? '99+' : mentionCount()}
        </span>
      </Show>
      <Show when={unreadCount() > 0 && mentionCount() === 0}>
        <span class="flex-shrink-0 w-2 h-2 bg-brand rounded-full" />
      </Show>
    </A>
  );
}

// ============================================
// Handler Functions
// ============================================

function handleChannelClick(channelId: string) {
  selectChannel(channelId);
  uiStore.setMobileSidebarOpen(false);
}

export default Sidebar;
