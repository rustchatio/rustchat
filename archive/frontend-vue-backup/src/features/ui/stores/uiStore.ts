// UI Store - Pure state management for UI state

import { defineStore } from 'pinia'
import { ref, readonly } from 'vue'

export type RhsView = 'thread' | 'search' | 'info' | 'saved' | 'pinned' | 'members' | null
export type Density = 'comfortable' | 'compact'

export const useUIStore = defineStore('uiStore', () => {
  // State
  const isRhsOpen = ref(false)
  const isSettingsOpen = ref(false)
  const rhsView = ref<RhsView>(null)
  const rhsContextId = ref<string | null>(null)
  const videoCallUrl = ref<string | null>(null)
  const isVideoCallOpen = ref(false)
  const density = ref<Density>('comfortable')

  // Actions
  function openSettings() {
    isSettingsOpen.value = true
  }

  function closeSettings() {
    isSettingsOpen.value = false
  }

  function openRhs(view: RhsView, contextId?: string) {
    rhsView.value = view
    rhsContextId.value = contextId || null
    isRhsOpen.value = true
  }

  function closeRhs() {
    isRhsOpen.value = false
    rhsView.value = null
    rhsContextId.value = null
  }

  function toggleRhs(view: RhsView) {
    if (isRhsOpen.value && rhsView.value === view) {
      closeRhs()
    } else {
      openRhs(view)
    }
  }

  function openVideoCall(url: string) {
    videoCallUrl.value = url
    isVideoCallOpen.value = true
  }

  function closeVideoCall() {
    isVideoCallOpen.value = false
    videoCallUrl.value = null
  }

  function setDensity(newDensity: Density) {
    density.value = newDensity
    localStorage.setItem('density', newDensity)
  }

  // Initialize from storage
  function initialize() {
    const savedDensity = localStorage.getItem('density') as Density
    if (savedDensity === 'comfortable' || savedDensity === 'compact') {
      density.value = savedDensity
    }
  }

  return {
    // State (readonly)
    isRhsOpen: readonly(isRhsOpen),
    isSettingsOpen: readonly(isSettingsOpen),
    rhsView: readonly(rhsView),
    rhsContextId: readonly(rhsContextId),
    videoCallUrl: readonly(videoCallUrl),
    isVideoCallOpen: readonly(isVideoCallOpen),
    density: readonly(density),

    // Actions
    openSettings,
    closeSettings,
    openRhs,
    closeRhs,
    toggleRhs,
    openVideoCall,
    closeVideoCall,
    setDensity,
    initialize
  }
})
