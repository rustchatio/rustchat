import { ref } from 'vue'
import { useAuthStore } from '../stores/auth'
import { useMessageStore, postToMessage } from '../stores/messages'
import { usePresenceStore } from '../features/presence'
import type { PresenceStatus } from '../core/entities/User'
import { useUnreadStore } from '../stores/unreads'
import { useChannelStore } from '../stores/channels'
import { useToast } from './useToast'
import { postsApi, type ChannelUnreadAt, type Post } from '../api/posts'
import type { Channel } from '../api/channels'
import { normalizeEntityId, normalizeIdsDeep } from '../utils/idCompat'
import { applyUserStatusSnapshot } from './useUserSummary'

// Server -> Client
export interface WsEnvelope {
    type: 'event' | 'response' | 'error' | 'ack'
    event: string
    seq?: number
    channel_id?: string
    broadcast?: {
        channel_id?: string
        team_id?: string
        user_id?: string
    }
    data: any
}

// Client -> Server
export interface ClientEnvelope {
    type: 'command'
    event: string
    data: any
    channel_id?: string
    client_msg_id?: string
    seq?: number
}

// Singleton state
const ws = ref<WebSocket | null>(null)
const connected = ref(false)
const reconnectAttempts = ref(0)
const maxReconnectAttempts = 10
const subscriptions = ref<Set<string>>(new Set())
const listeners = ref<Record<string, Set<(data: any) => void>>>({})
let actionSeq = 1
let reconnectTimer: ReturnType<typeof setTimeout> | null = null

function clearReconnectTimer() {
    if (reconnectTimer) {
        clearTimeout(reconnectTimer)
        reconnectTimer = null
    }
}

function isAuthExpiryCloseEvent(event: CloseEvent): boolean {
    const reason = (event.reason || '').toLowerCase()
    return (
        (event.code === 1008 && reason.includes('token')) ||
        reason.includes('authentication token expired') ||
        reason.includes('token expired')
    )
}

function normalizeWsTimestamp(value: unknown, fallback: string): string {
    if (typeof value === 'number' && Number.isFinite(value)) {
        return new Date(value).toISOString()
    }
    if (typeof value === 'string' && value.length > 0) {
        return value
    }
    return fallback
}

function extractWsPostPayload(data: any): Record<string, any> | null {
    if (!data || typeof data !== 'object') {
        return null
    }

    if ('post' in data) {
        const wrappedPost = (data as Record<string, any>).post
        if (typeof wrappedPost === 'string') {
            try {
                const parsed = JSON.parse(wrappedPost)
                return parsed && typeof parsed === 'object' ? parsed : null
            } catch {
                return null
            }
        }
        if (wrappedPost && typeof wrappedPost === 'object') {
            return wrappedPost as Record<string, any>
        }
        return null
    }

    return data as Record<string, any>
}

function normalizeWsPost(data: any, envelopeChannelId?: string): Post | null {
    const rawPost = extractWsPostPayload(data)
    if (!rawPost || typeof rawPost.id !== 'string') {
        return null
    }
    const normalizedRawPost = normalizeIdsDeep(rawPost) as Record<string, any>

    const fallbackTimestamp = new Date().toISOString()
    const createdAt = normalizeWsTimestamp(normalizedRawPost.created_at ?? normalizedRawPost.create_at, fallbackTimestamp)
    const updatedAt = normalizeWsTimestamp(normalizedRawPost.updated_at ?? normalizedRawPost.update_at, createdAt)
    const postId = normalizeEntityId(normalizedRawPost.id) ?? normalizedRawPost.id
    const channelId = normalizeEntityId(normalizedRawPost.channel_id)
        ?? normalizeEntityId(envelopeChannelId)
        ?? normalizedRawPost.channel_id
        ?? envelopeChannelId
    const userId = normalizeEntityId(normalizedRawPost.user_id) ?? normalizedRawPost.user_id
    const rootPostId = normalizeEntityId(normalizedRawPost.root_post_id ?? normalizedRawPost.root_id)
        ?? normalizedRawPost.root_post_id
        ?? normalizedRawPost.root_id

    return {
        ...normalizedRawPost,
        id: postId,
        channel_id: channelId,
        user_id: userId,
        message: typeof normalizedRawPost.message === 'string' ? normalizedRawPost.message : '',
        root_post_id: rootPostId,
        root_id: rootPostId,
        created_at: createdAt,
        updated_at: updatedAt,
        client_msg_id: normalizedRawPost.client_msg_id ?? normalizedRawPost.pending_post_id,
        file_ids: Array.isArray(normalizedRawPost.file_ids)
            ? normalizedRawPost.file_ids.map((id: unknown) => normalizeEntityId(id) ?? id)
            : [],
        is_pinned: typeof normalizedRawPost.is_pinned === 'boolean' ? normalizedRawPost.is_pinned : false,
        seq: normalizedRawPost.seq ?? 0,
    } as unknown as Post
}

