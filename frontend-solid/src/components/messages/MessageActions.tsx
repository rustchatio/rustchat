// ============================================
// Message Actions Component
// Hover actions menu for messages
// ============================================

import { createSignal, Show, For, onMount, onCleanup } from 'solid-js';
import { authStore } from '../../stores/auth';

import { cn } from '../../utils/cn';
import type { Message } from '../../types/messages';

// Icons
import {
  HiOutlineFaceSmile,
  HiOutlineChatBubbleLeft,
  HiOutlinePencil,
  HiOutlineTrash,
  HiOutlineLink,
  HiOutlineEnvelope,
  HiOutlineCheck,
  HiOutlineEllipsisHorizontal,
  HiOutlineBookmark,
  HiOutlineFlag,
  HiOutlineExclamationCircle,
} from 'solid-icons/hi';

// ============================================
// Types
// ============================================

interface MessageActionsProps {
  message: Message;
  onReact?: () => void;
  onQuickReact?: (emoji: string) => void;
  onReply?: () => void;
  onEdit?: () => void;
  onDelete?: () => void;
  onCopyLink?: () => void;
  onMarkUnread?: () => void;
  onSave?: () => void;
  onUnsave?: () => void;
  onReport?: () => void;
}

type ActionItem = {
  id: string;
  label: string;
  icon: typeof HiOutlineFaceSmile;
  onClick: () => void;
  show: () => boolean;
  variant?: 'default' | 'danger';
  shortcut?: string;
};

// ============================================
// Common Emojis for Quick React
// ============================================

const QUICK_REACTIONS = ['👍', '❤️', '😂', '🎉', '😮'];

// ============================================
// Delete Confirmation Modal
// ============================================

interface DeleteConfirmModalProps {
  onConfirm: () => void;
  onCancel: () => void;
}

function DeleteConfirmModal(props: DeleteConfirmModalProps) {
  return (
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div class="bg-bg-surface-1 rounded-lg shadow-xl border border-border-1 max-w-sm w-full p-4">
        <div class="flex items-center gap-3 mb-3">
          <div class="w-10 h-10 rounded-full bg-danger/10 flex items-center justify-center">
            <HiOutlineExclamationCircle size={20} class="text-danger" />
          </div>
          <div>
            <h3 class="text-lg font-semibold text-text-1">Delete Message</h3>
            <p class="text-sm text-text-2">Are you sure? This cannot be undone.</p>
          </div>
        </div>
        <div class="flex justify-end gap-2 mt-4">
          <button
            onClick={props.onCancel}
            class="px-4 py-2 text-sm font-medium text-text-2 hover:text-text-1 hover:bg-bg-surface-2 rounded-lg transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={props.onConfirm}
            class="px-4 py-2 text-sm font-medium text-white bg-danger hover:bg-danger/90 rounded-lg transition-colors"
          >
            Delete
          </button>
        </div>
      </div>
    </div>
  );
}

// ============================================
// More Actions Menu
// ============================================

interface MoreActionsMenuProps {
  actions: ActionItem[];
  onClose: () => void;
}

function MoreActionsMenu(props: MoreActionsMenuProps) {
  let menuRef: HTMLDivElement | undefined;

  // Close on click outside
  onMount(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef && !menuRef.contains(e.target as Node)) {
        props.onClose();
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    onCleanup(() => document.removeEventListener('mousedown', handleClickOutside));
  });

  // Close on escape
  onMount(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        props.onClose();
      }
    };

    document.addEventListener('keydown', handleEscape);
    onCleanup(() => document.removeEventListener('keydown', handleEscape));
  });

  return (
    <div
      ref={menuRef}
      class="absolute right-0 top-full mt-1 min-w-[180px] bg-bg-surface-1 rounded-lg shadow-xl border border-border-1 py-1 z-50"
      role="menu"
    >
      <For each={props.actions.filter(a => a.show())}>
        {(action) => {
          const Icon = action.icon;
          return (
            <button
              type="button"
              onClick={() => {
                action.onClick();
                props.onClose();
              }}
              class={cn(
                'w-full flex items-center gap-3 px-3 py-2 text-sm text-left transition-colors',
                action.variant === 'danger'
                  ? 'text-danger hover:bg-danger/10'
                  : 'text-text-1 hover:bg-bg-surface-2'
              )}
              role="menuitem"
            >
              <Icon size={16} />
              <span class="flex-1">{action.label}</span>
              {action.shortcut && (
                <span class="text-xs text-text-3">{action.shortcut}</span>
              )}
            </button>
          );
        }}
      </For>
    </div>
  );
}

// ============================================
// Main Message Actions Component
// ============================================

