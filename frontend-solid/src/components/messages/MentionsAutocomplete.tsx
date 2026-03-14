import { createSignal, createEffect, For, Show } from 'solid-js';
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
    return () => window.removeEventListener('keydown', handleKeyDown);
  });

  return (
    <div class="mentions-autocomplete">
      <Show when={users().length > 0} fallback={<div class="no-results">No users found</div>}>
        <For each={users()}>
          {(user, index) => (
            <button
              class="mention-item"
              classList={{ selected: index() === selectedIndex() }}
              onClick={() => props.onSelect(user.username)}
              onMouseEnter={() => setSelectedIndex(index())}
            >
              <div class="mention-avatar">
                {user.display_name.charAt(0).toUpperCase()}
              </div>
              <div class="mention-info">
                <div class="mention-name">{user.display_name}</div>
                <div class="mention-username">@{user.username}</div>
              </div>
            </button>
          )}
        </For>
      </Show>
    </div>
  );
}
