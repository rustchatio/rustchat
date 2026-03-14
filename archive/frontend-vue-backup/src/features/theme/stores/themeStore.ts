// Theme Store - Pure state management for appearance

import { defineStore } from 'pinia'
import { ref, computed, readonly } from 'vue'
import type { Theme, ChatFont, ChatFontSize } from '../types'
import { DARK_THEMES } from '../types'

export const useThemeStore = defineStore('themeStore', () => {
  // State
  const theme = ref<Theme>('light')
  const font = ref<ChatFont>('inter')
  const fontSize = ref<ChatFontSize>(14)
  const syncedServerToken = ref<string | null>(null)

  // Getters
  const isDark = computed(() => DARK_THEMES.has(theme.value))

  // Actions
  function setTheme(value: Theme, persist = true) {
    theme.value = value
    if (persist) {
      // Theme is persisted by service
    }
  }

  function setFont(value: ChatFont, persist = true) {
    font.value = value
    if (persist) {
      // Font is persisted by service
    }
  }

  function setFontSize(value: ChatFontSize, persist = true) {
    fontSize.value = value
    if (persist) {
      // Font size is persisted by service
    }
  }

  function setSyncedServerToken(token: string | null) {
    syncedServerToken.value = token
  }

  return {
    // State (readonly)
    theme: readonly(theme),
    font: readonly(font),
    fontSize: readonly(fontSize),
    syncedServerToken: readonly(syncedServerToken),

    // Getters
    isDark,

    // Actions
    setTheme,
    setFont,
    setFontSize,
    setSyncedServerToken
  }
})