function normalizeWsEnvelope(envelope: WsEnvelope): WsEnvelope {
    const broadcastChannelId = normalizeEntityId(envelope.broadcast?.channel_id) ?? envelope.broadcast?.channel_id
    const channelId = normalizeEntityId(envelope.channel_id) ?? envelope.channel_id ?? broadcastChannelId

    return {
        ...envelope,
        channel_id: channelId,
        data: normalizeIdsDeep(envelope.data),
        broadcast: envelope.broadcast
            ? {
                ...envelope.broadcast,
                channel_id: broadcastChannelId,
                team_id: normalizeEntityId(envelope.broadcast.team_id) ?? envelope.broadcast.team_id,
                user_id: normalizeEntityId(envelope.broadcast.user_id) ?? envelope.broadcast.user_id,
            }
            : envelope.broadcast,
    }
}

function normalizeWsReactionPayload(data: any): Record<string, any> | null {
    if (!data || typeof data !== 'object') {
        return null
    }

    let rawReaction: unknown = data
    if ('reaction' in data) {
        rawReaction = (data as Record<string, unknown>).reaction
    }

    if (typeof rawReaction === 'string') {
        try {
            rawReaction = JSON.parse(rawReaction)
        } catch {
            return null
        }
    }

    if (!rawReaction || typeof rawReaction !== 'object') {
        return null
    }

    const normalized = normalizeIdsDeep(rawReaction) as Record<string, any>
    const emojiName = typeof normalized.emoji_name === 'string'
        ? normalized.emoji_name
        : typeof normalized.emoji === 'string'
            ? normalized.emoji
            : undefined

    if (!emojiName) {
        return null
    }

    return {
        ...normalized,
        post_id: normalizeEntityId(normalized.post_id) ?? normalized.post_id,
        user_id: normalizeEntityId(normalized.user_id) ?? normalized.user_id,
        emoji_name: emojiName,
    }
}

function normalizeWsChannelType(value: unknown): Channel['channel_type'] {
    const raw = String(value || '').toLowerCase()
    if (raw === 'o' || raw === 'public') return 'public'
    if (raw === 'p' || raw === 'private') return 'private'
    if (raw === 'd' || raw === 'direct') return 'direct'
    if (raw === 'g' || raw === 'group') return 'group'
    return 'public'
}

function normalizeWsChannelPayload(data: any): Channel | null {
    if (!data || typeof data !== 'object') {
        return null
    }

    const id = normalizeEntityId(data.id) ?? data.id
    const teamId = normalizeEntityId(data.team_id) ?? data.team_id
    if (typeof id !== 'string' || !id || typeof teamId !== 'string' || !teamId) {
        return null
    }

    const displayName = typeof data.display_name === 'string' && data.display_name
        ? data.display_name
        : (typeof data.name === 'string' ? data.name : '')
    const createdAtRaw = data.created_at ?? data.create_at

    return {
        id,
        team_id: teamId,
        name: typeof data.name === 'string' ? data.name : '',
        display_name: displayName,
        channel_type: normalizeWsChannelType(data.channel_type ?? data.type),
        header: typeof data.header === 'string' ? data.header : '',
        purpose: typeof data.purpose === 'string' ? data.purpose : '',
        created_at: normalizeWsTimestamp(createdAtRaw, new Date().toISOString()),
        creator_id: normalizeEntityId(data.creator_id) ?? data.creator_id ?? '',
        unreadCount: typeof data.unreadCount === 'number' ? data.unreadCount : 0,
        mentionCount: typeof data.mentionCount === 'number' ? data.mentionCount : 0,
    }
}

function normalizeWsPresence(value: unknown): PresenceStatus {
    const raw = String(value || '').toLowerCase()
    if (raw === 'online' || raw === 'away' || raw === 'dnd' || raw === 'offline') {
        return raw
    }
    return 'offline'
}

