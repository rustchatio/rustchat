import { createSignal, createEffect, For, Show, onCleanup } from 'solid-js';
import { channelStore } from '@/stores/channels';

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
  const [users, setUsers] = createSignal<User[]>([]);
  const [selectedIndex, setSelectedIndex] = createSignal(0);
  const channels = channelStore;

  createEffect(() => {
    // Get channel members and filter by query
    const channel = channels.channels.find(c => c.id === props.channelId);
    if (!channel) return;
    console.log('Channel:', channel);

    // Mock users - in real app would fetch from API
    const mockUsers: User[] = [
      { id: '1', username: 'alice', display_name: 'Alice Smith' },
      { id: '2', username: 'bob', display_name: 'Bob Johnson' },
      { id: '3', username: 'charlie', display_name: 'Charlie Brown' },
      { id: '4', username: 'diana', display_name: 'Diana Prince' },
      { id: '5', username: 'eve', display_name: 'Eve Davis' },
    ];

    const filtered = mockUsers.filter(
      u => 
        u.username.toLowerCase().includes(props.query.toLowerCase()) ||
        u.display_name.toLowerCase().includes(props.query.toLowerCase())
    );
    
    setUsers(filtered.slice(0, 5));
    setSelectedIndex(0);
  });

  const handleKeyDown = (e: KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(i => (i + 1) % users().length);
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(i => (i - 1 + users().length) % users().length);
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
        fallback={<div class="px-3 py-2 text-sm text-text-3">No users found</div>}
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