export default function MessageActions(props: MessageActionsProps) {
  const [showMoreMenu, setShowMoreMenu] = createSignal(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = createSignal(false);
  const [copied, setCopied] = createSignal(false);
  const [showEmojiPicker, setShowEmojiPicker] = createSignal(false);
  let quickReactionRef: HTMLDivElement | undefined;

  const currentUserId = () => authStore.user()?.id;
  const isOwnMessage = () => props.message.userId === currentUserId();


  // Handle quick reaction
  const handleQuickReaction = (emoji: string) => {
    props.onQuickReact?.(emoji);
    if (!props.onQuickReact) {
      props.onReact?.();
    }
    setShowEmojiPicker(false);
  };

  // Handle copy link
  const handleCopyLink = async () => {
    const url = `${window.location.origin}/channels/${props.message.channelId}/${props.message.id}`;
    try {
      await navigator.clipboard.writeText(url);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Fallback
      const textarea = document.createElement('textarea');
      textarea.value = url;
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand('copy');
      document.body.removeChild(textarea);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
    props.onCopyLink?.();
  };

  // Handle delete
  const handleDelete = () => {
    setShowDeleteConfirm(false);
    props.onDelete?.();
  };

  onMount(() => {
    const handlePointerDown = (event: MouseEvent) => {
      if (!showEmojiPicker()) return;
      if (!quickReactionRef) return;
      if (quickReactionRef.contains(event.target as Node)) return;
      setShowEmojiPicker(false);
    };

    document.addEventListener('mousedown', handlePointerDown);
    onCleanup(() => {
      document.removeEventListener('mousedown', handlePointerDown);
    });
  });

  // More actions menu items
  const moreActions: ActionItem[] = [
    {
      id: 'save',
      label: props.message.isSaved ? 'Unsave' : 'Save',
      icon: HiOutlineBookmark,
      onClick: () => props.message.isSaved ? props.onUnsave?.() : props.onSave?.(),
      show: () => true,
    },
    {
      id: 'copy-link',
      label: copied() ? 'Copied!' : 'Copy Link',
      icon: copied() ? HiOutlineCheck : HiOutlineLink,
      onClick: handleCopyLink,
      show: () => true,
    },
    {
      id: 'mark-unread',
      label: 'Mark Unread',
      icon: HiOutlineEnvelope,
      onClick: () => props.onMarkUnread?.(),
      show: () => true,
    },
    {
      id: 'edit',
      label: 'Edit',
      icon: HiOutlinePencil,
      onClick: () => props.onEdit?.(),
      show: () => isOwnMessage(),
      shortcut: 'E',
    },
    {
      id: 'delete',
      label: 'Delete',
      icon: HiOutlineTrash,
      onClick: () => setShowDeleteConfirm(true),
      show: () => isOwnMessage(),
      variant: 'danger',
    },
    {
      id: 'report',
      label: 'Report',
      icon: HiOutlineFlag,
      onClick: () => props.onReport?.(),
      show: () => !isOwnMessage(),
      variant: 'danger',
    },
  ];

  return (
    <>
      {/* Delete Confirmation Modal */}
      <Show when={showDeleteConfirm()}>
        <DeleteConfirmModal
          onConfirm={handleDelete}
          onCancel={() => setShowDeleteConfirm(false)}
        />
      </Show>

      {/* Actions Bar */}
      <div class="flex items-center gap-0.5 bg-bg-surface-1 rounded-lg shadow-sm border border-border-1 opacity-0 group-hover:opacity-100 focus-within:opacity-100 transition-opacity">
        {/* Quick Reactions */}
        <div class="relative" ref={quickReactionRef}>
          <Show when={showEmojiPicker()}>
            <div class="absolute bottom-full left-0 mb-1 p-2 bg-bg-surface-1 rounded-lg shadow-lg border border-border-1 z-[10000]">
              <div class="flex gap-1">
                <For each={QUICK_REACTIONS}>
                  {(emoji) => (
                    <button
                      type="button"
                      onClick={() => handleQuickReaction(emoji)}
                      class="relative inline-flex h-8 w-8 items-center justify-center rounded text-xl leading-none hover:bg-bg-surface-2 transition-colors"
                      style={{
                        'font-family': '"Apple Color Emoji", "Segoe UI Emoji", "Noto Color Emoji", sans-serif',
                        'line-height': '1',
                      }}
                    >
                      {emoji}
                    </button>
                  )}
                </For>
              </div>
            </div>
          </Show>
          <button
            type="button"
            onClick={() => {
              if (props.onQuickReact) {
                setShowEmojiPicker(!showEmojiPicker());
              } else {
                props.onReact?.();
              }
            }}
            class="p-1.5 text-text-3 hover:text-text-1 hover:bg-bg-surface-2 rounded-md transition-colors"
            title="Add reaction"
            aria-label="Add reaction"
          >
            <HiOutlineFaceSmile size={18} />
          </button>
        </div>

        {/* Reply */}
        <button
          type="button"
          onClick={() => props.onReply?.()}
          class="p-1.5 text-text-3 hover:text-text-1 hover:bg-bg-surface-2 rounded-md transition-colors"
          title="Reply in thread"
          aria-label="Reply in thread"
        >
          <HiOutlineChatBubbleLeft size={18} />
        </button>

        {/* Edit (only for own messages) */}
        <Show when={isOwnMessage()}>
          <button
            type="button"
            onClick={() => props.onEdit?.()}
            class="p-1.5 text-text-3 hover:text-text-1 hover:bg-bg-surface-2 rounded-md transition-colors"
            title="Edit"
            aria-label="Edit message"
          >
            <HiOutlinePencil size={18} />
          </button>
        </Show>

        {/* More Actions */}
        <div class="relative">
          <Show when={showMoreMenu()}>
            <MoreActionsMenu actions={moreActions} onClose={() => setShowMoreMenu(false)} />
          </Show>
          <button
            type="button"
            onClick={() => setShowMoreMenu(!showMoreMenu())}
            class="p-1.5 text-text-3 hover:text-text-1 hover:bg-bg-surface-2 rounded-md transition-colors"
            title="More actions"
            aria-label="More actions"
            aria-expanded={showMoreMenu()}
            aria-haspopup="menu"
          >
            <HiOutlineEllipsisHorizontal size={18} />
          </button>
        </div>
      </div>
    </>
  );
}