export function useWebSocket() {
    const authStore = useAuthStore()
    const messageStore = useMessageStore()
    const presenceStore = usePresenceStore()
    const unreadStore = useUnreadStore()
    const channelStore = useChannelStore()
    const toast = useToast()

    function applyInitialLoadSnapshot(data: any) {
        if (!data || typeof data !== 'object') {
            return
        }

        if (Array.isArray(data.channels)) {
            data.channels.forEach((rawChannel: any) => {
                const channel = normalizeWsChannelPayload(rawChannel)
                if (channel) {
                    channelStore.addChannel(channel)
                }
            })
        }

        if (Array.isArray(data.channel_unreads)) {
            data.channel_unreads.forEach((rawUnread: any) => {
                const channelId = normalizeEntityId(rawUnread.channel_id) ?? rawUnread.channel_id
                if (typeof channelId !== 'string' || !channelId) {
                    return
                }

                const unreadCount = Number(rawUnread.msg_count ?? rawUnread.unread_count ?? 0)
                const mentionCount = Number(rawUnread.mention_count ?? 0)
                unreadStore.channelUnreads[channelId] = Number.isFinite(unreadCount) ? unreadCount : 0
                unreadStore.channelMentions[channelId] = Number.isFinite(mentionCount) ? mentionCount : 0
            })
        }

        if (Array.isArray(data.statuses)) {
            data.statuses.forEach((rawStatus: any) => {
                const userId = normalizeEntityId(rawStatus.user_id) ?? rawStatus.user_id
                const status = normalizeWsPresence(rawStatus.status)
                if (typeof userId === 'string' && userId) {
                    presenceStore.updatePresenceFromEvent(userId, status)
                    applyUserStatusSnapshot({
                        userId,
                        presence: status,
                        statusText: rawStatus.text ?? null,
                        statusEmoji: rawStatus.emoji ?? null,
                        statusExpiresAt: rawStatus.expires_at ?? null,
                    })
                }
            })
        }
    }


    function connect() {
        if (!authStore.token) {
            console.log('No auth token, skipping WebSocket connection')
            return
        }

        if (ws.value?.readyState === WebSocket.OPEN) return;

        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
        const host = window.location.host
        // Align with Mattermost mobile websocket endpoint semantics.
        const url = `${protocol}//${host}/api/v4/websocket`

        try {
            // Browsers can't set Authorization headers for WebSocket handshakes.
            // Pass bearer token via Sec-WebSocket-Protocol.
            const socket = new WebSocket(url, [authStore.token])
            ws.value = socket

            socket.onopen = () => {
                const openedAfterReconnect = reconnectAttempts.value > 0
                console.log('WebSocket connected')
                connected.value = true
                reconnectAttempts.value = 0
                clearReconnectTimer()

                // Resubscribe to channels
                subscriptions.value.forEach(cid => {
                    send({
                        type: 'command',
                        event: 'subscribe_channel',
                        channel_id: cid,
                        data: {}
                    })
                })

                // Trigger resync for current channel if needed
                if (channelStore.currentChannelId) {
                    // Implement resync logic here or notify store
                    // For now, simpler to just refetch messages if connection was lost for a while
                    // or rely on 'after' cursor fetch.
                    messageStore.fetchMessages(channelStore.currentChannelId)
                }

                if (openedAfterReconnect) {
                    // Non-reliable WS clients must explicitly request a reconnect snapshot.
                    sendAction('reconnect', {})
                }
            }

            socket.onclose = (event) => {
                console.log('WebSocket disconnected', event.code, event.reason)
                connected.value = false
                ws.value = null

                if (isAuthExpiryCloseEvent(event)) {
                    clearReconnectTimer()
                    reconnectAttempts.value = 0
                    void authStore.logout('expired')
                    return
                }

                if (!authStore.token) {
                    clearReconnectTimer()
                    reconnectAttempts.value = 0
                    return
                }

                // Attempt to reconnect
                if (reconnectAttempts.value < maxReconnectAttempts) {
                    reconnectAttempts.value++
                    // Exponential backoff with jitter
                    const baseDelay = Math.min(1000 * Math.pow(1.5, reconnectAttempts.value), 30000)
                    const jitter = Math.random() * 1000
                    const delay = baseDelay + jitter

                    console.log(`Reconnecting in ${Math.round(delay)}ms...`)
                    clearReconnectTimer()
                    reconnectTimer = setTimeout(() => {
                        if (!connected.value) connect()
                    }, delay)
                }
            }

            socket.onerror = (error) => {
                console.error('WebSocket connection failed:', error)
                toast.error('Real-time connection error', 'The connection to the server was refused. Please check your network.')
            }

            socket.onmessage = (event) => {
                try {
                    const rawEnvelope: WsEnvelope = JSON.parse(event.data)
                    const envelope = normalizeWsEnvelope(rawEnvelope)
                    handleMessage(envelope)
                } catch (e) {
                    console.error('Failed to parse WebSocket message:', e)
                }
            }
        } catch (e) {
            console.error('Failed to create WebSocket:', e)
        }
    }

    function handleMessage(envelope: WsEnvelope) {
        // console.log('WS Received:', envelope.event, envelope.data)

        switch (envelope.event) {
            case 'hello':
                console.log('WebSocket hello received', envelope.data)
                break

            case 'initial_load':
                applyInitialLoadSnapshot(envelope.data)
                break

            case 'posted':
            case 'message_created':
            case 'post_created': // Fallback
            case 'message_posted':
            // Mattermost standard
            case 'thread_reply_created': {
                const post = normalizeWsPost(envelope.data, envelope.channel_id)
                if (!post) {
                    break
                }
                // If it's a thread reply, logic might slightly differ (handled by store)
                messageStore.handleNewMessage(post)

                // Notifications handling (counters are handled by unread_counts_updated)
                if (post.channel_id !== channelStore.currentChannelId && post.user_id !== authStore.user?.id) {
                    const mentionsUser = post.message?.includes(`@${authStore.user?.username}`) || false

                    if (mentionsUser) {
                        const channel = channelStore.channels.find(c => c.id === post.channel_id)
                        const title = channel ? `#${channel.name}` : 'New Mention'

                        if (Notification.permission === 'granted') {
                            new Notification(title, { body: post.message })
                        } else if (Notification.permission !== 'denied') {
                            Notification.requestPermission().then(p => {
                                if (p === 'granted') {
                                    new Notification(title, { body: post.message })
                                }
                            })
                        }
                    }
                }
                break
            }

            case 'message_updated':
            case 'post_edited': // Fallback
            case 'thread_reply_updated':
                if (envelope.data.id) {
                    // Partial update or full post?
                    // Backend sends { id, reply_count_inc, last_reply_at } for thread updates
                    // Or full post for edits.
                    // Store needs to handle both.
                    messageStore.handleMessageUpdate(envelope.data)
                }
                break

            case 'message_deleted':
            case 'post_deleted': // Fallback
            case 'thread_reply_deleted':
                if (envelope.data && (envelope.data.post_id || envelope.data.id)) {
                    messageStore.handleMessageDelete(envelope.data.post_id || envelope.data.id)
                }
                break

            case 'reaction_added':
                {
                    const reaction = normalizeWsReactionPayload(envelope.data)
                    if (reaction) {
                        messageStore.handleReactionAdded(reaction)
                    }
                }
                break

            case 'reaction_removed':
                {
                    const reaction = normalizeWsReactionPayload(envelope.data)
                    if (reaction) {
                        messageStore.handleReactionRemoved(reaction)
                    }
                }
                break

            case 'user_typing':
            case 'typing': // Compatibility with some mobile clients
                if (envelope.data) {
                    const typingChannelId = envelope.channel_id || envelope.broadcast?.channel_id || envelope.data.channel_id
                    if (!typingChannelId) {
                        break
                    }
                    presenceStore.addTypingUser(
                        envelope.data.user_id,
                        envelope.data.display_name || envelope.data.username || 'Someone',
                        typingChannelId,
                        envelope.data.thread_root_id
                    )
                }
                break

            case 'user_typing_stop':
            case 'stop_typing':
                if (envelope.data) {
                    const typingChannelId = envelope.channel_id || envelope.broadcast?.channel_id || envelope.data.channel_id
                    if (!typingChannelId) {
                        break
                    }
                    presenceStore.removeTypingUser(
                        envelope.data.user_id,
                        typingChannelId,
                        envelope.data.thread_root_id
                    )
                }
                break

            case 'user_presence':
                if (envelope.data) {
                    presenceStore.updatePresenceFromEvent(
                        envelope.data.user_id,
                        envelope.data.status || 'online'
                    )
                }
                break

            case 'status_change':
                if (envelope.data) {
                    presenceStore.updatePresenceFromEvent(
                        envelope.data.user_id,
                        envelope.data.status || 'online'
                    )
                    applyUserStatusSnapshot({
                        userId: envelope.data.user_id,
                        presence: envelope.data.status,
                        statusText: envelope.data.text ?? null,
                        statusEmoji: envelope.data.emoji ?? null,
                        statusExpiresAt: envelope.data.expires_at ?? null,
                    })

                    if (envelope.data.user_id === authStore.user?.id) {
                        authStore.syncUserStatusSnapshot({
                            status: envelope.data.status,
                            text: envelope.data.text ?? null,
                            emoji: envelope.data.emoji ?? null,
                            expiresAt: envelope.data.expires_at ?? null,
                        })
                    }
                }
                break

            case 'channel_created': {
                if (envelope.data) {
                    const channel = normalizeWsChannelPayload(envelope.data)
                    if (channel) {
                        channelStore.addChannel(channel)
                    }
                }
                break
            }

            case 'unread_counts_updated': {
                if (envelope.data) {
                    unreadStore.handleUnreadUpdate(envelope.data)
                }
                break
            }

            case 'post_unread': {
                if (envelope.data) {
                    const payload = normalizeIdsDeep(envelope.data) as ChannelUnreadAt
                    unreadStore.applyPostUnread(payload)
                }
                break
            }

            case 'error':
                console.error('WS Error:', envelope.data)
                break
        }

        // Notify listeners
        const eventListeners = listeners.value[envelope.event]
        if (eventListeners) {
            eventListeners.forEach(cb => cb(envelope.data))
        }
    }

    function disconnect() {
        clearReconnectTimer()
        if (ws.value) {
            ws.value.close()
            ws.value = null
        }
        connected.value = false
        reconnectAttempts.value = 0
        subscriptions.value.clear()
    }

    function send(envelope: ClientEnvelope) {
        if (ws.value && connected.value) {
            ws.value.send(JSON.stringify(envelope))
        }
    }

    function sendAction(action: string, data: Record<string, unknown>) {
        if (ws.value && connected.value) {
            ws.value.send(JSON.stringify({
                action,
                seq: actionSeq++,
                data,
            }))
        }
    }

    function subscribe(channelId: string) {
        if (!subscriptions.value.has(channelId)) {
            subscriptions.value.add(channelId)
            send({
                type: 'command',
                event: 'subscribe_channel',
                channel_id: channelId,
                data: {}
            })
        }
    }

    function unsubscribe(channelId: string) {
        if (subscriptions.value.has(channelId)) {
            subscriptions.value.delete(channelId)
            send({
                type: 'command',
                event: 'unsubscribe_channel',
                channel_id: channelId,
                data: {}
            })
        }
    }

    function sendTyping(channelId: string, threadRootId?: string) {
        // Match Mattermost web/mobile typing command format.
        sendAction('user_typing', {
            channel_id: channelId,
            parent_id: threadRootId,
        })
    }

    function sendStopTyping(channelId: string, threadRootId?: string) {
        sendAction('user_typing_stop', {
            channel_id: channelId,
            parent_id: threadRootId,
        })
    }

    async function sendMessage(channelId: string, content: string, rootId?: string, fileIds: string[] = []) {
        const clientMsgId = crypto.randomUUID()
        const authStore = useAuthStore()
        const messageStore = useMessageStore()

        // Create temp message for optimistic UI
        const tempMsg: any = {
            id: clientMsgId,
            channelId,
            userId: authStore.user?.id || '',
            username: authStore.user?.username || 'Me',
            avatarUrl: authStore.user?.avatar_url,
            content,
            timestamp: new Date().toISOString(),
            reactions: [],
            files: [],
            isPinned: false,
            isSaved: false,
            status: 'sending',
            clientMsgId,
            rootId
        }

        messageStore.addOptimisticMessage(tempMsg)

        try {
            const { data: post } = await postsApi.create({
                channel_id: channelId,
                message: content,
                root_post_id: rootId,
                file_ids: fileIds,
                client_msg_id: clientMsgId
            })

            // Convert server Post to frontend Message using store helper
            const finalMsg = postToMessage(post)
            messageStore.updateOptimisticMessage(clientMsgId, finalMsg)
        } catch (error) {
            console.error('Failed to send message via REST:', error)
            // Ideally we'd have a store method to mark as failed
            const msg = (messageStore.messagesByChannel[channelId] || []).find(m => m.id === clientMsgId)
            if (msg) msg.status = 'failed'
        }
    }

    function sendPresence(status: string) {
        send({
            type: 'command',
            event: 'presence',
            data: { status }
        })
    }

    function onEvent(event: string, callback: (data: any) => void) {
        if (!listeners.value[event]) {
            listeners.value[event] = new Set()
        }
        listeners.value[event].add(callback)
    }

    function offEvent(event: string, callback: (data: any) => void) {
        if (listeners.value[event]) {
            listeners.value[event].delete(callback)
        }
    }

    return {
        connected,
        connect,
        disconnect,
        subscribe,
        unsubscribe,
        sendTyping,
        sendStopTyping,
        sendMessage,
        sendPresence,
        onEvent,
        offEvent,
    }
}
