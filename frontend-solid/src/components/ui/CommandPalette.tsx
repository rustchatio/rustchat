import { For, Show, createEffect, createMemo, createSignal, onCleanup } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { authStore, logout } from '@/stores/auth';
import { channelStore, type Channel } from '@/stores/channels';
import { isAdminRole } from '@/utils/roles';

interface CommandItem {
  id: string;
  label: string;
  subtitle?: string;
  execute: () => void;
}

const MAX_RESULTS = 12;

function toChannelLabel(channel: Channel): string {
  if (channel.channel_type === 'direct' || channel.channel_type === 'group') {
    return `@${channel.display_name || channel.name || 'Direct Message'}`;
  }
  return `#${channel.display_name || channel.name || 'Channel'}`;
}

export default function CommandPalette() {
  const navigate = useNavigate();
  const [isOpen, setIsOpen] = createSignal(false);
  const [query, setQuery] = createSignal('');
  let inputRef: HTMLInputElement | undefined;

  const open = () => {
    setQuery('');
    setIsOpen(true);
    window.setTimeout(() => inputRef?.focus(), 0);
  };

  const close = () => {
    setIsOpen(false);
    setQuery('');
  };

  const baseCommands = createMemo<CommandItem[]>(() => {
    if (!authStore.isAuthenticated) return [];

    const items: CommandItem[] = [
      {
        id: 'playbooks',
        label: 'Playbooks',
        subtitle: '/playbooks',
        execute: () => navigate('/playbooks'),
      },
      {
        id: 'settings-profile',
        label: 'Profile Settings',
        subtitle: '/settings/profile',
        execute: () => navigate('/settings/profile'),
      },
      {
        id: 'settings-notifications',
        label: 'Notification Settings',
        subtitle: '/settings/notifications',
        execute: () => navigate('/settings/notifications'),
      },
      {
        id: 'logout',
        label: 'Sign Out',
        subtitle: 'End your current session',
        execute: () => {
          void logout('manual');
          navigate('/login');
        },
      },
    ];

    if (isAdminRole(authStore.user()?.role)) {
      items.unshift({
        id: 'admin-console',
        label: 'Admin Console',
        subtitle: '/admin',
        execute: () => navigate('/admin'),
      });
    }

    return items;
  });

  const channelCommands = createMemo<CommandItem[]>(() => {
    if (!authStore.isAuthenticated) return [];

    return channelStore.channels.map((channel) => ({
      id: `channel-${channel.id}`,
      label: toChannelLabel(channel),
      subtitle: channel.purpose || channel.header || `/channels/${channel.id}`,
      execute: () => navigate(`/channels/${channel.id}`),
    }));
  });

  const allCommands = createMemo<CommandItem[]>(() => [
    ...baseCommands(),
    ...channelCommands(),
  ]);

  const filteredCommands = createMemo<CommandItem[]>(() => {
    const value = query().trim().toLowerCase();
    if (!value) return allCommands().slice(0, MAX_RESULTS);

    return allCommands()
      .filter((item) => {
        const label = item.label.toLowerCase();
        const subtitle = (item.subtitle || '').toLowerCase();
        return label.includes(value) || subtitle.includes(value);
      })
      .slice(0, MAX_RESULTS);
  });

  createEffect(() => {
    const onOpenEvent = () => open();

    const onKeyDown = (event: KeyboardEvent) => {
      const isMetaK = (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k';
      if (isMetaK) {
        const target = event.target as HTMLElement | null;
        if (target) {
          const isTypingField =
            target.closest('input, textarea, select') !== null ||
            target.getAttribute('contenteditable') === 'true' ||
            target.getAttribute('role') === 'textbox';
          if (isTypingField) {
            return;
          }
        }
        event.preventDefault();
        open();
        return;
      }

      if (event.key === 'Escape' && isOpen()) {
        event.preventDefault();
        close();
      }
    };

    window.addEventListener('rustchat:open-command-palette', onOpenEvent as EventListener);
    window.addEventListener('keydown', onKeyDown);

    onCleanup(() => {
      window.removeEventListener('rustchat:open-command-palette', onOpenEvent as EventListener);
      window.removeEventListener('keydown', onKeyDown);
    });
  });

  return (
    <Show when={isOpen()}>
      <div class="fixed inset-0 z-[120]">
        <button
          type="button"
          aria-label="Close command palette"
          class="absolute inset-0 h-full w-full bg-black/50 backdrop-blur-[1px]"
          onClick={close}
        />
        <div class="absolute left-1/2 top-[10vh] w-[min(680px,92vw)] -translate-x-1/2 rounded-xl border border-border-1 bg-bg-surface-1 shadow-2xl">
          <div class="border-b border-border-1 px-4 py-3">
            <input
              ref={inputRef}
              type="text"
              value={query()}
              onInput={(event) => setQuery(event.currentTarget.value)}
              placeholder="Search channels or run actions..."
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1 outline-none focus:border-brand"
            />
          </div>
          <div class="max-h-[60vh] overflow-y-auto p-2">
            <Show
              when={filteredCommands().length > 0}
              fallback={<p class="px-3 py-4 text-sm text-text-3">No matching commands.</p>}
            >
              <For each={filteredCommands()}>
                {(item) => (
                  <button
                    type="button"
                    class="w-full rounded-lg px-3 py-2 text-left transition-colors hover:bg-bg-surface-2"
                    onClick={() => {
                      close();
                      item.execute();
                    }}
                  >
                    <p class="text-sm font-medium text-text-1">{item.label}</p>
                    <Show when={item.subtitle}>
                      <p class="text-xs text-text-3">{item.subtitle}</p>
                    </Show>
                  </button>
                )}
              </For>
            </Show>
          </div>
          <div class="border-t border-border-1 px-4 py-2 text-xs text-text-3">
            Press <kbd class="rounded bg-bg-app px-1.5 py-0.5">Esc</kbd> to close
          </div>
        </div>
      </div>
    </Show>
  );
}
