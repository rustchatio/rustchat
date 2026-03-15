// ============================================
// Message Reactions Component
// Display and toggle message reactions
// ============================================

import { createSignal, Show, For, createMemo } from 'solid-js';
import { authStore } from '../../stores/auth';
import type { Reaction } from '../../types/messages';
import { cn } from '../../utils/cn';

// Icons
import { HiOutlinePlus } from 'solid-icons/hi';

// ============================================
// Types
// ============================================

interface ReactionsProps {
  reactions: Reaction[];
  messageId: string;
  onAddReaction?: (emoji: string) => void;
  onRemoveReaction?: (emoji: string) => void;
}

interface ReactionTooltipProps {
  users: string[];
  emoji: string;
  currentUserReacted: boolean;
}

// ============================================
// Common Emoji Map
// ============================================

const COMMON_EMOJIS = [
  '👍', '❤️', '😂', '🎉', '😮', '😢', '😡', '👏',
  '🔥', '❤️‍🔥', '🚀', '👀', '😍', '🤔', '👎', '🙏',
];

// ============================================
// Reaction Tooltip Component
// ============================================

function ReactionTooltip(props: ReactionTooltipProps) {
  const text = createMemo(() => {
    const maxShown = 3;
    const shown = props.users.slice(0, maxShown);
    const remaining = props.users.length - maxShown;
    
    if (props.currentUserReacted) {
      if (props.users.length === 1) {
        return `You reacted with ${props.emoji}`;
      }
      const others = props.users.filter(u => u !== authStore.user()?.id);
      if (others.length === 0) {
        return `You reacted with ${props.emoji}`;
      }
      if (others.length === 1) {
        return `You and 1 other reacted with ${props.emoji}`;
      }
      return `You and ${others.length} others reacted with ${props.emoji}`;
    }
    
    if (props.users.length === 1) {
      return `1 person reacted with ${props.emoji}`;
    }
    if (remaining > 0) {
      return `${shown.join(', ')} and ${remaining} others reacted with ${props.emoji}`;
    }
    return `${shown.join(', ')} reacted with ${props.emoji}`;
  });

  return (
    <div class="absolute bottom-full left-1/2 z-[9999] mb-1 -translate-x-1/2 whitespace-nowrap rounded border border-border-1 bg-bg-surface-1 px-2 py-1 text-xs text-text-1 opacity-0 shadow-lg transition-opacity pointer-events-none group-hover:opacity-100">
      {text()}
      <div class="absolute top-full left-1/2 -translate-x-1/2 -mt-1 border-4 border-transparent border-t-bg-surface-1" />
    </div>
  );
}

// ============================================
// Emoji Picker Component
// ============================================

interface EmojiPickerProps {
  onSelect: (emoji: string) => void;
  onClose: () => void;
}

function EmojiPicker(props: EmojiPickerProps) {
  return (
    <div class="absolute bottom-full left-0 z-[9999] mb-1 rounded-lg border border-border-1 bg-bg-surface-1 p-2 shadow-lg">
      <div class="grid grid-cols-8 gap-1">
        <For each={COMMON_EMOJIS}>
          {(emoji) => (
            <button
              type="button"
              onClick={() => props.onSelect(emoji)}
              class="w-8 h-8 flex items-center justify-center text-lg hover:bg-bg-surface-2 rounded transition-colors"
              title={`React with ${emoji}`}
            >
              {emoji}
            </button>
          )}
        </For>
      </div>
    </div>
  );
}

// ============================================
// Main Reactions Component
// ============================================

export default function Reactions(props: ReactionsProps) {
  const [showPicker, setShowPicker] = createSignal(false);
  const currentUserId = () => authStore.user()?.id;

  // Check if current user has reacted with this emoji
  const hasReacted = (reaction: Reaction) => {
    const userId = currentUserId();
    return userId ? reaction.users.includes(userId) : false;
  };

  // Handle reaction click (toggle)
  const handleReactionClick = (reaction: Reaction) => {
    if (hasReacted(reaction)) {
      props.onRemoveReaction?.(reaction.emoji);
    } else {
      props.onAddReaction?.(reaction.emoji);
    }
  };

  // Handle adding new reaction from picker
  const handleAddNewReaction = (emoji: string) => {
    props.onAddReaction?.(emoji);
    setShowPicker(false);
  };

  return (
    <div class="flex flex-wrap items-center gap-1 mt-1">
      {/* Existing Reactions */}
      <For each={props.reactions}>
        {(reaction) => {
          const userReacted = hasReacted(reaction);
          return (
            <div class="relative group">
              <ReactionTooltip
                users={reaction.users}
                emoji={reaction.emoji}
                currentUserReacted={userReacted}
              />
              <button
                type="button"
                onClick={() => handleReactionClick(reaction)}
                class={cn(
                  'inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-sm transition-all',
                  'border hover:scale-105 active:scale-95',
                  userReacted
                    ? 'bg-brand/10 border-brand text-brand'
                    : 'bg-bg-surface-2 border-border-1 text-text-1 hover:border-brand/50'
                )}
                title={userReacted ? 'Click to remove reaction' : 'Click to add reaction'}
              >
                <span>{reaction.emoji}</span>
                <span class={cn(
                  'text-xs font-medium',
                  userReacted ? 'text-brand' : 'text-text-2'
                )}>
                  {reaction.count}
                </span>
              </button>
            </div>
          );
        }}
      </For>

      {/* Add Reaction Button */}
      <div class="relative">
        <Show when={showPicker()}>
          <EmojiPicker
            onSelect={handleAddNewReaction}
            onClose={() => setShowPicker(false)}
          />
        </Show>
        <button
          type="button"
          onClick={() => setShowPicker(!showPicker())}
          class={cn(
            'inline-flex items-center justify-center w-7 h-7 rounded-full',
            'text-text-3 hover:text-text-1 hover:bg-bg-surface-2',
            'transition-colors',
            showPicker() && 'bg-bg-surface-2 text-text-1'
          )}
          title="Add reaction"
          aria-label="Add reaction"
          aria-expanded={showPicker()}
        >
          <HiOutlinePlus size={14} />
        </button>
      </div>
    </div>
  );
}

// ============================================
// Compact Reactions Display (for read-only views)
// ============================================

export function ReactionsCompact(props: { reactions: Reaction[] }) {
  return (
    <Show when={props.reactions && props.reactions.length > 0}>
      <div class="flex flex-wrap items-center gap-1">
        <For each={props.reactions}>
          {(reaction) => (
            <span class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded bg-bg-surface-2 text-xs">
              <span>{reaction.emoji}</span>
              <span class="text-text-3">{reaction.count}</span>
            </span>
          )}
        </For>
      </div>
    </Show>
  );
}
