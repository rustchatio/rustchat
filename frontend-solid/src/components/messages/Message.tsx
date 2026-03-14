// ============================================
// Message Component
// Single message display with actions
// ============================================

import { createSignal, Show } from 'solid-js';
import { authStore } from '../../stores/auth';
import { cn } from '../../utils/cn';
import { formatMessageTime, formatFullDateTime, getInitials } from '../../utils/date';
import type { Message as MessageType } from '../../types/messages';


// Components
import MessageContent from './MessageContent';
import Reactions from './Reactions';
import MessageActions from './MessageActions';

// Icons
import { HiOutlineChatBubbleLeft } from 'solid-icons/hi';

// ============================================
// Types
// ============================================

interface MessageProps {
  message: MessageType;
  showAvatar?: boolean;
  isCompact?: boolean;
  isHighlighted?: boolean;
  isUnread?: boolean;
  onReply?: (messageId: string) => void;
  onEdit?: (message: MessageType) => void;
  onDelete?: (messageId: string) => void;
  onAddReaction?: (messageId: string, emoji: string) => void;
  onRemoveReaction?: (messageId: string, emoji: string) => void;
  onPin?: (messageId: string) => void;
  onUnpin?: (messageId: string) => void;
  onSave?: (messageId: string) => void;
  onUnsave?: (messageId: string) => void;
  onCopyLink?: (messageId: string) => void;
  onMarkUnread?: (messageId: string) => void;
  onThreadClick?: (messageId: string) => void;
}

// ============================================
// Avatar Component
// ============================================

interface AvatarProps {
  url?: string;
  username: string;
  size?: 'sm' | 'md' | 'lg';
}

function Avatar(props: AvatarProps) {
  const sizeClasses = {
    sm: 'w-8 h-8 text-xs',
    md: 'w-10 h-10 text-sm',
    lg: 'w-12 h-12 text-base',
  };

  const initials = () => getInitials(props.username, 2);

  return (
    <Show
      when={props.url}
      fallback={
        <div
          class={cn(
            'rounded-full bg-brand/10 flex items-center justify-center text-brand font-medium flex-shrink-0',
            sizeClasses[props.size || 'md']
          )}
        >
          {initials()}
        </div>
      }
    >
      <img
        src={props.url}
        alt={props.username}
        class={cn(
          'rounded-full object-cover flex-shrink-0',
          sizeClasses[props.size || 'md']
        )}
      />
    </Show>
  );
}

// ============================================
// Thread Preview Component
// ============================================

interface ThreadPreviewProps {
  count: number;
  lastReplyAt?: string;
  onClick: () => void;
}

function ThreadPreview(props: ThreadPreviewProps) {
  return (
    <button
      type="button"
      onClick={props.onClick}
      class="mt-2 inline-flex items-center gap-2 px-3 py-1.5 rounded-lg bg-bg-surface-2 hover:bg-bg-surface-1 border border-border-1 hover:border-brand/50 transition-colors group"
    >
      <HiOutlineChatBubbleLeft size={14} class="text-brand" />
      <span class="text-sm text-brand font-medium">{props.count} replies</span>
      <Show when={props.lastReplyAt}>
        <span class="text-xs text-text-3">
          Last reply {formatMessageTime(props.lastReplyAt!)}
        </span>
      </Show>
    </button>
  );
}

// ============================================
// Main Message Component
// ============================================

