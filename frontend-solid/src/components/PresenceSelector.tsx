// ============================================
// Presence Selector Component
// ============================================

import { createSignal, Show, For } from 'solid-js';
import { presenceStore, type Presence } from '../stores/presence';
import { useWebSocket } from '../realtime';

interface PresenceOption {
  value: Presence;
  label: string;
  icon: string;
  color: string;
  description: string;
}

const PRESENCE_OPTIONS: PresenceOption[] = [
  {
    value: 'online',
    label: 'Online',
    icon: '●',
    color: 'text-success',
    description: 'Available for messages',
  },
  {
    value: 'away',
    label: 'Away',
    icon: '◐',
    color: 'text-warning',
    description: 'Stepped away',
  },
  {
    value: 'dnd',
    label: 'Do Not Disturb',
    icon: '⊘',
    color: 'text-error',
    description: 'Notifications muted',
  },
  {
    value: 'offline',
    label: 'Offline',
    icon: '○',
    color: 'text-base-content/40',
    description: 'Appear offline',
  },
];

interface PresenceSelectorProps {
  class?: string;
  size?: 'sm' | 'md' | 'lg';
  showLabel?: boolean;
}

export function PresenceSelector(props: PresenceSelectorProps) {
  const ws = useWebSocket();
  const [isOpen, setIsOpen] = createSignal(false);

  const sizeClasses = {
    sm: 'w-4 h-4 text-xs',
    md: 'w-5 h-5 text-sm',
    lg: 'w-6 h-6 text-base',
  };

  const currentPresence = () => presenceStore.selfPresence()?.presence || 'online';

  const currentOption = () =>
    PRESENCE_OPTIONS.find((o) => o.value === currentPresence()) || PRESENCE_OPTIONS[0];

  const handleSelect = (presence: Presence) => {
    // Update local state immediately for responsiveness
    presenceStore.updateSelfPresence(presence);

    // Send to server
    ws.sendPresence(presence);

    setIsOpen(false);
  };

  return (
    <div class={`relative ${props.class || ''}`}>
      {/* Trigger button */}
      <button
        type="button"
        onClick={() => setIsOpen(!isOpen())}
        class="flex items-center gap-2 px-2 py-1.5 rounded-lg hover:bg-base-200/50 transition-colors"
        aria-haspopup="listbox"
        aria-expanded={isOpen()}
        aria-label={`Current status: ${currentOption().label}. Click to change`}
      >
        <span class={`${sizeClasses[props.size || 'md']} ${currentOption().color}`} aria-hidden="true">
          {currentOption().icon}
        </span>
        <Show when={props.showLabel !== false}>
          <span class="text-sm font-medium">{currentOption().label}</span>
        </Show>
        <svg
          class={`w-4 h-4 text-base-content/50 transition-transform ${isOpen() ? 'rotate-180' : ''}`}
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          aria-hidden="true"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      {/* Dropdown */}
      <Show when={isOpen()}>
        {/* Backdrop to close on click outside */}
        <div class="fixed inset-0 z-40" onClick={() => setIsOpen(false)} aria-hidden="true" />

        <div
          class="absolute right-0 mt-1 w-56 rounded-xl bg-base-100 shadow-xl border border-base-300 z-50 py-1"
          role="listbox"
          aria-label="Select your status"
        >
          <For each={PRESENCE_OPTIONS}>
            {(option) => (
              <button
                type="button"
                role="option"
                aria-selected={currentPresence() === option.value}
                onClick={() => handleSelect(option.value)}
                class={`w-full flex items-center gap-3 px-3 py-2.5 hover:bg-base-200/50 transition-colors text-left
                  ${currentPresence() === option.value ? 'bg-primary/5' : ''}`}
              >
                <span class={`${sizeClasses[props.size || 'md']} ${option.color}`}>{option.icon}</span>
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium">{option.label}</div>
                  <div class="text-xs text-base-content/50">{option.description}</div>
                </div>
                <Show when={currentPresence() === option.value}>
                  <svg
                    class="w-4 h-4 text-primary flex-shrink-0"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    aria-hidden="true"
                  >
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M5 13l4 4L19 7" />
                  </svg>
                </Show>
              </button>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
}

// Simple badge version for compact displays
export function PresenceBadge(props: { presence: Presence; class?: string; size?: 'sm' | 'md' | 'lg' }) {
  const sizeClasses = {
    sm: 'w-2 h-2',
    md: 'w-2.5 h-2.5',
    lg: 'w-3 h-3',
  };

  const colorClasses: Record<Presence, string> = {
    online: 'bg-success',
    away: 'bg-warning',
    dnd: 'bg-error',
    offline: 'bg-base-content/30',
  };

  return (
    <span
      class={`inline-block rounded-full ${sizeClasses[props.size || 'md']} ${colorClasses[props.presence]} ${props.class || ''}`}
      aria-label={`Status: ${props.presence}`}
      title={props.presence}
    />
  );
}

export default PresenceSelector;
