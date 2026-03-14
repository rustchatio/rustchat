import { createSignal, createEffect, For, Show } from 'solid-js';

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
    return () => window.removeEventListener('keydown', handleKeyDown);
  });

  return (
    <div class="slash-commands">
      <Show when={filtered().length > 0} fallback={<div class="no-results">No commands found</div>}>
        <For each={filtered()}>
          {(command, index) => (
            <button
              class="command-item"
              classList={{ selected: index() === selectedIndex() }}
              onClick={() => props.onSelect(command.name)}
              onMouseEnter={() => setSelectedIndex(index())}
            >
              <div class="command-name">/{command.name}</div>
              <div class="command-description">
                {command.description}
                {command.shortcut && <span class="command-shortcut">{command.shortcut}</span>}
              </div>
            </button>
          )}
        </For>
      </Show>
    </div>
  );
}
