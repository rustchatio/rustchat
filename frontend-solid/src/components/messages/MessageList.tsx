// ============================================
// Message List Component
// Virtual scrolling with infinite scroll up
// ============================================

import {
  createEffect,
  createSignal,
  createMemo,
  For,
  Show,
  onMount,
  onCleanup,
} from 'solid-js';
import { createStore } from 'solid-js/store';

import { formatDateSeparator, isSameDay } from '../../utils/date';
import type { Message as MessageType } from '../../types/messages';

// Components
import MessageComp from './Message';

// Icons
import { HiOutlineChevronDown } from 'solid-icons/hi';

// ============================================
// Types
// ============================================

interface MessageListProps {
  channelId: string;
  messages: MessageType[];
  isLoading: boolean;
  isLoadingOlder: boolean;
  hasMoreOlder: boolean;
  lastReadMessageId?: string | null;
  currentUserId?: string;
  onLoadOlder: () => void;
  onReply?: (messageId: string) => void;
  onEdit?: (message: MessageType) => void;
  onDelete?: (messageId: string) => void;
  onAddReaction?: (messageId: string, emoji: string) => void;
  onRemoveReaction?: (messageId: string, emoji: string) => void;
  onThreadClick?: (messageId: string) => void;
}

interface MessageItem {
  type: 'date-separator' | 'unread-separator' | 'message';
  id: string;
  message?: MessageType;
  date?: Date;
  label?: string;
  isFirstUnread?: boolean;
}

// ============================================
// Date Separator Component
// ============================================

interface DateSeparatorProps {
  label: string;
}

function DateSeparator(props: DateSeparatorProps) {
  return (
    <div class="sticky top-0 z-10 flex items-center justify-center py-2 bg-bg-app/95 backdrop-blur-sm">
      <span class="px-3 py-1 text-xs font-medium text-text-3 bg-bg-surface-1 rounded-full border border-border-1 shadow-sm">
        {props.label}
      </span>
    </div>
  );
}

// ============================================
// Unread Separator Component
// ============================================

function UnreadSeparator() {
  return (
    <div class="relative flex items-center py-2 my-1">
      <div class="flex-1 h-0.5 bg-brand/50" />
      <span class="px-3 py-1 text-xs font-medium text-brand bg-brand/10 rounded-full mx-2">
        New messages
      </span>
      <div class="flex-1 h-0.5 bg-brand/50" />
    </div>
  );
}

// ============================================
// Loading Skeleton Component
// ============================================

function MessageSkeleton() {
  return (
    <div class="flex gap-3 px-4 py-3 animate-pulse">
      <div class="w-10 h-10 rounded-full bg-bg-surface-2 flex-shrink-0" />
      <div class="flex-1 space-y-2">
        <div class="flex items-center gap-2">
          <div class="h-4 w-24 bg-bg-surface-2 rounded" />
          <div class="h-3 w-12 bg-bg-surface-2 rounded" />
        </div>
        <div class="h-3 w-3/4 bg-bg-surface-2 rounded" />
        <div class="h-3 w-1/2 bg-bg-surface-2 rounded" />
      </div>
    </div>
  );
}

// ============================================
// Empty State Component
// ============================================

interface EmptyStateProps {
  channelName?: string;
}

function EmptyState(props: EmptyStateProps) {
  return (
    <div class="flex flex-col items-center justify-center h-full p-8 text-center">
      <div class="w-16 h-16 rounded-full bg-bg-surface-2 flex items-center justify-center mb-4">
        <span class="text-3xl">💬</span>
      </div>
      <h3 class="text-lg font-semibold text-text-1 mb-2">
        {props.channelName ? `Welcome to #${props.channelName}` : 'No messages yet'}
      </h3>
      <p class="text-sm text-text-3 max-w-xs">
        {props.channelName 
          ? 'Be the first to send a message in this channel!'
          : 'Start a conversation by sending a message.'}
      </p>
    </div>
  );
}

// ============================================
// Jump to Bottom Button
// ============================================

interface JumpToBottomButtonProps {
  onClick: () => void;
  newMessageCount?: number;
}

