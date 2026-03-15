// ============================================
// Thread Route - Thread View
// ============================================

import { createEffect, createMemo, createSignal, Show } from 'solid-js';
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
import { channelStore, fetchChannel } from '../stores/channels';

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
  const [teamId, setTeamId] = createSignal<string | null>(null);
  const [isFollowing, setIsFollowing] = createSignal(false);

  // Get replies and root message
  const replies = createMemo(() => {
    const id = threadId();
    return id ? getReplies(id) : [];
  });

  // Get root message from the store or replies
  const rootMessage = createMemo(() => {
    const tid = threadId();
    const cid = channelId();
    if (!tid || !cid) return undefined;

    // First check if the root message is in the replies
    const threadReplies = getReplies(tid);
    const root = threadReplies.find((r: { id: string }) => r.id === tid);
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

  createEffect(() => {
    const cid = channelId();
    if (!cid) {
      setTeamId(null);
      return;
    }

    const existing = channelStore.getChannel(cid);
    if (existing?.team_id) {
      setTeamId(existing.team_id);
      return;
    }

    void fetchChannel(cid)
      .then((channel) => {
        setTeamId(channel.team_id);
      })
      .catch(() => {
        setTeamId(null);
      });
  });

  const loadFollowing = async () => {
    const userId = authStore.user()?.id;
    const token = authStore.token;
    const tid = threadId();
    const currentTeamId = teamId();
    if (!userId || !token || !tid || !currentTeamId) {
      setIsFollowing(false);
      return;
    }

    try {
      const response = await fetch(
        `/api/v4/users/${encodeURIComponent(userId)}/teams/${encodeURIComponent(currentTeamId)}/threads/${encodeURIComponent(tid)}`,
        {
          headers: { Authorization: `Bearer ${token}` },
        }
      );
      if (!response.ok) {
        setIsFollowing(false);
        return;
      }
      const payload = (await response.json()) as {
        is_following?: boolean;
        isFollowing?: boolean;
      };
      setIsFollowing(Boolean(payload.is_following ?? payload.isFollowing));
    } catch {
      setIsFollowing(false);
    }
  };

  createEffect(() => {
    threadId();
    teamId();
    authStore.user()?.id;
    authStore.token;
    void loadFollowing();
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

  const setFollowingState = async (follow: boolean) => {
    const userId = authStore.user()?.id;
    const token = authStore.token;
    const tid = threadId();
    const currentTeamId = teamId();
    if (!userId || !token || !tid || !currentTeamId) return;

    const method = follow ? 'PUT' : 'DELETE';
    const response = await fetch(
      `/api/v4/users/${encodeURIComponent(userId)}/teams/${encodeURIComponent(currentTeamId)}/threads/${encodeURIComponent(tid)}/following`,
      {
        method,
        headers: { Authorization: `Bearer ${token}` },
      }
    );

    if (!response.ok) {
      throw new Error('Failed to update thread follow state');
    }

    const payload = (await response.json()) as {
      is_following?: boolean;
      isFollowing?: boolean;
    };
    if (typeof payload.is_following === 'boolean' || typeof payload.isFollowing === 'boolean') {
      setIsFollowing(Boolean(payload.is_following ?? payload.isFollowing));
    } else {
      setIsFollowing(follow);
    }
  };

  const handleFollowThread = async () => {
    await setFollowingState(true);
  };

  const handleUnfollowThread = async () => {
    await setFollowingState(false);
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
          replies={replies().filter((r: { id: string }) => r.id !== threadId())}
          isLoading={messageStore.isLoading()}
          currentUserId={authStore.user()?.id}
          isFollowing={isFollowing()}
          onClose={handleClose}
          onSendReply={handleSendReply}
          onAddReaction={handleAddReaction}
          onRemoveReaction={handleRemoveReaction}
          onFollowThread={handleFollowThread}
          onUnfollowThread={handleUnfollowThread}
          onEdit={handleEdit}
          onDelete={handleDelete}
        />
      </Show>
    </div>
  );
}
