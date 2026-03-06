import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { postsApi, type Post } from '../api/posts'
import { useChannelStore } from './channels'
import { useUnreadStore } from './unreads'
import { useAuthStore } from './auth'
import { useTeamStore } from './teams'
import { normalizeEntityId } from '../utils/idCompat'

export interface Message {
    id: string
    channelId: string
    userId: string
    username: string
    avatarUrl?: string
    email?: string
    content: string
    timestamp: string
    reactions: { emoji: string; count: number; users: string[] }[]
    threadCount?: number
    lastReplyAt?: string
    rootId?: string
    files?: { id: string; name: string; url: string; size: number; mime_type: string }[]
    isPinned: boolean
    isSaved: boolean
    status?: 'sending' | 'delivered' | 'failed'
    clientMsgId?: string
    props?: any
    seq: number | string
    editedAt?: string
}

function toIsoTimestamp(value: unknown): string {
    if (typeof value === 'number' && Number.isFinite(value)) {
        return new Date(value).toISOString()
    }
    if (typeof value === 'string' && value.length > 0) {
        return value
    }
    return new Date().toISOString()
}

function toOptionalIsoTimestamp(value: unknown): string | undefined {
    if (
        value === null ||
        value === undefined ||
        value === '' ||
        value === 0 ||
        value === '0'
    ) {
        return undefined
    }
    return toIsoTimestamp(value)
}

function comparableId(value: unknown): string | undefined {
    if (typeof value !== 'string' || value.length === 0) {
        return undefined
    }
    return normalizeEntityId(value) ?? value
}

function idsMatch(left: unknown, right: unknown): boolean {
    const lhs = comparableId(left)
    const rhs = comparableId(right)
    return !!lhs && !!rhs && lhs === rhs
}

function resolveAuthorDetails(rawPost: Post & { username?: string; avatar_url?: string; email?: string }): {
    username: string
    avatarUrl?: string
    email?: string
} {
    const usernameFromPost = typeof rawPost.username === 'string' ? rawPost.username.trim() : ''
    if (usernameFromPost) {
        return {
            username: usernameFromPost,
            avatarUrl: rawPost.avatar_url,
            email: rawPost.email,
        }
    }

    const authStore = useAuthStore()
    if (idsMatch(rawPost.user_id, authStore.user?.id)) {
        return {
            username: authStore.user?.display_name || authStore.user?.username || 'Unknown',
            avatarUrl: rawPost.avatar_url || authStore.user?.avatar_url,
            email: rawPost.email || authStore.user?.email,
        }
    }

    const teamStore = useTeamStore()
    const teamMember = teamStore.members.find((member) => idsMatch(member.user_id, rawPost.user_id))
    if (teamMember) {
        return {
            username: teamMember.display_name || teamMember.username || 'Unknown',
            avatarUrl: rawPost.avatar_url || teamMember.avatar_url,
            email: rawPost.email,
        }
    }

    return {
        username: 'Unknown',
        avatarUrl: rawPost.avatar_url,
        email: rawPost.email,
    }
}

export function postToMessage(post: Post): Message {
    const rawPost = post as Post & {
        root_id?: string
        create_at?: string | number
        update_at?: string | number
        edit_at?: string | number
        edited_at?: string | number
        pending_post_id?: string
        last_reply_at?: string | number | null
    }
    const rootId = (rawPost.root_post_id ?? rawPost.root_id) || undefined
    const author = resolveAuthorDetails(rawPost)

    return {
        id: rawPost.id,
        channelId: rawPost.channel_id,
        userId: rawPost.user_id,
        username: author.username,
        avatarUrl: author.avatarUrl,
        email: author.email,
        content: rawPost.message,
        timestamp: toIsoTimestamp(rawPost.created_at ?? rawPost.create_at),
        reactions: rawPost.reactions?.map((r: any) => ({
            emoji: r.emoji,
            count: r.count,
            users: r.users.map((u: any) => normalizeEntityId(u) ?? u.toString())
        })) || [],
        rootId,
        threadCount: rawPost.reply_count || 0,
        lastReplyAt: toOptionalIsoTimestamp(rawPost.last_reply_at),
        files: rawPost.files || [],
        isPinned: Boolean(rawPost.is_pinned),
        isSaved: rawPost.is_saved || false,
        status: 'delivered',
        clientMsgId: rawPost.client_msg_id ?? rawPost.pending_post_id,
        props: rawPost.props,
        seq: rawPost.seq ?? 0,
        editedAt: toOptionalIsoTimestamp(rawPost.edited_at ?? rawPost.edit_at),
    }
}

