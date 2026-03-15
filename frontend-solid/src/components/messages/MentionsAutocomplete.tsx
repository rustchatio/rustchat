import { createSignal, createEffect, For, Show, onCleanup, createMemo } from 'solid-js';
import { channelStore, fetchChannelMembers } from '@/stores/channels';

interface User {
  id: string;
  username: string;
  display_name: string;
  avatar_url?: string;
}

interface MentionsAutocompleteProps {
  query: string;
  channelId: string;
  onSelect: (username: string) => void;
  onClose: () => void;
}

export function MentionsAutocomplete(props: MentionsAutocompleteProps) {
  const [selectedIndex, setSelectedIndex] = createSignal(0);

  createEffect(() => {
    const channelId = props.channelId;
    if (!channelId) return;
    const members = channelStore.membersByChannel[channelId] || [];
    if (members.length === 0) {
      void fetchChannelMembers(channelId);
    }
  });

  const users = createMemo<User[]>(() => {
    const members = channelStore.membersByChannel[props.channelId] || [];
    const query = props.query.trim().toLowerCase();

    return members
      .map((member) => ({
        id: member.user_id,
        username: member.username || member.display_name || member.user_id,
        display_name: member.display_name || member.username || member.user_id,
        avatar_url: member.avatar_url || undefined,
      }))
      .filter((user) => {
        if (!query) return true;
        return (
          user.username.toLowerCase().includes(query) ||
          user.display_name.toLowerCase().includes(query)
        );
      })
      .slice(0, 8);
  });

  createEffect(() => {
    users();
    setSelectedIndex(0);
  });

  const handleKeyDown = (e: KeyboardEvent) => {
    const list = users();
    if (list.length === 0) return;
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex((i) => (i + 1) % list.length);
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex((i) => (i - 1 + list.length) % list.length);
        break;
      case 'Enter':
        e.preventDefault();
        const selected = users()[selectedIndex()];
        if (selected) {
          props.onSelect(selected.username);
        }
        break;
      case 'Escape':
        props.onClose();
        break;
    }
  };

  // Add keyboard listener
  createEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    onCleanup(() => window.removeEventListener('keydown', handleKeyDown));
  });

  return (
    <div class="absolute bottom-full left-3 right-3 z-40 mb-2 overflow-hidden rounded-xl border border-border-1 bg-bg-surface-1 shadow-xl sm:left-0 sm:right-auto sm:w-[420px]">
      <Show
        when={users().length > 0}
        fallback={<div class="px-3 py-2 text-sm text-text-3">No matching members found</div>}
      >
        <For each={users()}>
          {(user, index) => (
            <button
              class={`flex w-full items-center gap-3 px-3 py-2 text-left transition-colors ${
                index() === selectedIndex()
                  ? 'bg-brand/10'
                  : 'hover:bg-bg-surface-2'
              }`}
              onClick={() => props.onSelect(user.username)}
              onMouseEnter={() => setSelectedIndex(index())}
              type="button"
            >
              <div class="inline-flex h-8 w-8 items-center justify-center rounded-full bg-brand/15 text-sm font-semibold text-brand">
                {user.display_name.charAt(0).toUpperCase()}
              </div>
              <div class="min-w-0">
                <div class="truncate text-sm font-medium text-text-1">{user.display_name}</div>
                <div class="truncate text-xs text-text-3">@{user.username}</div>
              </div>
            </button>
          )}
        </For>
      </Show>
    </div>
  );
}