export default function Message(props: MessageProps) {
  const [isEditing, setIsEditing] = createSignal(false);
  const [editContent, setEditContent] = createSignal(props.message.content);

  const currentUserId = () => authStore.user()?.id;
  const isOwnMessage = () => props.message.userId === currentUserId();
  const isCompact = () => props.isCompact ?? false;

  // Handle reaction add
  const handleAddReaction = (emoji: string) => {
    props.onAddReaction?.(props.message.id, emoji);
  };

  // Handle reaction remove
  const handleRemoveReaction = (emoji: string) => {
    props.onRemoveReaction?.(props.message.id, emoji);
  };

  // Handle edit start
  const handleEditStart = () => {
    setEditContent(props.message.content);
    setIsEditing(true);
  };

  // Handle edit save
  const handleEditSave = () => {
    if (editContent().trim() !== props.message.content) {
      props.onEdit?.({ ...props.message, content: editContent().trim() });
    }
    setIsEditing(false);
  };

  // Handle edit cancel
  const handleEditCancel = () => {
    setEditContent(props.message.content);
    setIsEditing(false);
  };

  return (
    <div
      class={cn(
        'group relative flex gap-3 px-4 py-2 transition-colors',
        // Highlighted state (e.g., for search results or pinned)
        props.isHighlighted && 'bg-brand/5 border-l-2 border-brand',
        // Unread separator
        props.isUnread && 'border-t-2 border-brand',
        // Hover background
        !props.isHighlighted && 'hover:bg-bg-surface-2/50',
        // Compact mode
        isCompact() && 'py-1'
      )}
      data-message-id={props.message.id}
      data-user-id={props.message.userId}
    >
      {/* Avatar */}
      <Show when={props.showAvatar !== false}>
        <div class="flex-shrink-0 pt-0.5">
          <Avatar url={props.message.avatarUrl} username={props.message.username} />
        </div>
      </Show>
      <Show when={props.showAvatar === false}>
        <div class="flex-shrink-0 w-10" /> {/* Spacer for alignment */}
      </Show>

      {/* Message Content */}
      <div class="flex-1 min-w-0">
        {/* Header */}
        <Show when={!isCompact()}>
          <div class="flex items-baseline gap-2 mb-0.5">
            <span
              class={cn(
                'font-semibold text-sm',
                isOwnMessage() ? 'text-brand' : 'text-text-1'
              )}
            >
              {props.message.username}
            </span>
            <time
              class="text-xs text-text-3"
              title={formatFullDateTime(props.message.timestamp)}
              dateTime={props.message.timestamp}
            >
              {formatMessageTime(props.message.timestamp)}
            </time>

          </div>
        </Show>

        {/* Message Body */}
        <Show
          when={!isEditing()}
          fallback={
            <div class="mt-1">
              <textarea
                value={editContent()}
                onInput={(e) => setEditContent(e.currentTarget.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && !e.shiftKey) {
                    e.preventDefault();
                    handleEditSave();
                  }
                  if (e.key === 'Escape') {
                    handleEditCancel();
                  }
                }}
                rows={3}
                class="w-full px-3 py-2 bg-bg-app border border-border-1 rounded-lg text-text-1 text-sm focus:outline-none focus:border-brand resize-none"
                autofocus
              />
              <div class="flex items-center gap-2 mt-2">
                <button
                  onClick={handleEditSave}
                  class="px-3 py-1.5 bg-brand text-white text-sm font-medium rounded-lg hover:bg-brand/90 transition-colors"
                >
                  Save
                </button>
                <button
                  onClick={handleEditCancel}
                  class="px-3 py-1.5 text-sm font-medium text-text-2 hover:text-text-1 transition-colors"
                >
                  Cancel
                </button>
                <span class="text-xs text-text-3">press Enter to save, Esc to cancel</span>
              </div>
            </div>
          }
        >
          <MessageContent
            content={props.message.content}
            files={props.message.files}
            editedAt={props.message.editedAt}
          />
        </Show>

        {/* Reactions */}
        <Show when={props.message.reactions && props.message.reactions.length > 0}>
          <Reactions
            reactions={props.message.reactions}
            messageId={props.message.id}
            onAddReaction={handleAddReaction}
            onRemoveReaction={handleRemoveReaction}
          />
        </Show>

        {/* Thread Preview */}
        <Show when={props.message.threadCount && props.message.threadCount > 0}>
          <ThreadPreview
            count={props.message.threadCount!}
            lastReplyAt={props.message.lastReplyAt}
            onClick={() => props.onThreadClick?.(props.message.id)}
          />
        </Show>
      </div>

      {/* Actions Menu */}
      <Show when={!isEditing()}>
        <div class="absolute top-2 right-4 opacity-0 group-hover:opacity-100 transition-opacity">
          <MessageActions
            message={props.message}
            onReact={() => {/* Emoji picker opened by MessageActions */}}
            onReply={() => props.onReply?.(props.message.id)}
            onEdit={handleEditStart}
            onDelete={() => props.onDelete?.(props.message.id)}
            onCopyLink={() => props.onCopyLink?.(props.message.id)}
            onMarkUnread={() => props.onMarkUnread?.(props.message.id)}
            onSave={() => props.onSave?.(props.message.id)}
            onUnsave={() => props.onUnsave?.(props.message.id)}
          />
        </div>
      </Show>

      {/* Unread Separator Label */}
      <Show when={props.isUnread}>
        <div class="absolute -top-3 left-1/2 -translate-x-1/2 px-2 py-0.5 bg-brand text-white text-xs font-medium rounded-full">
          New messages
        </div>
      </Show>
    </div>
  );
}

// ============================================
// Compact Message Variant
// ============================================

export function CompactMessage(props: MessageProps) {
  return <Message {...props} isCompact />;
}

// ============================================
// System Message Variant
// ============================================

interface SystemMessageProps {
  content: string;
  timestamp: string;
}

export function SystemMessage(props: SystemMessageProps) {
  return (
    <div class="flex items-center justify-center gap-2 py-2 text-text-3 text-sm">
      <span class="h-px flex-1 bg-border-1/50" />
      <span>{props.content}</span>
      <time class="text-xs" title={formatFullDateTime(props.timestamp)}>
        {formatMessageTime(props.timestamp)}
      </time>
      <span class="h-px flex-1 bg-border-1/50" />
    </div>
  );
}
