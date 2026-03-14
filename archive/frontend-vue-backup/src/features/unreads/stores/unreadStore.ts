// Unread Store - Pure state management for unread counts

import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'
import type { ChannelId } from '../../../core/entities/Channel'
import type { TeamId } from '../../../core/entities/Team'
import type { ReadState } from '../repositories/unreadRepository'

export const useUnreadStore = defineStore('unreadStore', () => {
  // State
  const channelUnreads = ref<Map<ChannelId, number>>(new Map())
  const channelMentions = ref<Map<ChannelId, number>>(new Map())
  const teamUnreads = ref<Map<TeamId, number>>(new Map())
  const channelReadStates = ref<Map<ChannelId, ReadState>>(new Map())
  const loading = ref(false)

  // Getters
  const totalUnreadCount = computed(() => {
    let total = 0
    for (const count of channelUnreads.value.values()) {
      total += count
    }
    return total
  })

  const totalMentionCount = computed(() => {
    let total = 0
    for (const count of channelMentions.value.values()) {
      total += count
    }
    return total
  })

  // Actions
  function getChannelUnread(channelId: ChannelId): number {
    return channelUnreads.value.get(channelId) || 0
  }

  function getChannelMentions(channelId: ChannelId): number {
    return channelMentions.value.get(channelId) || 0
  }

  function getTeamUnread(teamId: TeamId): number {
    return teamUnreads.value.get(teamId) || 0
  }

  function getChannelReadState(channelId: ChannelId): ReadState | undefined {
    return channelReadStates.value.get(channelId)
  }

  function setChannelUnread(channelId: ChannelId, count: number) {
    channelUnreads.value.set(channelId, count)
  }

  function setChannelMentions(channelId: ChannelId, count: number) {
    channelMentions.value.set(channelId, count)
  }

  function setTeamUnread(teamId: TeamId, count: number) {
    teamUnreads.value.set(teamId, count)
  }

  function setReadState(channelId: ChannelId, state: ReadState) {
    channelReadStates.value.set(channelId, state)
  }

  function clearChannel(channelId: ChannelId) {
    channelUnreads.value.delete(channelId)
    channelMentions.value.delete(channelId)
    channelReadStates.value.delete(channelId)
  }

  function clearAll() {
    channelUnreads.value.clear()
    channelMentions.value.clear()
    teamUnreads.value.clear()
    channelReadStates.value.clear()
  }

  function setLoading(value: boolean) {
    loading.value = value
  }

  return {
    // State (readonly)
    channelUnreads: readonly(channelUnreads),
    channelMentions: readonly(channelMentions),
    teamUnreads: readonly(teamUnreads),
    channelReadStates: readonly(channelReadStates),
    loading: readonly(loading),

    // Getters
    totalUnreadCount,
    totalMentionCount,

    // Actions
    getChannelUnread,
    getChannelMentions,
    getTeamUnread,
    getChannelReadState,
    setChannelUnread,
    setChannelMentions,
    setTeamUnread,
    setReadState,
    clearChannel,
    clearAll,
    setLoading
  }
})
