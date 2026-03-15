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
  fetchMessages,
  fetchOlderMessages,
  getMessages,
  addReaction,
  removeReaction,
  editMessage,
  deleteMessage,
} from '../stores/messages';
import { useWebSocket } from '../hooks/useWebSocket';
import type { Message } from '../types/messages';

// Components
import { MessageList } from '../components/messages';
import { ChannelHeader } from '../components/channel';
import { MessageInput } from '../components/messages/MessageInput';

// ============================================
// Channel Route Component
// ============================================

export default function Channel() {
  const params = useParams();
  const navigate = useNavigate();
  const channelId = () => params.channelId;

  // Initialize WebSocket connection
  useWebSocket();

  // Local state
  const [isLoadingOlder, setIsLoadingOlder] = createSignal(false);
  const [showSearch, setShowSearch] = createSignal(false);
  const [searchQuery, setSearchQuery] = createSignal('');

  // Get current channel
  const channel = currentChannel;

  // Get messages for current channel
  const messages = createMemo(() => {
    const id = channelId();
    return id ? getMessages(id) : [];
  });

  // Check if there are more older messages to load
  const hasMoreOlder = createMemo(() => {
    const id = channelId();
    return id ? messages().length >= 30 : false;
  });

  // Select channel when ID changes
  createEffect(() => {
    const id = channelId();
    if (id) {
      selectChannel(id);
      fetchMessages(id);
      fetchChannelMembers(id);
    }
  });

  // Handle adding a reaction
  const handleAddReaction = async (messageId: string, emoji: string) => {
    await addReaction(messageId, emoji);
  };

  // Handle removing a reaction
  const handleRemoveReaction = async (messageId: string, emoji: string) => {
    await removeReaction(messageId, emoji);
  };

  // Handle editing a message
  const handleEditMessage = async (message: Message) => {
    const newContent = prompt('Edit message:', message.content);
    if (newContent && newContent !== message.content) {
      await editMessage(message.id, newContent);
    }
  };

  // Handle deleting a message
  const handleDeleteMessage = async (messageId: string) => {
    if (confirm('Are you sure you want to delete this message?')) {
      await deleteMessage(messageId);
    }
  };

  // Handle loading older messages
  const handleLoadOlder = async () => {
    const id = channelId();
    if (!id || isLoadingOlder()) return;

    setIsLoadingOlder(true);
    try {
      await fetchOlderMessages(id);
    } finally {
      setIsLoadingOlder(false);
    }
  };

  // Handle reply to thread
  const handleReply = (messageId: string) => {
    navigate(`/channels/${channelId()}/threads/${messageId}`);
  };

  return (
    <div class="h-full flex flex-col bg-bg-app">
      {/* Channel Header */}
      <ChannelHeader onSearchClick={() => setShowSearch(true)} />

      {/* Search Bar (conditionally shown) */}
      <Show when={showSearch()}>
        <div class="px-4 py-2 border-b border-border-1 bg-bg-surface-1 flex items-center gap-2">
          <input
            type="text"
            placeholder="Search in channel..."
            value={searchQuery()}
            onInput={(e) => setSearchQuery(e.currentTarget.value)}
            class="flex-1 px-3 py-1.5 bg-bg-app border border-border-1 rounded-lg text-sm focus:outline-none focus:border-brand"
          />
          <button
            type="button"
            class="px-3 py-1.5 text-sm text-text-2 hover:text-text-1"
            onClick={() => setShowSearch(false)}
          >
            Cancel
          </button>
        </div>
      </Show>

      {/* Message List */}
      <div class="flex-1 overflow-hidden">
        <Show when={channelId()}>
          {(id) => (
            <MessageList
              channelId={id()}
              messages={messages()}
              isLoading={false}
              isLoadingOlder={isLoadingOlder()}
              hasMoreOlder={hasMoreOlder()}
              onLoadOlder={handleLoadOlder}
              onAddReaction={handleAddReaction}
              onRemoveReaction={handleRemoveReaction}
              onEdit={handleEditMessage}
              onDelete={handleDeleteMessage}
              onReply={handleReply}
            />
          )}
        </Show>
      </div>

      {/* Message Input */}
      <div class="border-t border-border-1 bg-bg-surface-1">
        <Show when={channelId()}>
          {(id) => (
            <MessageInput
              channelId={id()}
              placeholder={`Message ${channel()?.display_name || 'channel'}`}
            />
          )}
        </Show>
      </div>
    </div>
  );
}
