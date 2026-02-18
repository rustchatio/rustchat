// Message Store - Pure state management, no business logic
// Business logic is in messageService.ts

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Message, MessageId } from '../../../core/entities/Message'
import type { ChannelId } from '../../../core/entities/Channel'
import type { Reaction } from '../../../core/entities/Message'

export const useMessageStore = defineStore('messageStore', () => {
  // State
  const messagesByChannel = ref<Map<ChannelId, Message[]>>(new Map())
  const threadRepliesByRoot = ref<Map<MessageId, Message[]>>(new Map())
  const threadLoading = ref<Set<MessageId>>(new Set())
  const loading = ref(false)
  const loadingOlder = ref(false)
  const error = ref<string | null>(null)
  const hasMoreOlderByChannel = ref<Map<ChannelId, boolean>>(new Map())

  // Getters
  const getMessages = computed(() => (channelId: ChannelId) => {
    return messagesByChannel.value.get(channelId) || []
  })

  const getMessageById = computed(() => (channelId: ChannelId, id: MessageId) => {
    const messages = messagesByChannel.value.get(channelId)
    return messages?.find(m => m.id === id)
  })

  const findMessageByClientId = computed(() => (channelId: ChannelId, clientId: string) => {
    const messages = messagesByChannel.value.get(channelId)
    return messages?.find(m => m.clientId === clientId)
  })

  const getThreadReplies = computed(() => (rootId: MessageId) => {
    return threadRepliesByRoot.value.get(rootId) || []
  })

  const isThreadLoading = computed(() => (rootId: MessageId) => {
    return threadLoading.value.has(rootId)
  })

  const hasMoreOlder = computed(() => (channelId: ChannelId) => {
    return hasMoreOlderByChannel.value.get(channelId) ?? true
  })

  // Actions - Simple state mutations only
  function setMessages(channelId: ChannelId, messages: Message[]) {
    messagesByChannel.value.set(channelId, messages)
  }

  function prependMessages(channelId: ChannelId, messages: Message[]) {
    const existing = messagesByChannel.value.get(channelId) || []
    messagesByChannel.value.set(channelId, [...messages, ...existing])
  }

  function addMessage(channelId: ChannelId, message: Message) {
    const existing = messagesByChannel.value.get(channelId) || []
    messagesByChannel.value.set(channelId, [...existing, message])
  }

  function updateMessage(message: Message) {
    // Update in main channel
    const channelMessages = messagesByChannel.value.get(message.channelId)
    if (channelMessages) {
      const index = channelMessages.findIndex(m => m.id === message.id)
      if (index !== -1) {
        channelMessages[index] = message
      }
    }

    // Update in thread if it's a reply
    if (message.rootId) {
      const threadMessages = threadRepliesByRoot.value.get(message.rootId)
      if (threadMessages) {
        const index = threadMessages.findIndex(m => m.id === message.id)
        if (index !== -1) {
          threadMessages[index] = message
        }
      }
    }
  }

  function patchMessage(messageId: MessageId, updates: Partial<Message>) {
    // Search in all channels
    for (const [channelId, messages] of messagesByChannel.value) {
      const message = messages.find(m => m.id === messageId)
      if (message) {
        Object.assign(message, updates)
        return
      }
    }
  }

  function removeMessage(channelId: ChannelId, messageId: MessageId) {
    const messages = messagesByChannel.value.get(channelId)
    if (messages) {
      const index = messages.findIndex(m => m.id === messageId)
      if (index !== -1) {
        messages.splice(index, 1)
      }
    }
  }

  function replaceOptimisticMessage(
    channelId: ChannelId,
    clientId: string,
    message: Message
  ) {
    const messages = messagesByChannel.value.get(channelId)
    if (messages) {
      const index = messages.findIndex(m => m.clientId === clientId)
      if (index !== -1) {
        messages[index] = message
      }
    }
  }

  function markMessageFailed(channelId: ChannelId, clientId: string) {
    const messages = messagesByChannel.value.get(channelId)
    if (messages) {
      const message = messages.find(m => m.clientId === clientId)
      if (message) {
        message.status = 'failed'
      }
    }
  }

  // Thread actions
  function setThreadReplies(rootId: MessageId, replies: Message[]) {
    threadRepliesByRoot.value.set(rootId, replies)
  }

  function addThreadReply(rootId: MessageId, reply: Message) {
    const existing = threadRepliesByRoot.value.get(rootId) || []
    threadRepliesByRoot.value.set(rootId, [...existing, reply])
  }

  function replaceOptimisticThreadReply(
    rootId: MessageId,
    clientId: string,
    message: Message
  ) {
    const replies = threadRepliesByRoot.value.get(rootId)
    if (replies) {
      const index = replies.findIndex(m => m.clientId === clientId)
      if (index !== -1) {
        replies[index] = message
      }
    }
  }

  function setThreadLoading(rootId: MessageId, loading: boolean) {
    if (loading) {
      threadLoading.value.add(rootId)
    } else {
      threadLoading.value.delete(rootId)
    }
  }

  // Reaction actions
  function addReaction(messageId: MessageId, emoji: string, userId: string) {
    for (const messages of messagesByChannel.value.values()) {
      const message = messages.find(m => m.id === messageId)
      if (message) {
        const reaction = message.reactions.find(r => r.emoji === emoji)
        if (reaction) {
          if (!reaction.users.includes(userId)) {
            reaction.users.push(userId)
            reaction.count++
          }
        } else {
          message.reactions.push({ emoji, count: 1, users: [userId] })
        }
        return
      }
    }
  }

  function removeReaction(messageId: MessageId, emoji: string, userId: string) {
    for (const messages of messagesByChannel.value.values()) {
      const message = messages.find(m => m.id === messageId)
      if (message) {
        const index = message.reactions.findIndex(r => r.emoji === emoji)
        if (index !== -1) {
          const reaction = message.reactions[index]
          const userIndex = reaction.users.indexOf(userId)
          if (userIndex !== -1) {
            reaction.users.splice(userIndex, 1)
            reaction.count--
            if (reaction.count <= 0) {
              message.reactions.splice(index, 1)
            }
          }
        }
        return
      }
    }
  }

  function addOptimisticReaction(messageId: MessageId, emoji: string, userId: string) {
    addReaction(messageId, emoji, userId)
  }

  // Loading state
  function setLoading(value: boolean) {
    loading.value = value
  }

  function setLoadingOlder(value: boolean) {
    loadingOlder.value = value
  }

  function setHasMoreOlder(channelId: ChannelId, value: boolean) {
    hasMoreOlderByChannel.value.set(channelId, value)
  }

  function setError(err: string | null) {
    error.value = err
  }

  function clearError() {
    error.value = null
  }

  function clearChannel(channelId: ChannelId) {
    messagesByChannel.value.delete(channelId)
    hasMoreOlderByChannel.value.delete(channelId)
  }

  return {
    // State
    messagesByChannel,
    threadRepliesByRoot,
    loading,
    loadingOlder,
    error,
    
    // Getters
    getMessages,
    getMessageById,
    findMessageByClientId,
    getThreadReplies,
    isThreadLoading,
    hasMoreOlder,
    
    // Actions
    setMessages,
    prependMessages,
    addMessage,
    updateMessage,
    patchMessage,
    removeMessage,
    replaceOptimisticMessage,
    markMessageFailed,
    setThreadReplies,
    addThreadReply,
    replaceOptimisticThreadReply,
    setThreadLoading,
    addReaction,
    removeReaction,
    addOptimisticReaction,
    setLoading,
    setLoadingOlder,
    setHasMoreOlder,
    setError,
    clearError,
    clearChannel
  }
})
