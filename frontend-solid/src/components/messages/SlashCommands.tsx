import { createSignal, createEffect, For, Show, onCleanup } from 'solid-js';

interface Command {
  name: string;
  description: string;
  shortcut?: string;
}

interface SlashCommandsProps {
  query: string;
  onSelect: (command: string) => void;
  onClose: () => void;
}

const COMMANDS: Command[] = [
  { name: 'shrug', description: 'Shrug shoulders', shortcut: '¯\\_(ツ)_/¯' },
  { name: 'tableflip', description: 'Flip the table', shortcut: '(╯°□°)╯︵ ┻━┻' },
  { name: 'unflip', description: 'Put the table back', shortcut: '┬─┬ ノ( ゜-゜ノ)' },
  { name: 'away', description: 'Set status to away' },
  { name: 'dnd', description: 'Set status to do not disturb' },
  { name: 'online', description: 'Set status to online' },
  { name: 'offline', description: 'Set status to offline' },
];

export function SlashCommands(props: SlashCommandsProps) {
  const [filtered, setFiltered] = createSignal<Command[]>(COMMANDS);
  const [selectedIndex, setSelectedIndex] = createSignal(0);

  createEffect(() => {
    const query = props.query.toLowerCase();
    if (!query) {
      setFiltered(COMMANDS);
    } else {
      setFiltered(
        COMMANDS.filter(c => 
          c.name.toLowerCase().includes(query) ||
          c.description.toLowerCase().includes(query)
        )
      );
    }
    setSelectedIndex(0);
  });

  const handleKeyDown = (e: KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(i => (i + 1) % filtered().length);
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(i => (i - 1 + filtered().length) % filtered().length);
        break;
      case 'Enter':
        e.preventDefault();
        const selected = filtered()[selectedIndex()];
        if (selected) {
          props.onSelect(selected.name);
        }
        break;
      case 'Escape':
        props.onClose();
        break;
    }
  };

  createEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    onCleanup(() => window.removeEventListener('keydown', handleKeyDown));
  });

  return (
    <div class="absolute bottom-full left-3 right-3 z-40 mb-2 overflow-hidden rounded-xl border border-border-1 bg-bg-surface-1 shadow-xl sm:left-0 sm:right-auto sm:w-[420px]">
      <Show
        when={filtered().length > 0}
        fallback={<div class="px-3 py-2 text-sm text-text-3">No commands found</div>}
      >
        <For each={filtered()}>
          {(command, index) => (
            <button
              class={`w-full px-3 py-2 text-left transition-colors ${
                index() === selectedIndex()
                  ? 'bg-brand/10'
                  : 'hover:bg-bg-surface-2'
              }`}
              onClick={() => props.onSelect(command.name)}
              onMouseEnter={() => setSelectedIndex(index())}
              type="button"
            >
              <div class="text-sm font-medium text-text-1">/{command.name}</div>
              <div class="text-xs text-text-3">
                {command.description}
                {command.shortcut && <span class="ml-2 text-text-2">{command.shortcut}</span>}
              </div>
            </button>
          )}
        </For>
      </Show>
    </div>
  );
}
