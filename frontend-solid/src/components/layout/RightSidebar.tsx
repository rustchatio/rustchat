// ============================================
// RightSidebar - Collapsible Right Panel
// ============================================

import { Show, For, createSignal, createMemo } from 'solid-js';
import { uiStore, type PanelTab } from '@/stores/ui';
import { channelStore } from '@/stores/channels';
import { cn } from '@/utils/cn';

// Icons
import {
  HiOutlineXMark,
  HiOutlineUsers,
  HiOutlineBookmark,
  HiOutlinePaperClip,
  HiOutlineInformationCircle,
  HiOutlineMagnifyingGlass,
} from 'solid-icons/hi';

// ============================================
// Props Interface
// ============================================

export interface RightSidebarProps {
  // No props needed, uses stores
}

// ============================================
// RightSidebar Component
// ============================================

export function RightSidebar(_props: RightSidebarProps) {
  const isOpen = () => uiStore.preferences.rightPanelOpen;
  const activeTab = () => uiStore.activePanelTab();
  const width = () => uiStore.preferences.rightPanelWidth;

  return (
    <Show when={isOpen()}>
      <aside
        class="h-full bg-bg-surface-1 border-l border-border-1 flex flex-col shrink-0 transition-all duration-200 relative"
        style={{ width: `${width()}px` }}
      >
        {/* Resize Handle */}
        <ResizeHandle />

        {/* Tab Content */}
        <div class="flex-1 overflow-hidden flex flex-col">
          <SwitchTab activeTab={activeTab()} />
        </div>

        {/* Collapse Button */}
        <div class="p-2 border-t border-border-1 flex justify-center">
          <button
            type="button"
            class="p-2 rounded-lg text-text-3 hover:bg-bg-surface-2 hover:text-text-1 transition-colors"
            onClick={() => uiStore.setRightPanelOpen(false)}
            aria-label="Close sidebar"
            title="Close sidebar"
          >
            <HiOutlineXMark size={18} />
          </button>
        </div>
      </aside>
    </Show>
  );
}

// ============================================
// SwitchTab Component
// ============================================

interface SwitchTabProps {
  activeTab: PanelTab;
}

function SwitchTab(props: SwitchTabProps) {
  return (
    <>
      <Show when={props.activeTab === 'members'}>
        <MembersPanel />
      </Show>
      <Show when={props.activeTab === 'pinned'}>
        <PinnedMessagesPanel />
      </Show>
      <Show when={props.activeTab === 'files'}>
        <FilesPanel />
      </Show>
      <Show when={props.activeTab === 'info'}>
        <ChannelInfoPanel />
      </Show>
    </>
  );
}

// ============================================
// Members Panel
// ============================================

interface Member {
  user_id: string;
  roles?: string;
}

function MembersPanel() {
  const currentChannelId = () => channelStore.currentChannelId();
  const members = createMemo(() => {
    const id = currentChannelId();
    return id ? channelStore.membersByChannel[id] || [] : [];
  });

  const [searchQuery, setSearchQuery] = createSignal('');

  const filteredMembers = createMemo(() => {
    const allMembers = members();
    if (searchQuery()) {
      const query = searchQuery().toLowerCase();
      return allMembers.filter((m) => m.user_id.toLowerCase().includes(query));
    }
    return allMembers;
  });

  // Group by role (mock for now)
  const groupedMembers = createMemo(() => {
    const online: Member[] = [];
    const offline: Member[] = [];

    filteredMembers().forEach((member) => {
      // Mock online status - in real app, check presence store
      const isOnline = Math.random() > 0.3;
      if (isOnline) {
        online.push(member);
      } else {
        offline.push(member);
      }
    });

    return { Online: online, Offline: offline };
  });

  return (
    <>
      {/* Panel Header */}
      <div class="p-3 border-b border-border-1">
        <div class="flex items-center gap-2 mb-3">
          <HiOutlineUsers size={18} class="text-text-2" />
          <h2 class="font-semibold text-text-1">Members</h2>
          <span class="text-xs text-text-3">({members().length})</span>
        </div>

        {/* Search */}
        <div class="relative">
          <HiOutlineMagnifyingGlass
            size={16}
            class="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-3"
          />
          <input
            type="text"
            placeholder="Find members"
            value={searchQuery()}
            onInput={(e) => setSearchQuery(e.currentTarget.value)}
            class="w-full pl-9 pr-3 py-1.5 bg-bg-app border border-border-1 rounded-lg text-sm text-text-1 placeholder:text-text-3 focus:outline-none focus:border-brand"
          />
        </div>
      </div>

      {/* Members List */}
      <div class="flex-1 overflow-y-auto custom-scrollbar p-2">
        <For each={Object.entries(groupedMembers())}>
          {([status, statusMembers]) => (
            <Show when={statusMembers.length > 0}>
              <div class="mb-3">
                <h3 class="px-2 py-1 text-xs font-semibold text-text-3 uppercase tracking-wider">
                  {status} — {statusMembers.length}
                </h3>
                <For each={statusMembers}>
                  {(member) => <MemberItem member={member} status={status.toLowerCase() as 'online' | 'offline'} />}
                </For>
              </div>
            </Show>
          )}
        </For>
      </div>
    </>
  );
}

