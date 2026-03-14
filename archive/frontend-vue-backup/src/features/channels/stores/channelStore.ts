// Channel Store - Pure state management for channels
// No business logic - just state and simple mutations

import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'
import type { Channel, ChannelId, ChannelType } from '../../../core/entities/Channel'
import type { TeamId } from '../../../core/entities/Team'
import type { ChannelUnreadCounts } from '../repositories/channelRepository'

export const useChannelStore = defineStore('channelStore', () => {
  // State
  const channels = ref<Map<ChannelId, Channel>>(new Map())
  const joinableChannels = ref<Channel[]>([])
  const currentChannelId = ref<ChannelId | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const allChannels = computed(() => Array.from(channels.value.values()))

  const currentChannel = computed(() => {
    if (!currentChannelId.value) return null
    return channels.value.get(currentChannelId.value) || null
  })

  const publicChannels = computed(() => 
    allChannels.value.filter(c => c.type === 'public')
  )

  const privateChannels = computed(() => 
    allChannels.value.filter(c => c.type === 'private')
  )

  const directMessages = computed(() => 
    allChannels.value.filter(c => c.type === 'direct' || c.type === 'group')
  )

  const channelsByTeam = computed(() => (teamId: TeamId) => 
    allChannels.value.filter(c => c.teamId === teamId)
  )

  function getChannelById(channelId: ChannelId): Channel | undefined {
    return channels.value.get(channelId)
  }

  function getChannelsByType(type: ChannelType): Channel[] {
    return allChannels.value.filter(c => c.type === type)
  }

  // Actions - Simple state mutations only
  function setChannels(items: Channel[]) {
    channels.value.clear()
    for (const channel of items) {
      channels.value.set(channel.id, channel)
    }
  }

  function addChannel(channel: Channel) {
    channels.value.set(channel.id, channel)
  }

  function updateChannel(channel: Channel) {
    const existing = channels.value.get(channel.id)
    if (existing) {
      channels.value.set(channel.id, { ...existing, ...channel })
    }
  }

  function removeChannel(channelId: ChannelId) {
    channels.value.delete(channelId)
    
    // If we removed the current channel, clear it
    if (currentChannelId.value === channelId) {
      currentChannelId.value = null
    }
  }

  function setCurrentChannelId(channelId: ChannelId | null) {
    currentChannelId.value = channelId
  }

  function setJoinableChannels(items: Channel[]) {
    joinableChannels.value = items
  }

  // Unread/mention counts
  function setUnreadCounts(counts: ChannelUnreadCounts[]) {
    for (const { channelId, unreadCount, mentionCount } of counts) {
      const channel = channels.value.get(channelId)
      if (channel) {
        channel.unreadCount = unreadCount
        channel.mentionCount = mentionCount
      }
    }
  }

  function incrementUnread(channelId: ChannelId) {
    const channel = channels.value.get(channelId)
    if (channel) {
      channel.unreadCount = (channel.unreadCount || 0) + 1
    }
  }

  function incrementMention(channelId: ChannelId) {
    const channel = channels.value.get(channelId)
    if (channel) {
      channel.mentionCount = (channel.mentionCount || 0) + 1
    }
  }

  function clearCounts(channelId: ChannelId) {
    const channel = channels.value.get(channelId)
    if (channel) {
      channel.unreadCount = 0
      channel.mentionCount = 0
    }
  }

  // Loading state
  function setLoading(value: boolean) {
    loading.value = value
  }

  function setError(err: string | null) {
    error.value = err
  }

  function clearError() {
    error.value = null
  }

  function clearChannels() {
    channels.value.clear()
    currentChannelId.value = null
    joinableChannels.value = []
  }

  return {
    // State (readonly)
    channels: readonly(channels),
    joinableChannels: readonly(joinableChannels),
    currentChannelId: readonly(currentChannelId),
    loading: readonly(loading),
    error: readonly(error),

    // Getters
    allChannels,
    currentChannel,
    publicChannels,
    privateChannels,
    directMessages,
    channelsByTeam,
    getChannelById,
    getChannelsByType,

    // Actions
    setChannels,
    addChannel,
    updateChannel,
    removeChannel,
    setCurrentChannelId,
    setJoinableChannels,
    setUnreadCounts,
    incrementUnread,
    incrementMention,
    clearCounts,
    setLoading,
    setError,
    clearError,
    clearChannels
  }
})
