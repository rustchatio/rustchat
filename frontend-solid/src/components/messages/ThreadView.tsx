// ============================================
// Thread View Component
// Display and reply to thread messages
// ============================================

import {
  createEffect,
  createSignal,
  Show,
  For,
} from 'solid-js';
import { cn } from '../../utils/cn';
import { formatMessageTime, formatFullDateTime, getInitials } from '../../utils/date';
import { avatarSizedUrl } from '../../utils/avatar';
import type { Message } from '../../types/messages';

// Components
import MessageContent from './MessageContent';
import Reactions from './Reactions';

// Icons
import {
  HiOutlineXMark,
  HiOutlinePaperAirplane,
  HiOutlineChatBubbleLeft,
  HiOutlineBell,
  HiOutlineBellSlash,
} from 'solid-icons/hi';

// ============================================
// Types
// ============================================

interface ThreadViewProps {
  rootMessage: Message;
  replies: Message[];
  isLoading: boolean;
  currentUserId?: string;
  isFollowing?: boolean;
  onClose: () => void;
  onSendReply: (content: string) => void;
  onAddReaction?: (messageId: string, emoji: string) => void;
  onRemoveReaction?: (messageId: string, emoji: string) => void;
  onFollowThread?: () => void;
  onUnfollowThread?: () => void;
  onEdit?: (message: Message) => void;
  onDelete?: (messageId: string) => void;
}

// ============================================
// Thread Header Component
// ============================================

interface ThreadHeaderProps {
  replyCount: number;
  participantCount: number;
  isFollowing: boolean;
  onClose: () => void;
  onFollowToggle: () => void;
}

function ThreadHeader(props: ThreadHeaderProps) {
  return (
    <div class="flex items-center justify-between px-4 py-3 border-b border-border-1 bg-bg-surface-1">
      <div class="flex items-center gap-3">
        <div class="w-8 h-8 rounded-full bg-brand/10 flex items-center justify-center">
          <HiOutlineChatBubbleLeft size={16} class="text-brand" />
        </div>
        <div>
          <h3 class="font-semibold text-text-1">Thread</h3>
          <p class="text-xs text-text-3">
            {props.replyCount} {props.replyCount === 1 ? 'reply' : 'replies'}
            {' · '}
            {props.participantCount} {props.participantCount === 1 ? 'participant' : 'participants'}
          </p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <button
          type="button"
          onClick={props.onFollowToggle}
          class={cn(
            'p-2 rounded-lg transition-colors',
            props.isFollowing
              ? 'text-brand bg-brand/10 hover:bg-brand/20'
              : 'text-text-3 hover:text-text-1 hover:bg-bg-surface-2'
          )}
          title={props.isFollowing ? 'Unfollow thread' : 'Follow thread'}
        >
          <Show when={props.isFollowing} fallback={<HiOutlineBellSlash size={18} />}>
            <HiOutlineBell size={18} />
          </Show>
        </button>
        <button
          type="button"
          onClick={props.onClose}
          class="p-2 text-text-3 hover:text-text-1 hover:bg-bg-surface-2 rounded-lg transition-colors"
          title="Close thread"
        >
          <HiOutlineXMark size={20} />
        </button>
      </div>
    </div>
  );
}

// ============================================
// Parent Message Component
// ============================================

interface ParentMessageProps {
  message: Message;
  currentUserId?: string;
  onAddReaction?: (messageId: string, emoji: string) => void;
  onRemoveReaction?: (messageId: string, emoji: string) => void;
}

function ParentMessage(props: ParentMessageProps) {
  const initials = () => getInitials(props.message.username, 2);
  const isOwnMessage = () => props.message.userId === props.currentUserId;

  return (
    <div class="px-4 py-4 border-b border-border-1 bg-bg-surface-1/50">
      <div class="flex gap-3">
        {/* Avatar */}
        <div class="flex-shrink-0">
          <Show
            when={props.message.avatarUrl}
            fallback={
              <div class="w-10 h-10 rounded-full bg-brand/10 flex items-center justify-center text-brand font-medium">
                {initials()}
              </div>
            }
          >
            <img
              src={avatarSizedUrl(props.message.avatarUrl, 'md')}
              alt={props.message.username}
              class="w-10 h-10 rounded-full object-cover"
            />
          </Show>
        </div>

        {/* Content */}
        <div class="flex-1 min-w-0">
          <div class="flex items-baseline gap-2 mb-1">
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
            >
              {formatMessageTime(props.message.timestamp)}
            </time>
          </div>

          <MessageContent
            content={props.message.content}
            files={props.message.files}
            editedAt={props.message.editedAt}
          />

          <Show when={(props.onAddReaction || props.onRemoveReaction) || (props.message.reactions && props.message.reactions.length > 0)}>
            <Reactions
              reactions={props.message.reactions || []}
              messageId={props.message.id}
              onAddReaction={(emoji) => props.onAddReaction?.(props.message.id, emoji)}
              onRemoveReaction={(emoji) => props.onRemoveReaction?.(props.message.id, emoji)}
            />
          </Show>
        </div>
      </div>
    </div>
  );
}

