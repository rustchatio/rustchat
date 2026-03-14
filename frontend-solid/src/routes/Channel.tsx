// ============================================
// Channel Route - Message List View
// ============================================

import { createEffect, createSignal, Show, createMemo } from 'solid-js';
import { useParams, useNavigate } from '@solidjs/router';
import {
  selectChannel,
  fetchChannelMembers,
  currentChannel,
} from '../stores/channels';
import {
  messageStore,
  fetchMessages,
  fetchOlderMessages,
  getMessages,
  sendMessage,
  addReaction,
  removeReaction,
  editMessage,
  deleteMessage,
} from '../stores/messages';
import { authStore } from '../stores/auth';
import { cn } from '../utils/cn';

// Components
import { MessageList } from '../components/messages';

// Icons
import {
  HiOutlinePaperAirplane,
  HiOutlinePaperClip,
  HiOutlineFaceSmile,
  HiOutlineHashtag,
  HiOutlineLockClosed,
  HiOutlineUser,
} from 'solid-icons/hi';

// ============================================
// Channel Header Component
// ============================================

function ChannelHeader() {
  const channel = currentChannel;
  const memberCount = () => {
    const id = channel()?.id;
    return id ? (messageStore as unknown as { membersByChannel: Record<string, unknown[]> }).membersByChannel[id]?.length || 0 : 0;
  };

  const channelIcon = () => {
    const type = channel()?.channel_type;
    if (type === 'private') return <HiOutlineLockClosed size={18} />;
    if (type === 'direct') return <HiOutlineUser size={18} />;
    return <HiOutlineHashtag size={18} />;
  };

  return (
    <div class="flex items-center justify-between px-4 py-2 border-b border-border-1 bg-bg-surface-1">
      <div class="flex items-center gap-2">
        <span class="text-text-2">{channelIcon()}</span>
        <h2 class="font-semibold text-text-1">{channel()?.display_name}</h2>
        <Show when={channel()?.header}>
          <span class="text-text-3 text-sm hidden md:inline">|</span>
          <p class="text-text-3 text-sm hidden md:block truncate max-w-md">
            {channel()?.header}
          </p>
        </Show>
      </div>
      <button class="flex items-center gap-2 px-3 py-1.5 text-sm text-text-2 hover:text-text-1 hover:bg-bg-surface-2 rounded-lg transition-colors">
        <HiOutlineUser size={16} />
        <span>{memberCount()}</span>
      </button>
    </div>
  );
}

// ============================================
// Message Input Component
// ============================================

interface MessageInputProps {
  channelId: string;
  onSend: (content: string) => Promise<void>;
}

function MessageInput(props: MessageInputProps) {
  const [content, setContent] = createSignal('');
  const [isSending, setIsSending] = createSignal(false);
  const channel = currentChannel;

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    const text = content().trim();
    if (!text || isSending()) return;

    setIsSending(true);
    try {
      await props.onSend(text);
      setContent('');
    } catch (err) {
      console.error('Failed to send message:', err);
    } finally {
      setIsSending(false);
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      void handleSubmit(e);
    }
  };

  return (
    <form
      onSubmit={handleSubmit}
      class="border-t border-border-1 p-4 bg-bg-surface-1"
    >
      <div class="flex flex-col gap-2">
        {/* Input Toolbar */}
        <div class="flex items-center gap-1 px-2">
          <button
            type="button"
            class="p-1.5 rounded text-text-3 hover:bg-bg-surface-2 hover:text-text-1 transition-colors"
            aria-label="Add attachment"
            title="Add attachment"
          >
            <HiOutlinePaperClip size={18} />
          </button>
          <button
            type="button"
            class="p-1.5 rounded text-text-3 hover:bg-bg-surface-2 hover:text-text-1 transition-colors"
            aria-label="Add emoji"
            title="Add emoji"
          >
            <HiOutlineFaceSmile size={18} />
          </button>
        </div>

        {/* Input Field */}
        <div class="flex gap-2">
          <div class="flex-1 relative">
            <textarea
              value={content()}
              onInput={(e) => setContent(e.currentTarget.value)}
              onKeyDown={handleKeyDown}
              placeholder={`Message ${channel()?.display_name || ''}`}
              rows={1}
              disabled={isSending()}
              class="w-full px-4 py-2.5 bg-bg-app border border-border-1 rounded-lg text-text-1 placeholder:text-text-3 focus:outline-none focus:border-brand resize-none min-h-[44px] max-h-[200px]"
              style={{ 'overflow-y': 'auto' }}
            />
          </div>
          <button
            type="submit"
            disabled={!content().trim() || isSending()}
            class={cn(
              'px-4 py-2 bg-brand text-white rounded-lg font-medium transition-colors flex items-center justify-center',
              (!content().trim() || isSending()) && 'opacity-50 cursor-not-allowed'
            )}
            aria-label="Send message"
          >
            <Show when={isSending()}>
              <span class="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin mr-2" />
            </Show>
            <HiOutlinePaperAirplane size={20} />
          </button>
        </div>
      </div>
    </form>
  );
}

