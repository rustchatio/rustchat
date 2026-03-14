// ============================================
// Typing Indicator Component
// ============================================

import { For, Show, createMemo } from 'solid-js';
import { presenceStore } from '../stores/presence';

interface TypingIndicatorProps {
  channelId: string;
  threadRootId?: string;
  class?: string;
}

export function TypingIndicator(props: TypingIndicatorProps) {

  const typingUsers = createMemo(() => {
    return presenceStore.getTypingUsersForChannel(props.channelId, props.threadRootId)();
  });

  const typingText = createMemo(() => {
    const users = typingUsers();
    if (users.length === 0) return null;

    const names = users.map((u) => u.username);

    if (names.length === 1) {
      return `${names[0]} is typing...`;
    } else if (names.length === 2) {
      return `${names[0]} and ${names[1]} are typing...`;
    } else if (names.length === 3) {
      return `${names[0]}, ${names[1]}, and ${names[2]} are typing...`;
    } else {
      return `${names.length} people are typing...`;
    }
  });

  return (
    <Show when={typingUsers().length > 0}>
      <div
        class={`flex items-center gap-2 text-sm text-base-content/60 ${props.class || ''}`}
        role="status"
        aria-live="polite"
        aria-atomic="true"
      >
        {/* Animated dots */}
        <div class="flex gap-0.5" aria-hidden="true">
          <span
            class="w-1.5 h-1.5 rounded-full bg-primary animate-bounce"
            style={{ 'animation-delay': '0ms' }}
          />
          <span
            class="w-1.5 h-1.5 rounded-full bg-primary animate-bounce"
            style={{ 'animation-delay': '150ms' }}
          />
          <span
            class="w-1.5 h-1.5 rounded-full bg-primary animate-bounce"
            style={{ 'animation-delay': '300ms' }}
          />
        </div>

        {/* Typing text */}
        <span class="truncate">{typingText()}</span>
      </div>
    </Show>
  );
}

// Compact version for channel list
export function TypingIndicatorCompact(props: TypingIndicatorProps) {
  const typingUsers = createMemo(() => {
    return presenceStore.getTypingUsersForChannel(props.channelId, props.threadRootId)();
  });

  return (
    <Show when={typingUsers().length > 0}>
      <div
        class={`flex items-center gap-1 ${props.class || ''}`}
        aria-label={`${typingUsers().length} user(s) typing`}
      >
        <div class="flex gap-px">
          <span class="w-1 h-1 rounded-full bg-primary animate-pulse" />
          <span class="w-1 h-1 rounded-full bg-primary animate-pulse" style={{ 'animation-delay': '150ms' }} />
          <span class="w-1 h-1 rounded-full bg-primary animate-pulse" style={{ 'animation-delay': '300ms' }} />
        </div>
      </div>
    </Show>
  );
}

// Avatar stack for typing users
export function TypingAvatars(props: TypingIndicatorProps) {
  const typingUsers = createMemo(() => {
    return presenceStore.getTypingUsersForChannel(props.channelId, props.threadRootId)();
  });

  return (
    <Show when={typingUsers().length > 0}>
      <div class={`flex items-center gap-1 ${props.class || ''}`}>
        <div class="flex -space-x-2">
          <For each={typingUsers().slice(0, 3)}>
            {(user) => (
              <div class="w-6 h-6 rounded-full bg-primary/20 border-2 border-base-100 flex items-center justify-center text-xs font-medium text-primary">
                {user.username.charAt(0).toUpperCase()}
              </div>
            )}
          </For>
        </div>
        <Show when={typingUsers().length > 3}>
          <span class="text-xs text-base-content/50">+{typingUsers().length - 3}</span>
        </Show>
      </div>
    </Show>
  );
}

export default TypingIndicator;
