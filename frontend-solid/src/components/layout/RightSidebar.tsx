// ============================================
// RightSidebar - Collapsible Right Panel
// ============================================

import { Show, For, createSignal, createMemo, createEffect } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { uiStore, type PanelTab } from '@/stores/ui';
import { authStore } from '@/stores/auth';
import { channelStore, fetchChannelMembers, leaveChannel, resolveDefaultChannelPath } from '@/stores/channels';
import { presenceStore, type Presence } from '@/stores/presence';
import { postsApi } from '@/api/messages';
import { client, getErrorMessage } from '@/api/client';
import { toast } from '@/hooks/useToast';
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
      <Show when={props.activeTab === 'search'}>
        <SearchPanel />
      </Show>
      <Show when={props.activeTab === 'saved'}>
        <SavedMessagesPanel />
      </Show>
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
// Search Panel
// ============================================

interface SearchResult {
  id: string;
  channelId: string;
  rootPostId?: string;
  author: string;
  content: string;
  createdAt: string;
}

function toPostTimestamp(value: string | number | undefined): string {
  if (typeof value === 'number') {
    return new Date(value).toISOString();
  }
  if (typeof value === 'string') {
    return value;
  }
  return new Date().toISOString();
}

function SearchPanel() {
  const navigate = useNavigate();
  const [query, setQuery] = createSignal('');
  const [isLoading, setIsLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [results, setResults] = createSignal<SearchResult[]>([]);

  createEffect(() => {
    const channelId = channelStore.currentChannelId();
    const terms = query().trim();

    if (!channelId || terms.length < 2) {
      setResults([]);
      setError(null);
      return;
    }

    let cancelled = false;
    setIsLoading(true);
    setError(null);

    const timer = window.setTimeout(() => {
      void postsApi
        .list(channelId, { q: terms, limit: 50 })
        .then((response) => {
          if (cancelled) return;
          const posts = Array.isArray(response.messages) ? response.messages : [];
          const mapped = posts.map((post) => ({
            id: post.id,
            channelId: post.channel_id,
            rootPostId: post.root_post_id,
            author: post.username || post.user_id || 'Unknown',
            content: post.message || '',
            createdAt: toPostTimestamp(post.created_at),
          }));
          setResults(mapped);
        })
        .catch((err) => {
          if (cancelled) return;
          setResults([]);
          setError(getErrorMessage(err) || 'Search failed.');
        })
        .finally(() => {
          if (!cancelled) {
            setIsLoading(false);
          }
        });
    }, 250);

    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  });

  const openResult = (result: SearchResult) => {
    uiStore.setRightPanelOpen(false);
    if (result.rootPostId) {
      navigate(`/channels/${result.channelId}/threads/${result.rootPostId}`);
      return;
    }
    navigate(`/channels/${result.channelId}`);
  };

  return (
    <>
      <div class="p-3 border-b border-border-1 space-y-3">
        <div class="flex items-center gap-2">
          <HiOutlineMagnifyingGlass size={18} class="text-text-2" />
          <h2 class="font-semibold text-text-1">Search</h2>
        </div>
        <input
          type="text"
          value={query()}
          onInput={(event) => setQuery(event.currentTarget.value)}
          placeholder="Search messages in this channel"
          class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1 placeholder:text-text-3 focus:outline-none focus:border-brand"
        />
      </div>
      <div class="flex-1 overflow-y-auto custom-scrollbar p-3">
        <Show when={isLoading()}>
          <p class="text-sm text-text-3">Searching...</p>
        </Show>
        <Show when={error()}>
          <p class="text-sm text-danger">{error()}</p>
        </Show>
        <Show
          when={!isLoading() && !error() && results().length > 0}
          fallback={
            <Show when={!isLoading() && !error()}>
              <p class="text-sm text-text-3">
                {query().trim().length < 2 ? 'Type at least 2 characters.' : 'No matches found.'}
              </p>
            </Show>
          }
        >
          <div class="space-y-2">
            <For each={results()}>
              {(result) => (
                <button
                  type="button"
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-left hover:border-border-2 hover:bg-bg-surface-2 transition-colors"
                  onClick={() => openResult(result)}
                >
                  <p class="text-xs text-text-3">{result.author}</p>
                  <p class="text-sm text-text-1 line-clamp-3 whitespace-pre-wrap break-words">{result.content}</p>
                  <p class="mt-1 text-xs text-text-3">{formatTimestamp(result.createdAt)}</p>
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
// Saved Messages Panel
// ============================================

function SavedMessagesPanel() {
  const navigate = useNavigate();
  const [isLoading, setIsLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [results, setResults] = createSignal<SearchResult[]>([]);

  createEffect(() => {
    if (!authStore.isAuthenticated) {
      setResults([]);
      return;
    }

    let cancelled = false;
    setIsLoading(true);
    setError(null);

    void postsApi
      .getSaved()
      .then((posts) => {
        if (cancelled) return;
        const mapped = posts.map((post) => ({
          id: post.id,
          channelId: post.channel_id,
          rootPostId: post.root_post_id,
          author: post.username || post.user_id || 'Unknown',
          content: post.message || '',
          createdAt: toPostTimestamp(post.created_at),
        }));
        setResults(mapped);
      })
      .catch((err) => {
        if (cancelled) return;
        setResults([]);
        setError(getErrorMessage(err) || 'Failed to load saved messages.');
      })
      .finally(() => {
        if (!cancelled) {
          setIsLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  });

  const openResult = (result: SearchResult) => {
    uiStore.setRightPanelOpen(false);
    if (result.rootPostId) {
      navigate(`/channels/${result.channelId}/threads/${result.rootPostId}`);
      return;
    }
    navigate(`/channels/${result.channelId}`);
  };

  return (
    <>
      <div class="p-3 border-b border-border-1">
        <div class="flex items-center gap-2">
          <HiOutlineBookmark size={18} class="text-text-2" />
          <h2 class="font-semibold text-text-1">Saved Messages</h2>
        </div>
      </div>
      <div class="flex-1 overflow-y-auto custom-scrollbar p-3">
        <Show when={isLoading()}>
          <p class="text-sm text-text-3">Loading saved messages...</p>
        </Show>
        <Show when={error()}>
          <p class="text-sm text-danger">{error()}</p>
        </Show>
        <Show
          when={!isLoading() && !error() && results().length > 0}
          fallback={
            <Show when={!isLoading() && !error()}>
              <p class="text-sm text-text-3">No saved messages.</p>
            </Show>
          }
        >
          <div class="space-y-2">
            <For each={results()}>
              {(result) => (
                <button
                  type="button"
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-left hover:border-border-2 hover:bg-bg-surface-2 transition-colors"
                  onClick={() => openResult(result)}
                >
                  <p class="text-xs text-text-3">{result.author}</p>
                  <p class="text-sm text-text-1 line-clamp-3 whitespace-pre-wrap break-words">{result.content}</p>
                  <p class="mt-1 text-xs text-text-3">{formatTimestamp(result.createdAt)}</p>
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
// Members Panel
// ============================================

interface Member {
  user_id: string;
  username?: string;
  display_name?: string;
  avatar_url?: string;
  roles?: string;
  presence?: string;
}

type MemberPresence = Presence;

function normalizePresence(raw: string | null | undefined): MemberPresence {
  const value = String(raw || '').toLowerCase();
  if (value === 'online') return 'online';
  if (value === 'away') return 'away';
  if (value === 'dnd') return 'dnd';
  return 'offline';
}

function resolveMemberPresence(member: Member): MemberPresence {
  const livePresence = presenceStore.getUserPresence(member.user_id)()?.presence;
  return normalizePresence(livePresence || member.presence);
}

function displayMemberName(member: Member): string {
  return member.display_name || member.username || member.user_id;
}

function MembersPanel() {
  const currentChannelId = () => channelStore.currentChannelId();
  const members = createMemo<Member[]>(() => {
    const id = currentChannelId();
    return id ? (channelStore.membersByChannel[id] as unknown as Member[]) || [] : [];
  });
  createEffect(() => {
    const channelId = currentChannelId();
    if (!channelId) return;
    if (members().length === 0) {
      void fetchChannelMembers(channelId);
    }
  });

  const [searchQuery, setSearchQuery] = createSignal('');

  const filteredMembers = createMemo(() => {
    const query = searchQuery().trim().toLowerCase();
    const allMembers = members();
    if (!query) return allMembers;

    return allMembers.filter((member) => {
      const username = (member.username || '').toLowerCase();
      const displayName = (member.display_name || '').toLowerCase();
      const userId = member.user_id.toLowerCase();
      return (
        username.includes(query) ||
        displayName.includes(query) ||
        userId.includes(query)
      );
    });
  });

  const groupedMembers = createMemo(() => {
    const groups: Record<MemberPresence, Member[]> = {
      online: [],
      away: [],
      dnd: [],
      offline: [],
    };

    filteredMembers().forEach((member) => {
      const presence = resolveMemberPresence(member);
      groups[presence].push(member);
    });

    return groups;
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
            onInput={(event) => setSearchQuery(event.currentTarget.value)}
            class="w-full pl-9 pr-3 py-1.5 bg-bg-app border border-border-1 rounded-lg text-sm text-text-1 placeholder:text-text-3 focus:outline-none focus:border-brand"
          />
        </div>
      </div>

      {/* Members List */}
      <div class="flex-1 overflow-y-auto custom-scrollbar p-2">
        <For each={[
          { key: 'online', label: 'Online' },
          { key: 'away', label: 'Away' },
          { key: 'dnd', label: 'Do Not Disturb' },
          { key: 'offline', label: 'Offline' },
        ] as Array<{ key: MemberPresence; label: string }>}
        >
          {(group) => (
            <Show when={groupedMembers()[group.key].length > 0}>
              <div class="mb-3">
                <h3 class="px-2 py-1 text-xs font-semibold text-text-3 uppercase tracking-wider">
                  {group.label} - {groupedMembers()[group.key].length}
                </h3>
                <For each={groupedMembers()[group.key]}>
                  {(member) => (
                    <MemberItem member={member} status={group.key} />
                  )}
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
  status: MemberPresence;
}

function MemberItem(props: MemberItemProps) {
  const initials = () => displayMemberName(props.member).slice(0, 2).toUpperCase();

  return (
    <button
      type="button"
      class="w-full flex items-center gap-2 px-2 py-1.5 rounded-lg hover:bg-bg-surface-2 transition-colors text-left group"
    >
      <div class="relative">
        <Show
          when={props.member.avatar_url}
          fallback={
            <div class="w-8 h-8 rounded-full bg-brand/10 flex items-center justify-center text-brand text-xs font-medium">
              {initials()}
            </div>
          }
        >
          <img
            src={props.member.avatar_url!}
            alt={displayMemberName(props.member)}
            class="w-8 h-8 rounded-full object-cover"
          />
        </Show>
        <span
          class={cn(
            'absolute bottom-0 right-0 w-2.5 h-2.5 rounded-full border-2 border-bg-surface-1',
            props.status === 'online' && 'bg-green-500',
            props.status === 'away' && 'bg-yellow-500',
            props.status === 'dnd' && 'bg-red-500',
            props.status === 'offline' && 'bg-gray-400'
          )}
        />
      </div>
      <div class="flex-1 min-w-0">
        <p class="text-sm text-text-1 truncate">{displayMemberName(props.member)}</p>
        <Show when={props.member.username && props.member.username !== props.member.display_name}>
          <p class="text-xs text-text-3 truncate">@{props.member.username}</p>
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
  createdAt: string;
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  if (Number.isNaN(date.getTime())) return '';
  return date.toLocaleString();
}

function PinnedMessagesPanel() {
  const [isLoading, setIsLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [pinnedMessages, setPinnedMessages] = createSignal<PinnedMessage[]>([]);

  createEffect(() => {
    const channelId = channelStore.currentChannelId();
    if (!channelId) {
      setPinnedMessages([]);
      return;
    }

    let cancelled = false;
    setIsLoading(true);
    setError(null);

    void postsApi
      .getPinned(channelId)
      .then((posts) => {
        if (cancelled) return;
        const mapped = posts.map((post) => ({
          id: post.id,
          author: post.username || post.user_id || 'Unknown',
          content: post.message || '',
          createdAt:
            typeof post.created_at === 'number'
              ? new Date(post.created_at).toISOString()
              : String(post.created_at || ''),
        }));
        setPinnedMessages(mapped);
      })
      .catch((err) => {
        if (cancelled) return;
        setPinnedMessages([]);
        setError(getErrorMessage(err) || 'Failed to load pinned messages.');
      })
      .finally(() => {
        if (!cancelled) {
          setIsLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  });

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
        <Show when={isLoading()}>
          <p class="text-sm text-text-3">Loading pinned messages...</p>
        </Show>
        <Show when={error()}>
          <p class="text-sm text-danger">{error()}</p>
        </Show>
        <Show
          when={!isLoading() && !error() && pinnedMessages().length > 0}
          fallback={
            <Show when={!isLoading() && !error()}>
              <div class="text-center py-8 text-text-3">
                <HiOutlineBookmark size={32} class="mx-auto mb-2 opacity-50" />
                <p class="text-sm">No pinned messages</p>
              </div>
            </Show>
          }
        >
          <div class="space-y-3">
            <For each={pinnedMessages()}>
              {(message) => (
                <div class="p-3 bg-bg-app rounded-lg border border-border-1">
                  <div class="flex items-center gap-2 mb-2">
                    <div class="w-6 h-6 rounded-full bg-brand/10 flex items-center justify-center text-brand text-xs font-medium">
                      {message.author.charAt(0).toUpperCase()}
                    </div>
                    <span class="text-sm font-medium text-text-1 truncate">{message.author}</span>
                  </div>
                  <p class="text-sm text-text-2 whitespace-pre-wrap break-words">{message.content}</p>
                  <p class="mt-2 text-xs text-text-3">{formatTimestamp(message.createdAt)}</p>
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

interface PostFile {
  id: string;
  name: string;
  size: number;
  mime_type?: string;
  url?: string;
}

interface ChannelPost {
  id: string;
  user_id?: string;
  username?: string;
  created_at?: string | number;
  files?: PostFile[];
}

interface FileItem {
  id: string;
  name: string;
  size: number;
  author: string;
  createdAt: string;
  mimeType?: string;
  url?: string;
}

function formatFileSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  let size = bytes;
  let unitIndex = 0;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex += 1;
  }
  return `${size.toFixed(size >= 10 || unitIndex === 0 ? 0 : 1)} ${units[unitIndex]}`;
}

function normalizePostTimestamp(value: string | number | undefined): string {
  if (typeof value === 'number') {
    return new Date(value).toISOString();
  }
  if (typeof value === 'string') {
    return value;
  }
  return new Date().toISOString();
}

function FilesPanel() {
  const [isLoading, setIsLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [files, setFiles] = createSignal<FileItem[]>([]);
  const [openingFileId, setOpeningFileId] = createSignal<string | null>(null);

  createEffect(() => {
    const channelId = channelStore.currentChannelId();
    if (!channelId) {
      setFiles([]);
      return;
    }

    let cancelled = false;
    setIsLoading(true);
    setError(null);

    void client
      .get<{ posts?: ChannelPost[]; messages?: ChannelPost[] }>(`/channels/${channelId}/posts`, {
        params: { limit: 100 },
      })
      .then((response) => {
        if (cancelled) return;

        const payload = response.data;
        const posts = Array.isArray(payload.messages)
          ? payload.messages
          : Array.isArray(payload.posts)
            ? payload.posts
            : [];

        const collected: FileItem[] = [];
        for (const post of posts) {
          const postFiles = Array.isArray(post.files) ? post.files : [];
          for (const file of postFiles) {
            if (!file?.id || !file?.name) {
              continue;
            }
            collected.push({
              id: file.id,
              name: file.name,
              size: typeof file.size === 'number' ? file.size : 0,
              author: post.username || post.user_id || 'Unknown',
              createdAt: normalizePostTimestamp(post.created_at),
              mimeType: file.mime_type,
              url: file.url,
            });
          }
        }

        const deduped = new Map<string, FileItem>();
        collected
          .sort(
            (left, right) =>
              new Date(right.createdAt).getTime() - new Date(left.createdAt).getTime()
          )
          .forEach((file) => {
            if (!deduped.has(file.id)) {
              deduped.set(file.id, file);
            }
          });

        setFiles(Array.from(deduped.values()));
      })
      .catch((err) => {
        if (cancelled) return;
        setFiles([]);
        setError(getErrorMessage(err) || 'Failed to load channel files.');
      })
      .finally(() => {
        if (!cancelled) {
          setIsLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  });

  const openFile = async (file: FileItem) => {
    if (openingFileId()) return;

    setOpeningFileId(file.id);
    try {
      let resolvedUrl = file.url;
      if (!resolvedUrl) {
        const response = await client.get<{ url?: string }>(`/files/${file.id}/download`);
        resolvedUrl = response.data?.url;
      }

      if (!resolvedUrl) {
        throw new Error('File URL is not available');
      }

      window.open(resolvedUrl, '_blank', 'noopener,noreferrer');
    } catch (err) {
      toast.error('Unable to open file', getErrorMessage(err) || 'File could not be opened.');
    } finally {
      setOpeningFileId(null);
    }
  };

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
        <Show when={isLoading()}>
          <p class="px-2 py-2 text-sm text-text-3">Loading files...</p>
        </Show>
        <Show when={error()}>
          <p class="px-2 py-2 text-sm text-danger">{error()}</p>
        </Show>
        <Show
          when={!isLoading() && !error() && files().length > 0}
          fallback={
            <Show when={!isLoading() && !error()}>
              <div class="text-center py-8 text-text-3">
                <HiOutlinePaperClip size={32} class="mx-auto mb-2 opacity-50" />
                <p class="text-sm">No files shared yet</p>
              </div>
            </Show>
          }
        >
          <div class="space-y-1">
            <For each={files()}>
              {(file) => (
                <button
                  type="button"
                  onClick={() => {
                    void openFile(file);
                  }}
                  class="w-full p-2 rounded-lg hover:bg-bg-surface-2 transition-colors text-left"
                  disabled={openingFileId() === file.id}
                >
                  <div class="flex items-center gap-2">
                    <div class="w-8 h-8 rounded-lg bg-brand/10 flex items-center justify-center text-brand">
                      <FileIcon filename={file.name} />
                    </div>
                    <div class="flex-1 min-w-0">
                      <p class="text-sm text-text-1 truncate">{file.name}</p>
                      <p class="text-xs text-text-3 truncate">
                        {formatFileSize(file.size)} - {file.author}
                      </p>
                      <p class="text-xs text-text-3 truncate">{formatTimestamp(file.createdAt)}</p>
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
        return 'PDF';
      case 'png':
      case 'jpg':
      case 'jpeg':
      case 'gif':
      case 'webp':
        return 'IMG';
      case 'doc':
      case 'docx':
        return 'DOC';
      case 'xls':
      case 'xlsx':
        return 'XLS';
      default:
        return 'FILE';
    }
  };

  return <span class="text-[10px] font-semibold">{icon()}</span>;
}

// ============================================
// Channel Info Panel
// ============================================

function ChannelInfoPanel() {
  const navigate = useNavigate();
  const channel = () => channelStore.currentChannel();
  const [isLeaving, setIsLeaving] = createSignal(false);

  const handleLeaveChannel = async () => {
    const current = channel();
    if (!current || isLeaving()) {
      return;
    }

    setIsLeaving(true);
    try {
      await leaveChannel(current.id);
      const nextPath = await resolveDefaultChannelPath();
      uiStore.setRightPanelOpen(false);
      navigate(nextPath || '/');
      toast.success('Left channel', `${current.display_name || current.name} has been left.`);
    } catch (err) {
      toast.error('Unable to leave channel', getErrorMessage(err) || 'Please try again.');
    } finally {
      setIsLeaving(false);
    }
  };

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
                  onClick={() => {
                    navigate('/settings/notifications');
                    uiStore.setRightPanelOpen(false);
                  }}
                >
                  Notification Preferences
                </button>
                <button
                  type="button"
                  class="w-full px-4 py-2 text-danger hover:bg-danger/10 rounded-lg text-sm transition-colors disabled:opacity-60"
                  disabled={isLeaving()}
                  onClick={() => {
                    void handleLeaveChannel();
                  }}
                >
                  {isLeaving() ? 'Leaving...' : 'Leave Channel'}
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