// ============================================
// Thread Reply Component
// ============================================

interface ThreadReplyProps {
  message: Message;
  currentUserId?: string;
  onAddReaction?: (messageId: string, emoji: string) => void;
  onRemoveReaction?: (messageId: string, emoji: string) => void;
  onEdit?: (message: Message) => void;
  onDelete?: (messageId: string) => void;
}

function ThreadReply(props: ThreadReplyProps) {
  const initials = () => getInitials(props.message.username, 2);
  const isOwnMessage = () => props.message.userId === props.currentUserId;
  const [isEditing, setIsEditing] = createSignal(false);
  const [editContent, setEditContent] = createSignal(props.message.content);

  const handleEditSave = () => {
    if (editContent().trim() !== props.message.content) {
      props.onEdit?.({ ...props.message, content: editContent().trim() });
    }
    setIsEditing(false);
  };

  return (
    <div
      class={cn(
        'group flex gap-3 px-4 py-3 hover:bg-bg-surface-2/50 transition-colors',
        isOwnMessage() && 'bg-brand/5'
      )}
    >
      {/* Avatar */}
      <div class="flex-shrink-0">
        <Show
          when={props.message.avatarUrl}
          fallback={
            <div class="w-8 h-8 rounded-full bg-brand/10 flex items-center justify-center text-brand font-medium text-xs">
              {initials()}
            </div>
          }
        >
          <img
            src={avatarSizedUrl(props.message.avatarUrl, 'sm')}
            alt={props.message.username}
            class="w-8 h-8 rounded-full object-cover"
          />
        </Show>
      </div>

      {/* Content */}
      <div class="flex-1 min-w-0">
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
          >
            {formatMessageTime(props.message.timestamp)}
          </time>
        </div>

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
                    setIsEditing(false);
                    setEditContent(props.message.content);
                  }
                }}
                rows={2}
                class="w-full px-3 py-2 bg-bg-app border border-border-1 rounded-lg text-text-1 text-sm focus:outline-none focus:border-brand resize-none"
                autofocus
              />
              <div class="flex items-center gap-2 mt-1">
                <button
                  onClick={handleEditSave}
                  class="px-2 py-1 bg-brand text-white text-xs font-medium rounded hover:bg-brand/90 transition-colors"
                >
                  Save
                </button>
                <button
                  onClick={() => {
                    setIsEditing(false);
                    setEditContent(props.message.content);
                  }}
                  class="px-2 py-1 text-xs font-medium text-text-2 hover:text-text-1 transition-colors"
                >
                  Cancel
                </button>
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

        <Show when={(props.onAddReaction || props.onRemoveReaction) || (props.message.reactions && props.message.reactions.length > 0)}>
          <Reactions
            reactions={props.message.reactions || []}
            messageId={props.message.id}
            onAddReaction={(emoji) => props.onAddReaction?.(props.message.id, emoji)}
            onRemoveReaction={(emoji) => props.onRemoveReaction?.(props.message.id, emoji)}
          />
        </Show>
      </div>

      {/* Actions */}
      <Show when={!isEditing()}>
        <div class="opacity-0 group-hover:opacity-100 transition-opacity">
          <div class="flex items-center gap-1">
            <button
              type="button"
              onClick={() => props.onAddReaction?.(props.message.id, '👍')}
              class="p-1.5 text-text-3 hover:text-text-1 hover:bg-bg-surface-2 rounded transition-colors"
              title="React"
            >
              👍
            </button>
            <Show when={isOwnMessage()}>
              <button
                type="button"
                onClick={() => setIsEditing(true)}
                class="p-1.5 text-text-3 hover:text-text-1 hover:bg-bg-surface-2 rounded transition-colors"
                title="Edit"
              >
                ✏️
              </button>
              <button
                type="button"
                onClick={() => props.onDelete?.(props.message.id)}
                class="p-1.5 text-text-3 hover:text-danger hover:bg-danger/10 rounded transition-colors"
                title="Delete"
              >
                🗑️
              </button>
            </Show>
          </div>
        </div>
      </Show>
    </div>
  );
}