// ============================================
// Main Channel Component
// ============================================

export default function Channel() {
  const params = useParams();
  const navigate = useNavigate();
  const channelId = () => params.channelId;

  // Get current channel and messages
  const messages = createMemo(() => {
    const id = channelId();
    return id ? getMessages(id)() : [];
  });

  const hasMoreOlder = createMemo(() => {
    const id = channelId();
    return id ? messageStore.hasMoreOlder(id)() : false;
  });
  


  const currentUserId = () => authStore.user()?.id;

  // Set current channel when params change
  createEffect(() => {
    const id = channelId();
    if (id) {
      selectChannel(id);
      void fetchMessages(id);
      void fetchChannelMembers(id);
    }
  });

  // Handle sending a message
  const handleSendMessage = async (content: string) => {
    const chId = channelId();
    if (!chId) return;
    await sendMessage(chId, content);
  };

  // Handle loading older messages
  const handleLoadOlder = () => {
    const chId = channelId();
    if (chId) {
      void fetchOlderMessages(chId);
    }
  };

  // Handle reply click - navigate to thread
  const handleReply = (messageId: string) => {
    const chId = channelId();
    if (chId) {
      navigate(`/channels/${chId}/thread/${messageId}`);
    }
  };

  // Handle edit message
  const handleEdit = async (message: { id: string; content: string }) => {
    await editMessage(message.id, message.content);
  };

  // Handle delete message
  const handleDelete = async (messageId: string) => {
    await deleteMessage(messageId);
  };

  // Handle add reaction
  const handleAddReaction = async (messageId: string, emoji: string) => {
    await addReaction(messageId, emoji);
  };

  // Handle remove reaction
  const handleRemoveReaction = async (messageId: string, emoji: string) => {
    await removeReaction(messageId, emoji);
  };

  // Handle thread click
  const handleThreadClick = (messageId: string) => {
    const chId = channelId();
    if (chId) {
      navigate(`/channels/${chId}/thread/${messageId}`);
    }
  };

  return (
    <div class="h-full flex flex-col bg-bg-app">
      {/* Channel Header */}
      <ChannelHeader />

      {/* Message List */}
      <div class="flex-1 min-h-0">
        <Show when={channelId()} fallback={<div class="flex items-center justify-center h-full text-text-3">Loading...</div>}>
          {(id) => (
            <MessageList
              channelId={id()}
              messages={messages()}
              isLoading={messageStore.isLoading()}
              isLoadingOlder={messageStore.isLoadingOlder()}
              hasMoreOlder={hasMoreOlder()}
              lastReadMessageId={messageStore.readStatesByChannel[id()]?.last_read_message_id?.toString()}
              currentUserId={currentUserId()}
              onLoadOlder={handleLoadOlder}
              onReply={handleReply}
              onEdit={handleEdit}
              onDelete={handleDelete}
              onAddReaction={handleAddReaction}
              onRemoveReaction={handleRemoveReaction}
              onThreadClick={handleThreadClick}
            />
          )}
        </Show>
      </div>

      {/* Message Input */}
      <Show when={channelId()}>
        {(id) => (
          <MessageInput
            channelId={id()}
            onSend={handleSendMessage}
          />
        )}
      </Show>
    </div>
  );
}
