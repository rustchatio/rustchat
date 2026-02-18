// Message Service - Orchestrates data flow between repository, store, and UI
// This is where business logic lives (optimistic updates, deduplication, etc)

import { messageRepository } from '../repositories/messageRepository'
import type { Message, MessageDraft, MessageId } from '../../../core/entities/Message'
import type { ChannelId } from '../../../core/entities/Channel'
import { useMessageStore } from '../stores/messageStore'
import { AppError } from '../../../core/errors/AppError'

class MessageService {
  private get store() {
    return useMessageStore()
  }

  // Load messages for a channel
  async loadMessages(channelId: ChannelId, options?: { before?: MessageId; limit?: number }) {
    this.store.setLoading(true)
    this.store.clearError()

    try {
      const { messages, readState } = await messageRepository.findByChannel(channelId, {
        before: options?.before,
        limit: options?.limit ?? 50
      })

      if (options?.before) {
        // Loading older messages - prepend
        this.store.prependMessages(channelId, messages)
        this.store.setHasMoreOlder(channelId, messages.length >= (options?.limit ?? 50))
      } else {
        // Initial load - replace
        this.store.setMessages(channelId, messages)
        this.store.setHasMoreOlder(channelId, messages.length >= 50)
      }

      return { messages, readState }
    } catch (error) {
      this.store.setError(error instanceof AppError ? error.message : 'Failed to load messages')
      throw error
    } finally {
      this.store.setLoading(false)
    }
  }

  async loadOlderMessages(channelId: ChannelId) {
    const messages = this.store.getMessages(channelId)
    if (messages.length === 0) {
      return this.loadMessages(channelId)
    }

    const oldestMessage = messages[0]
    if (!oldestMessage) return

    this.store.setLoadingOlder(true)
    
    try {
      await this.loadMessages(channelId, { before: oldestMessage.id })
    } finally {
      this.store.setLoadingOlder(false)
    }
  }

  async loadThread(rootId: MessageId) {
    this.store.setThreadLoading(rootId, true)
    
    try {
      const replies = await messageRepository.findThread(rootId)
      this.store.setThreadReplies(rootId, replies)
      return replies
    } finally {
      this.store.setThreadLoading(rootId, false)
    }
  }

  // Send message with optimistic update
  async sendMessage(draft: MessageDraft): Promise<Message> {
    const clientId = generateClientId()
    const optimisticMessage = this.createOptimisticMessage(draft, clientId)

    // 1. Optimistic update
    if (draft.rootId) {
      this.store.addThreadReply(draft.rootId, optimisticMessage)
    } else {
      this.store.addMessage(draft.channelId, optimisticMessage)
    }

    try {
      // 2. API call
      const message = await messageRepository.create({
        ...draft,
        props: { ...draft.props, client_msg_id: clientId }
      })

      // 3. Replace optimistic with real
      this.store.replaceOptimisticMessage(draft.channelId, clientId, message)
      if (draft.rootId) {
        this.store.replaceOptimisticThreadReply(draft.rootId, clientId, message)
      }

      return message
    } catch (error) {
      // 4. Mark as failed
      this.store.markMessageFailed(draft.channelId, clientId)
      throw error
    }
  }

  async editMessage(messageId: MessageId, newContent: string) {
    const message = await messageRepository.update(messageId, { content: newContent })
    this.store.updateMessage(message)
    return message
  }

  async deleteMessage(messageId: MessageId, channelId: ChannelId) {
    await messageRepository.delete(messageId)
    this.store.removeMessage(channelId, messageId)
  }

  async togglePin(messageId: MessageId, channelId: ChannelId, isPinned: boolean) {
    const message = await messageRepository.update(messageId, { isPinned })
    this.store.updateMessage(message)
  }

  async toggleSave(messageId: MessageId, isSaved: boolean) {
    if (isSaved) {
      await messageRepository.unsaveMessage(messageId)
    } else {
      await messageRepository.saveMessage(messageId)
    }
  }

  async addReaction(messageId: MessageId, emoji: string, userId: string) {
    // Optimistic
    this.store.addOptimisticReaction(messageId, emoji, userId)
    
    try {
      await messageRepository.addReaction(messageId, emoji)
    } catch (error) {
      // Rollback
      this.store.removeReaction(messageId, emoji, userId)
      throw error
    }
  }

  async removeReaction(messageId: MessageId, emoji: string, userId: string) {
    // Optimistic
    this.store.removeReaction(messageId, emoji, userId)
    
    try {
      await messageRepository.removeReaction(messageId, emoji)
    } catch (error) {
      // Rollback - add it back
      this.store.addOptimisticReaction(messageId, emoji, userId)
      throw error
    }
  }

  // Handle incoming WebSocket message
  handleIncomingMessage(message: Message) {
    // Deduplication: Check if we already have this message
    if (message.clientId) {
      const existing = this.store.findMessageByClientId(message.channelId, message.clientId)
      if (existing) {
        // Replace optimistic with server version
        this.store.replaceOptimisticMessage(message.channelId, message.clientId, message)
        return
      }
    }

    const existingById = this.store.getMessageById(message.channelId, message.id)
    if (existingById) {
      // Update existing
      this.store.updateMessage(message)
    } else {
      // New message
      if (message.rootId) {
        this.store.addThreadReply(message.rootId, message)
      } else {
        this.store.addMessage(message.channelId, message)
      }
    }
  }

  handleMessageUpdate(messageId: MessageId, updates: Partial<Message>) {
    this.store.patchMessage(messageId, updates)
  }

  handleMessageDelete(messageId: MessageId, channelId: ChannelId) {
    this.store.removeMessage(channelId, messageId)
  }

  handleReactionAdded(messageId: MessageId, emoji: string, userId: string) {
    this.store.addReaction(messageId, emoji, userId)
  }

  handleReactionRemoved(messageId: MessageId, emoji: string, userId: string) {
    this.store.removeReaction(messageId, emoji, userId)
  }

  private createOptimisticMessage(draft: MessageDraft, clientId: string): Message {
    const now = new Date()
    return {
      id: `temp-${clientId}`,
      channelId: draft.channelId,
      userId: 'me', // Will be resolved by auth store
      content: draft.content,
      rootId: draft.rootId,
      replyCount: 0,
      files: [],
      reactions: [],
      isPinned: false,
      isSaved: false,
      status: 'sending',
      clientId,
      createdAt: now,
      props: draft.props
    }
  }
}

function generateClientId(): string {
  return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
}

export const messageService = new MessageService()