// ============================================
// Member Item Component
// ============================================

interface MemberItemProps {
  member: Member;
  status: 'online' | 'offline';
}

function MemberItem(props: MemberItemProps) {
  const initials = () => props.member.user_id.slice(0, 2).toUpperCase();

  return (
    <button
      type="button"
      class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg hover:bg-bg-surface-2 transition-colors text-left group"
    >
      <div class="relative">
        <div class="w-8 h-8 rounded-full bg-brand/10 flex items-center justify-center text-brand text-xs font-medium">
          {initials()}
        </div>
        <span
          class={cn(
            'absolute bottom-0 right-0 w-2.5 h-2.5 rounded-full border-2 border-bg-surface-1',
            props.status === 'online' ? 'bg-green-500' : 'bg-gray-400'
          )}
        />
      </div>
      <div class="flex-1 min-w-0">
        <p class="text-sm text-text-1 truncate">{props.member.user_id}</p>
        <Show when={props.member.roles}>
          <p class="text-xs text-text-3 truncate">{props.member.roles}</p>
        </Show>
      </div>
    </button>
  );
}

// ============================================
// Pinned Messages Panel
// ============================================

interface PinnedMessage {
  id: string;
  author: string;
  content: string;
  date: string;
}

function PinnedMessagesPanel() {
  // Mock pinned messages
  const pinnedMessages = (): PinnedMessage[] => [
    { id: '1', author: 'Alice', content: 'Welcome to the channel everyone!', date: '2 days ago' },
    { id: '2', author: 'Bob', content: 'Important announcement: Server maintenance tonight at 2 AM.', date: '1 week ago' },
  ];

  return (
    <>
      {/* Panel Header */}
      <div class="p-3 border-b border-border-1">
        <div class="flex items-center gap-2">
          <HiOutlineBookmark size={18} class="text-text-2" />
          <h2 class="font-semibold text-text-1">Pinned Messages</h2>
        </div>
      </div>

      {/* Pinned Messages List */}
      <div class="flex-1 overflow-y-auto custom-scrollbar p-3">
        <Show
          when={pinnedMessages().length > 0}
          fallback={
            <div class="text-center py-8 text-text-3">
              <HiOutlineBookmark size={32} class="mx-auto mb-2 opacity-50" />
              <p class="text-sm">No pinned messages</p>
            </div>
          }
        >
          <div class="space-y-3">
            <For each={pinnedMessages()}>
              {(message) => (
                <div class="p-3 bg-bg-app rounded-lg border border-border-1">
                  <div class="flex items-center gap-2 mb-2">
                    <div class="w-6 h-6 rounded-full bg-brand/10 flex items-center justify-center text-brand text-xs font-medium">
                      {message.author.charAt(0)}
                    </div>
                    <span class="text-sm font-medium text-text-1">{message.author}</span>
                    <span class="text-xs text-text-3 ml-auto">{message.date}</span>
                  </div>
                  <p class="text-sm text-text-2">{message.content}</p>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>
    </>
  );
}

// ============================================
// Files Panel
// ============================================

interface FileItem {
  id: string;
  name: string;
  size: string;
  author: string;
  date: string;
}

function FilesPanel() {
  // Mock files
  const files = (): FileItem[] => [
    { id: '1', name: 'document.pdf', size: '2.4 MB', author: 'Alice', date: 'Yesterday' },
    { id: '2', name: 'image.png', size: '856 KB', author: 'Bob', date: '3 days ago' },
  ];

  return (
    <>
      {/* Panel Header */}
      <div class="p-3 border-b border-border-1">
        <div class="flex items-center gap-2">
          <HiOutlinePaperClip size={18} class="text-text-2" />
          <h2 class="font-semibold text-text-1">Recent Files</h2>
        </div>
      </div>

      {/* Files List */}
      <div class="flex-1 overflow-y-auto custom-scrollbar p-2">
        <Show
          when={files().length > 0}
          fallback={
            <div class="text-center py-8 text-text-3">
              <HiOutlinePaperClip size={32} class="mx-auto mb-2 opacity-50" />
              <p class="text-sm">No files shared yet</p>
            </div>
          }
        >
          <div class="space-y-1">
            <For each={files()}>
              {(file) => (
                <button
                  type="button"
                  class="w-full p-2 rounded-lg hover:bg-bg-surface-2 transition-colors text-left"
                >
                  <div class="flex items-center gap-2">
                    <div class="w-8 h-8 rounded-lg bg-brand/10 flex items-center justify-center text-brand">
                      <FileIcon filename={file.name} />
                    </div>
                    <div class="flex-1 min-w-0">
                      <p class="text-sm text-text-1 truncate">{file.name}</p>
                      <p class="text-xs text-text-3">
                        {file.size} • {file.author}
                      </p>
                    </div>
                  </div>
                </button>
              )}
            </For>
          </div>
        </Show>
      </div>
    </>
  );
}

// ============================================
// File Icon Component
// ============================================

function FileIcon(props: { filename: string }) {
  const ext = () => props.filename.split('.').pop()?.toLowerCase() || '';

  const icon = () => {
    switch (ext()) {
      case 'pdf':
        return '📄';
      case 'png':
      case 'jpg':
      case 'jpeg':
      case 'gif':
        return '🖼️';
      case 'doc':
      case 'docx':
        return '📝';
      case 'xls':
      case 'xlsx':
        return '📊';
      default:
        return '📎';
    }
  };

  return <span class="text-sm">{icon()}</span>;
}

// ============================================
// Channel Info Panel
// ============================================

function ChannelInfoPanel() {
  const channel = () => channelStore.currentChannel();

  return (
    <>
      {/* Panel Header */}
      <div class="p-3 border-b border-border-1">
        <div class="flex items-center gap-2">
          <HiOutlineInformationCircle size={18} class="text-text-2" />
          <h2 class="font-semibold text-text-1">Channel Info</h2>
        </div>
      </div>

      {/* Channel Details */}
      <div class="flex-1 overflow-y-auto custom-scrollbar p-4">
        <Show when={channel()}>
          {(ch) => (
            <div class="space-y-4">
              {/* Channel Name */}
              <div>
                <h3 class="text-xs font-semibold text-text-3 uppercase tracking-wider mb-1">Name</h3>
                <p class="text-sm text-text-1">{ch().display_name}</p>
              </div>

              {/* Channel Purpose */}
              <Show when={ch().purpose}>
                <div>
                  <h3 class="text-xs font-semibold text-text-3 uppercase tracking-wider mb-1">Purpose</h3>
                  <p class="text-sm text-text-1">{ch().purpose}</p>
                </div>
              </Show>

              {/* Channel Header */}
              <Show when={ch().header}>
                <div>
                  <h3 class="text-xs font-semibold text-text-3 uppercase tracking-wider mb-1">Topic</h3>
                  <p class="text-sm text-text-1">{ch().header}</p>
                </div>
              </Show>

              {/* Channel Type */}
              <div>
                <h3 class="text-xs font-semibold text-text-3 uppercase tracking-wider mb-1">Type</h3>
                <p class="text-sm text-text-1 capitalize">{ch().channel_type}</p>
              </div>

              {/* Created */}
              <div>
                <h3 class="text-xs font-semibold text-text-3 uppercase tracking-wider mb-1">Created</h3>
                <p class="text-sm text-text-1">
                  {new Date(ch().created_at).toLocaleDateString()}
                </p>
              </div>

              {/* Actions */}
              <div class="pt-4 border-t border-border-1 space-y-2">
                <button
                  type="button"
                  class="w-full px-4 py-2 bg-bg-app border border-border-1 rounded-lg text-sm text-text-1 hover:border-border-2 transition-colors"
                >
                  Notification Preferences
                </button>
                <button
                  type="button"
                  class="w-full px-4 py-2 text-danger hover:bg-danger/10 rounded-lg text-sm transition-colors"
                >
                  Leave Channel
                </button>
              </div>
            </div>
          )}
        </Show>
      </div>
    </>
  );
}

// ============================================
// Resize Handle Component
// ============================================

function ResizeHandle() {
  let startX = 0;
  let startWidth = 0;

  const handleMouseDown = (e: MouseEvent) => {
    startX = e.clientX;
    startWidth = uiStore.preferences.rightPanelWidth;
    uiStore.startResizing();

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    // Prevent text selection
    e.preventDefault();
  };

  const handleMouseMove = (e: MouseEvent) => {
    const delta = startX - e.clientX;
    const newWidth = startWidth + delta;
    uiStore.setRightPanelWidth(newWidth);
  };

  const handleMouseUp = () => {
    uiStore.stopResizing();
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  };

  return (
    <div
      class="absolute left-0 top-0 bottom-0 w-1 cursor-col-resize hover:bg-brand/50 transition-colors z-10"
      onMouseDown={handleMouseDown}
      role="separator"
      aria-orientation="vertical"
      aria-label="Resize sidebar"
    />
  );
}

export default RightSidebar;