export const useMessageStore = defineStore('messages', () => {
    // Messages grouped by channel
    const messagesByChannel = ref<Record<string, Message[]>>({})
    const repliesByThread = ref<Record<string, Message[]>>({})
    const hasMoreOlderByChannel = ref<Record<string, boolean>>({}) // Track if we can load more history
    const loading = ref(false)
    const isLoadingOlder = ref(false)
    const error = ref<string | null>(null)

    function getMessages(channelId: string) {
        return computed(() => messagesByChannel.value[channelId] || [])
    }

    const hasMoreOlder = computed(() => (channelId: string) => hasMoreOlderByChannel.value[channelId] ?? true)

    async function fetchMessages(channelId: string) {
        loading.value = true
        error.value = null
        try {
            const unreadStore = useUnreadStore()
            const response = await postsApi.list(channelId, { limit: 50 })
            const messages = response.data.messages
                .filter(p => !p.root_post_id)
                .map(postToMessage)
                .reverse()
            messagesByChannel.value[channelId] = messages

            // Update read state in unread store
            if (response.data.read_state) {
                unreadStore.setReadState(channelId, response.data.read_state)
            }

            // If we got fewer than 50, we probably reached the end
            hasMoreOlderByChannel.value[channelId] = response.data.messages.length >= 50
        } catch (e: any) {
            console.error(`Failed to fetch messages for channel ${channelId}:`, e);
            error.value = e.response?.data?.message || e.message || 'Failed to fetch messages'
        } finally {
            loading.value = false
        }
    }

    async function fetchOlderMessages(channelId: string) {
        if (loading.value || !hasMoreOlder.value(channelId)) return

        const currentMessages = messagesByChannel.value[channelId] || []
        if (currentMessages.length === 0) {
            await fetchMessages(channelId)
            return
        }

        // Use the ID of the OLDEST message as the cursor
        const before = currentMessages[0]!.id

        loading.value = true
        isLoadingOlder.value = true
        try {
            const response = await postsApi.list(channelId, { before, limit: 50 })
            const olderMessages = response.data.messages
                .filter(p => !p.root_post_id)
                .map(postToMessage)
                .reverse()

            if (olderMessages.length > 0) {
                messagesByChannel.value[channelId] = [...olderMessages, ...currentMessages]
            }

            hasMoreOlderByChannel.value[channelId] = response.data.messages.length >= 50
        } catch (e: any) {
            console.error('Failed to fetch older messages:', e)
        } finally {
            loading.value = false
            isLoadingOlder.value = false
        }
    }

    async function fetchThread(rootId: string) {
        loading.value = true
        error.value = null
        try {
            const response = await postsApi.getThread(rootId)
            const replies = response.data.map(postToMessage)
            repliesByThread.value[rootId] = replies
        } catch (e: any) {
            console.error(`Failed to fetch thread ${rootId}:`, e);
            error.value = e.response?.data?.message || e.message || 'Failed to fetch thread'
        } finally {
            loading.value = false
        }
    }

    // Optimized for WebSocket usage
    function addOptimisticMessage(message: Message) {
        if (message.rootId) {
            if (!repliesByThread.value[message.rootId]) {
                repliesByThread.value[message.rootId] = []
            }
            repliesByThread.value[message.rootId]?.push(message)
        } else {
            if (!messagesByChannel.value[message.channelId]) {
                messagesByChannel.value[message.channelId] = []
            }
            messagesByChannel.value[message.channelId]?.push(message)
        }
    }

    function updateOptimisticMessage(clientMsgId: string, serverMsg: Message) {
        const channelId = serverMsg.channelId
        const rootId = serverMsg.rootId

        if (rootId) {
            const threadReplies = repliesByThread.value[rootId]
            if (threadReplies) {
                const index = threadReplies.findIndex(m => m.clientMsgId === clientMsgId || m.id === clientMsgId)
                if (index !== -1) {
                    threadReplies[index] = serverMsg
                } else {
                    threadReplies.push(serverMsg)
                }
            }
        } else {
            const channelMessages = messagesByChannel.value[channelId]
            if (channelMessages) {
                const index = channelMessages.findIndex(m => m.clientMsgId === clientMsgId || m.id === clientMsgId)
                if (index !== -1) {
                    channelMessages[index] = serverMsg
                } else {
                    channelMessages.push(serverMsg)
                }
            }
        }
    }

    function handleNewMessage(post: Post) {
        if (!post) {
            return
        }

        const message = postToMessage(post)

        if (message.rootId) {
            // Handle reply
            if (!repliesByThread.value[message.rootId]) {
                repliesByThread.value[message.rootId] = []
            }
            const threadReplies = repliesByThread.value[message.rootId]
            if (threadReplies) {
                const index = threadReplies.findIndex(m => m.id === message.id || (m.clientMsgId && m.clientMsgId === message.clientMsgId))
                if (index !== -1) {
                    threadReplies[index] = message
                } else {
                    threadReplies.push(message)
                }
            }

            // Important: If it was accidentally added to the main channel feed (e.g. by a bug in optimistic logic), remove it
            const channelMessages = messagesByChannel.value[message.channelId]
            if (channelMessages) {
                const idx = channelMessages.findIndex(m => m.id === message.id || (m.clientMsgId && m.clientMsgId === message.clientMsgId))
                if (idx !== -1) {
                    channelMessages.splice(idx, 1)
                }
            }
        } else {
            // Handle root message
            if (!messagesByChannel.value[message.channelId]) {
                messagesByChannel.value[message.channelId] = []
            }
            const channelMessages = messagesByChannel.value[message.channelId]
            if (channelMessages) {
                const index = channelMessages.findIndex(m => m.id === message.id || (m.clientMsgId && m.clientMsgId === message.clientMsgId))
                if (index !== -1) {
                    channelMessages[index] = message
                } else {
                    channelMessages.push(message)
                }
            }
        }

        // Handle notifications
        const channelStore = useChannelStore()
        const authStore = useAuthStore()

        // Unread message counts are authoritative from websocket unread events.
        // Keep only local mention hinting for now.
        if (channelStore.currentChannelId !== message.channelId) {
            const unreadStore = useUnreadStore()

            // Check for mention
            const currentUser = authStore.user
            if (currentUser && message.content.includes(`@${currentUser.username}`)) {
                unreadStore.channelMentions[message.channelId] = (unreadStore.channelMentions[message.channelId] || 0) + 1
            }
        }
    }

    function clearMessages(channelId?: string) {
        if (channelId) {
            delete messagesByChannel.value[channelId]
        } else {
            messagesByChannel.value = {}
        }
    }

    async function pinMessage(messageId: string, channelId: string) {
        try {
            await postsApi.pin(messageId)
            // Update local state
            const message = messagesByChannel.value[channelId]?.find(m => m.id === messageId)
            if (message) {
                message.isPinned = true
            }
        } catch (e: any) {
            error.value = 'Failed to pin message'
            throw e
        }
    }

    async function unpinMessage(messageId: string, channelId: string) {
        try {
            await postsApi.unpin(messageId)
            // Update local state
            const message = messagesByChannel.value[channelId]?.find(m => m.id === messageId)
            if (message) {
                message.isPinned = false
            }
        } catch (e: any) {
            error.value = 'Failed to unpin message'
            throw e
        }
    }

    async function saveMessage(messageId: string, channelId: string) {
        try {
            await postsApi.save(messageId)
            // Update local state
            const message = messagesByChannel.value[channelId]?.find(m => m.id === messageId)
            if (message) {
                message.isSaved = true
            }
        } catch (e: any) {
            error.value = 'Failed to save message'
            throw e
        }
    }

    async function unsaveMessage(messageId: string, channelId: string) {
        try {
            await postsApi.unsave(messageId)
            // Update local state
            const message = messagesByChannel.value[channelId]?.find(m => m.id === messageId)
            if (message) {
                message.isSaved = false
            }
        } catch (e: any) {
            error.value = 'Failed to unsave message'
            throw e
        }
    }

    async function searchMessages(channelId: string, query: string) {
        loading.value = true
        error.value = null
        try {
            const response = await postsApi.list(channelId, { q: query })
            return response.data.messages.map(postToMessage)
        } catch (e: any) {
            error.value = 'Failed to search messages'
            throw e
        } finally {
            loading.value = false
        }
    }

    async function fetchPinnedMessages(channelId: string) {
        loading.value = true
        error.value = null
        try {
            const response = await postsApi.list(channelId, { is_pinned: true })
            return response.data.messages.map(postToMessage)
        } catch (e: any) {
            error.value = 'Failed to fetch pinned messages'
            throw e
        } finally {
            loading.value = false
        }
    }

    async function fetchSavedMessages() {
        loading.value = true
        try {
            const response = await postsApi.getSaved()
            return response.data.map(postToMessage)
        } catch (e: any) {
            error.value = 'Failed to fetch saved messages'
            throw e
        } finally {
            loading.value = false
        }
    }

    function handleMessageUpdate(data: any) {
        if (!data.id) return

        // 1. Update in main channels
        for (const cid in messagesByChannel.value) {
            const messages = messagesByChannel.value[cid]
            if (!messages) continue

            const index = messages.findIndex(m => m.id === data.id)
            if (index !== -1) {
                const msg = messages[index]
                if (!msg) continue

                if (data.message !== undefined) msg.content = data.message
                if (data.is_pinned !== undefined) msg.isPinned = data.is_pinned
                if (data.reply_count !== undefined) msg.threadCount = data.reply_count
                if (data.edited_at !== undefined || data.edit_at !== undefined) {
                    msg.editedAt = toOptionalIsoTimestamp(data.edited_at ?? data.edit_at)
                }
                if (data.reply_count_inc) {
                    msg.threadCount = (msg.threadCount || 0) + data.reply_count_inc
                }
            }
        }

        // 2. Update in cached threads
        for (const rootId in repliesByThread.value) {
            const replies = repliesByThread.value[rootId]
            if (!replies) continue

            const index = replies.findIndex(m => m.id === data.id)
            if (index !== -1) {
                const msg = replies[index]
                if (!msg) continue

                if (data.message !== undefined) msg.content = data.message
                if (data.is_pinned !== undefined) msg.isPinned = data.is_pinned
                if (data.edited_at !== undefined || data.edit_at !== undefined) {
                    msg.editedAt = toOptionalIsoTimestamp(data.edited_at ?? data.edit_at)
                }
            }
        }
    }

    function handleMessageDelete(messageId: string) {
        // 1. Remove from main channels
        for (const cid in messagesByChannel.value) {
            const messages = messagesByChannel.value[cid]
            if (messages) {
                const index = messages.findIndex(m => m.id === messageId)
                if (index !== -1) {
                    messages.splice(index, 1)
                }
            }
        }

        // 2. Remove from cached threads
        for (const rootId in repliesByThread.value) {
            const replies = repliesByThread.value[rootId]
            if (replies) {
                const index = replies.findIndex(m => m.id === messageId)
                if (index !== -1) {
                    replies.splice(index, 1)
                }
            }
        }
    }

    function handleReactionAdded(data: any) {
        // data: { post_id, user_id, emoji_name, created_at, ... }
        // We need to find the post. It usually comes via broadcast which has channel_id in envelope context, 
        // but robustly the event data should contain channel_id OR we search? 
        // The previous implementation relied on context. 
        // Let's assume we can pass channelId to this function or it's in data.
        // My backend broadcast for reaction added DOES include channel_id in the *envelope*, but data itself is just Reaction struct.
        // Reaction struct doesn't have channelId. 
        // However, I updated `add_reaction` in backend to broadcast the reaction... wait. 
        // The `ws` handler passes `envelope.data`. If `envelope.channel_id` is set, `useWebSocket` could pass it.
        // `useWebSocket` calls `handleReactionAdded(envelope.data)`.
        // If data doesn't have channel_id, we are stuck unless we search.
        // Simplification: Search all channels or require channel_id.
        // I should have injected channel_id into the data payload or use envelope context.

        // Quick fix: loop through loaded channels to find post.
        for (const cid in messagesByChannel.value) {
            const messages = messagesByChannel.value[cid];
            if (!messages) continue;

            const msg = messages.find(m => m.id === data.post_id)
            if (msg) {
                // Found message
                const existingReaction = msg.reactions.find(r => r.emoji === data.emoji_name)
                if (existingReaction) {
                    if (!existingReaction.users.includes(data.user_id)) {
                        existingReaction.users.push(data.user_id)
                        existingReaction.count++
                    }
                } else {
                    msg.reactions.push({
                        emoji: data.emoji_name,
                        count: 1,
                        users: [data.user_id]
                    })
                }
                return
            }
        }
    }

    function handleReactionRemoved(data: any) {
        for (const cid in messagesByChannel.value) {
            const messages = messagesByChannel.value[cid];
            if (!messages) continue;

            const msg = messages.find(m => m.id === data.post_id)
            if (msg) {
                const index = msg.reactions.findIndex(r => r.emoji === data.emoji_name)
                if (index !== -1) {
                    const reaction = msg.reactions[index]
                    if (!reaction) continue;

                    const userIndex = reaction.users.indexOf(data.user_id)
                    if (userIndex !== -1) {
                        reaction.users.splice(userIndex, 1)
                        reaction.count--
                        if (reaction.count <= 0) {
                            msg.reactions.splice(index, 1)
                        }
                    }
                }
                return
            }
        }
    }

    return {
        messagesByChannel,
        repliesByThread,
        loading,
        isLoadingOlder,
        error,
        getMessages,
        fetchMessages,
        fetchThread,
        addOptimisticMessage,
        updateOptimisticMessage,
        handleNewMessage,
        handleMessageUpdate,
        handleMessageDelete,
        handleReactionAdded,
        handleReactionRemoved,
        clearMessages,
        pinMessage,
        unpinMessage,
        saveMessage,
        unsaveMessage,
        fetchSavedMessages,
        fetchPinnedMessages,
        searchMessages,
        hasMoreOlder,
        fetchOlderMessages,
    }
})