// ============================================
// Reply Input Component
// ============================================

interface ReplyInputProps {
  onSend: (content: string) => void;
  disabled?: boolean;
}

function ReplyInput(props: ReplyInputProps) {
  const [content, setContent] = createSignal('');
  const [isSending, setIsSending] = createSignal(false);

  const handleSend = async () => {
    const text = content().trim();
    if (!text || props.disabled) return;

    setIsSending(true);
    try {
      await props.onSend(text);
      setContent('');
    } finally {
      setIsSending(false);
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div class="border-t border-border-1 p-4 bg-bg-surface-1">
      <div class="flex gap-2">
        <textarea
          value={content()}
          onInput={(e) => setContent(e.currentTarget.value)}
          onKeyDown={handleKeyDown}
          placeholder="Reply in thread..."
          rows={1}
          disabled={props.disabled || isSending()}
          class="flex-1 px-3 py-2 bg-bg-app border border-border-1 rounded-lg text-text-1 text-sm placeholder:text-text-3 focus:outline-none focus:border-brand resize-none min-h-[40px] max-h-[120px]"
        />
        <button
          type="button"
          onClick={handleSend}
          disabled={!content().trim() || isSending()}
          class={cn(
            'px-3 py-2 bg-brand text-white rounded-lg transition-colors flex items-center justify-center',
            (!content().trim() || isSending()) && 'opacity-50 cursor-not-allowed'
          )}
        >
          <Show when={isSending()}>
            <span class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin mr-1" />
          </Show>
          <HiOutlinePaperAirplane size={18} />
        </button>
      </div>
      <p class="mt-1 text-xs text-text-3">
        Press Enter to send, Shift+Enter for new line
      </p>
    </div>
  );
}

// ============================================
// Main Thread View Component
// ============================================

export default function ThreadView(props: ThreadViewProps) {
  let repliesContainerRef: HTMLDivElement | undefined;

  // Scroll to bottom when new replies arrive
  createEffect((prevLength?: number) => {
    const currentLength = props.replies.length;
    if (prevLength !== undefined && currentLength > prevLength) {
      if (repliesContainerRef) {
        repliesContainerRef.scrollTop = repliesContainerRef.scrollHeight;
      }
    }
    return currentLength;
  });

  // Get unique participant count
  const participantCount = () => {
    const userIds = new Set<string>();
    userIds.add(props.rootMessage.userId);
    props.replies.forEach((r) => userIds.add(r.userId));
    return userIds.size;
  };

  const handleFollowToggle = () => {
    if (props.isFollowing) {
      props.onUnfollowThread?.();
    } else {
      props.onFollowThread?.();
    }
  };

  return (
    <div class="flex flex-col h-full bg-bg-app">
      {/* Header */}
      <ThreadHeader
        replyCount={props.replies.length}
        participantCount={participantCount()}
        isFollowing={props.isFollowing ?? false}
        onClose={props.onClose}
        onFollowToggle={handleFollowToggle}
      />

      {/* Parent Message */}
      <ParentMessage
        message={props.rootMessage}
        currentUserId={props.currentUserId}
        onAddReaction={props.onAddReaction}
        onRemoveReaction={props.onRemoveReaction}
      />

      {/* Replies */}
      <div
        ref={repliesContainerRef}
        class="flex-1 overflow-y-auto custom-scrollbar"
      >
        <Show
          when={props.replies.length > 0}
          fallback={
            <div class="flex flex-col items-center justify-center py-8 text-center text-text-3">
              <div class="w-12 h-12 rounded-full bg-bg-surface-2 flex items-center justify-center mb-3">
                <HiOutlineChatBubbleLeft size={20} />
              </div>
              <p class="text-sm">No replies yet</p>
              <p class="text-xs mt-1">Be the first to reply!</p>
            </div>
          }
        >
          <For each={props.replies}>
            {(reply) => (
              <ThreadReply
                message={reply}
                currentUserId={props.currentUserId}
                onAddReaction={props.onAddReaction}
                onRemoveReaction={props.onRemoveReaction}
                onEdit={props.onEdit}
                onDelete={props.onDelete}
              />
            )}
          </For>
        </Show>
      </div>

      {/* Reply Input */}
      <ReplyInput
        onSend={props.onSendReply}
        disabled={props.isLoading}
      />
    </div>
  );
}

// ============================================
// Exports
// ============================================

export type { ThreadViewProps };
