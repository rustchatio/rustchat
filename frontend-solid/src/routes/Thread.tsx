// ============================================
// Thread Route - Thread View
// ============================================

import { createEffect, createMemo, Show } from 'solid-js';
import { useParams, useNavigate } from '@solidjs/router';

import {
  messageStore,
  fetchThread,
  getReplies,
  sendMessage,
  addReaction,
  removeReaction,
  editMessage,
  deleteMessage,
} from '../stores/messages';
import { authStore } from '../stores/auth';

// Components
import { ThreadView } from '../components/messages';

// ============================================
// Main Thread Route Component
// ============================================

export default function Thread() {
  const params = useParams();
  const navigate = useNavigate();
  const channelId = () => params.channelId;
  const threadId = () => params.threadId;

  // Get replies and root message
  const replies = createMemo(() => {
    const id = threadId();
    return id ? getReplies(id)() : [];
  });

  // Get root message from the store or replies
  const rootMessage = createMemo(() => {
    const tid = threadId();
    const cid = channelId();
    if (!tid || !cid) return undefined;

    // First check if the root message is in the replies
    const threadReplies = getReplies(tid)();
    const root = threadReplies.find((r) => r.id === tid);
    if (root) return root;

    // Otherwise check the channel messages
    const channelMessages = messageStore.messagesByChannel[cid] || [];
    return channelMessages.find((m) => m.id === tid);
  });

  // Fetch thread when params change
  createEffect(() => {
    const cid = channelId();
    const tid = threadId();
    if (cid && tid) {
      void fetchThread(tid);
    }
  });

  // Handle sending a reply
  const handleSendReply = async (content: string) => {
    const cid = channelId();
    const tid = threadId();
    if (!cid || !tid) return;

    await sendMessage(cid, content, tid);
  };

  // Handle close
  const handleClose = () => {
    navigate(`/channels/${channelId()}`, { replace: true });
  };

  // Handle add reaction
  const handleAddReaction = async (messageId: string, emoji: string) => {
    await addReaction(messageId, emoji);
  };

  // Handle remove reaction
  const handleRemoveReaction = async (messageId: string, emoji: string) => {
    await removeReaction(messageId, emoji);
  };

  // Handle edit message
  const handleEdit = async (message: { id: string; content: string }) => {
    await editMessage(message.id, message.content);
  };

  // Handle delete message
  const handleDelete = async (messageId: string) => {
    await deleteMessage(messageId);
  };

  return (
    <div class="h-full">
      <Show
        when={rootMessage()}
        fallback={
          <div class="flex items-center justify-center h-full">
            <div class="text-center text-text-3">
              <div class="w-8 h-8 border-4 border-brand/30 border-t-brand rounded-full animate-spin mx-auto mb-2" />
              <p>Loading thread...</p>
            </div>
          </div>
        }
      >
        <ThreadView
          rootMessage={rootMessage()!}
          replies={replies().filter((r) => r.id !== threadId())}
          isLoading={messageStore.isLoading()}
          currentUserId={authStore.user()?.id}
          isFollowing={false} // TODO: Add thread following to store
          onClose={handleClose}
          onSendReply={handleSendReply}
          onAddReaction={handleAddReaction}
          onRemoveReaction={handleRemoveReaction}
          onEdit={handleEdit}
          onDelete={handleDelete}
        />
      </Show>
    </div>
  );
}