function JumpToBottomButton(props: JumpToBottomButtonProps) {
  return (
    <button
      type="button"
      onClick={props.onClick}
      class="absolute bottom-4 right-4 flex items-center gap-2 px-3 py-2 bg-bg-surface-1 text-text-1 rounded-full shadow-lg border border-border-1 hover:bg-bg-surface-2 hover:shadow-xl transition-all z-20"
    >
      <Show when={props.newMessageCount && props.newMessageCount > 0}>
        <span class="text-xs font-medium bg-brand text-white px-2 py-0.5 rounded-full">
          {props.newMessageCount}
        </span>
      </Show>
      <span class="text-sm">Jump to bottom</span>
      <HiOutlineChevronDown size={16} />
    </button>
  );
}

// ============================================
// Loading Older Indicator
// ============================================

function LoadingOlderIndicator() {
  return (
    <div class="flex items-center justify-center py-4">
      <div class="w-6 h-6 border-2 border-brand/30 border-t-brand rounded-full animate-spin" />
    </div>
  );
}

// ============================================
// Main Message List Component
// ============================================

export default function MessageList(props: MessageListProps) {
  let scrollContainerRef: HTMLDivElement | undefined;
  let sentinelRef: HTMLDivElement | undefined;
  let isProgrammaticScroll = false;

  const [isNearBottom, setIsNearBottom] = createSignal(true);
  const [newMessageCount, setNewMessageCount] = createSignal(0);
  const [hasInitiallyScrolled, setHasInitiallyScrolled] = createSignal(false);
  const [localStore] = createStore({
    highlightedMessageId: null as string | null,
  });

  // Process messages into display items with separators
  const displayItems = createMemo<MessageItem[]>(() => {
    const items: MessageItem[] = [];
    const messages = props.messages;
    
    if (messages.length === 0) return items;

    let lastDate: Date | null = null;
    let hasAddedUnreadSeparator = false;

    for (const message of messages) {
      const messageDate = new Date(message.timestamp);

      // Add date separator if different day
      if (!lastDate || !isSameDay(lastDate, messageDate)) {
        items.push({
          type: 'date-separator',
          id: `date-${messageDate.toISOString().split('T')[0]}`,
          date: messageDate,
          label: formatDateSeparator(messageDate),
        });
        lastDate = messageDate;
      }

      // Add unread separator if this is the first unread message
      if (
        !hasAddedUnreadSeparator &&
        props.lastReadMessageId &&
        message.id === props.lastReadMessageId
      ) {
        items.push({
          type: 'unread-separator',
          id: 'unread-separator',
        });
        hasAddedUnreadSeparator = true;
      }

      // Add the message
      items.push({
        type: 'message',
        id: message.id,
        message,
        isFirstUnread: message.id === props.lastReadMessageId,
      });
    }

    return items;
  });

  // Check if we should show avatar (first message of the day or different sender)
  const shouldShowAvatar = (index: number): boolean => {
    const items = displayItems();
    const currentItem = items[index];
    
    if (currentItem.type !== 'message') return false;
    
    // First message
    if (index === 0) return true;
    
    const prevItem = items[index - 1];
    
    // Previous item is a separator
    if (prevItem.type !== 'message') return true;
    
    // Different sender
    if (prevItem.message?.userId !== currentItem.message?.userId) return true;
    
    // Time gap > 5 minutes
    const prevTime = new Date(prevItem.message!.timestamp).getTime();
    const currentTime = new Date(currentItem.message!.timestamp).getTime();
    if (currentTime - prevTime > 5 * 60 * 1000) return true;
    
    return false;
  };

  // Check if message is compact (consecutive from same sender)
  const isCompact = (index: number): boolean => {
    return !shouldShowAvatar(index);
  };

  // Scroll to bottom
  const scrollToBottom = (behavior: ScrollBehavior = 'smooth') => {
    if (!scrollContainerRef) return;
    isProgrammaticScroll = true;
    scrollContainerRef.scrollTo({
      top: scrollContainerRef.scrollHeight,
      behavior,
    });
    setTimeout(() => {
      isProgrammaticScroll = false;
    }, 100);
  };

  // Handle scroll
  const handleScroll = () => {
    if (!scrollContainerRef || isProgrammaticScroll) return;

    const { scrollTop, scrollHeight, clientHeight } = scrollContainerRef;
    const threshold = 100;
    const nearBottom = scrollHeight - scrollTop - clientHeight < threshold;

    setIsNearBottom(nearBottom);
    if (nearBottom) {
      setNewMessageCount(0);
    }

    // Check if we need to load older messages
    if (scrollTop < threshold && props.hasMoreOlder && !props.isLoadingOlder) {
      props.onLoadOlder();
    }
  };

  // Intersection observer for infinite scroll
  onMount(() => {
    if (!sentinelRef) return;

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting && props.hasMoreOlder && !props.isLoadingOlder) {
            props.onLoadOlder();
          }
        });
      },
      { root: scrollContainerRef, threshold: 0 }
    );

    observer.observe(sentinelRef);
    onCleanup(() => observer.disconnect());
  });

  // Initial scroll to bottom on first load
  createEffect(() => {
    if (props.messages.length > 0 && !hasInitiallyScrolled() && !props.isLoading) {
      scrollToBottom('auto');
      setHasInitiallyScrolled(true);
    }
  });

  // Track new messages
  createEffect((prevLength?: number) => {
    const currentLength = props.messages.length;
    
    if (prevLength !== undefined && currentLength > prevLength) {
      const newMessages = props.messages.slice(prevLength);
      const hasIncomingMessages = newMessages.some(
        (m) => m.userId !== props.currentUserId
      );
      
      if (hasIncomingMessages && !isNearBottom()) {
        setNewMessageCount((c) => c + newMessages.length);
      } else if (isNearBottom()) {
        scrollToBottom('smooth');
      }
    }
    
    return currentLength;
  });

  return (
    <div class="relative h-full flex flex-col">
      {/* Message List Container */}
      <div
        ref={scrollContainerRef}
        onScroll={handleScroll}
        class="flex-1 overflow-y-auto custom-scrollbar"
      >
        {/* Loading Older Indicator */}
        <Show when={props.isLoadingOlder}>
          <LoadingOlderIndicator />
        </Show>

        {/* Top Sentinel for Infinite Scroll */}
        <div ref={sentinelRef} class="h-1" />

        {/* Messages */}
        <Show
          when={!props.isLoading}
          fallback={
            <div class="space-y-2 py-4">
              <For each={[1, 2, 3, 4, 5]}>
                {() => <MessageSkeleton />}
              </For>
            </div>
          }
        >
          <Show
            when={props.messages.length > 0}
            fallback={<EmptyState />}
          >
            <div class="py-2">
              <For each={displayItems()}>
                {(item, index) => {
                  if (item.type === 'date-separator') {
                    return <DateSeparator label={item.label!} />;
                  }

                  if (item.type === 'unread-separator') {
                    return <UnreadSeparator />;
                  }

                  const message = item.message!;
                  const isHighlighted = localStore.highlightedMessageId === message.id;

                  return (
                    <MessageComp
                      message={message}
                      showAvatar={shouldShowAvatar(index())}
                      isCompact={isCompact(index())}
                      isHighlighted={isHighlighted}
                      isUnread={item.isFirstUnread}
                      onReply={props.onReply}
                      onEdit={props.onEdit}
                      onDelete={props.onDelete}
                      onAddReaction={props.onAddReaction}
                      onRemoveReaction={props.onRemoveReaction}
                      onThreadClick={props.onThreadClick}
                    />
                  );
                }}
              </For>
            </div>
          </Show>
        </Show>
      </div>

      {/* Jump to Bottom Button */}
      <Show when={!isNearBottom()}>
        <JumpToBottomButton
          onClick={() => scrollToBottom()}
          newMessageCount={newMessageCount()}
        />
      </Show>
    </div>
  );
}

// ============================================
// Exports
// ============================================

export { DateSeparator, UnreadSeparator, EmptyState };
export type { MessageListProps, MessageItem };
