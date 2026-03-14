// Theme Repository - Data access for appearance preferences

import { withRetry } from '../../../core/services/retry'
import type { Theme, ChatFont, ChatFontSize } from '../types'

const SERVER_PREFERENCE_URL = '/api/v4/users/me/preferences'
const SERVER_PREFERENCE_CATEGORY = 'rustchat_display'
const SERVER_PREFERENCE_THEME = 'theme'
const SERVER_PREFERENCE_FONT = 'font'
const SERVER_PREFERENCE_FONT_SIZE = 'font_size'

interface ServerPreference {
  user_id: string
  category: string
  name: string
  value: string
}

export interface AppearancePreferences {
  theme?: Theme
  font?: ChatFont
  fontSize?: ChatFontSize
}

export const themeRepository = {
  // Persist appearance to server
  async saveToServer(
    theme: Theme,
    font: ChatFont,
    fontSize: ChatFontSize,
    token: string
  ): Promise<void> {
    if (!token) return

    const payload: ServerPreference[] = [
      {
        user_id: 'me',
        category: SERVER_PREFERENCE_CATEGORY,
        name: SERVER_PREFERENCE_THEME,
        value: theme
      },
      {
        user_id: 'me',
        category: SERVER_PREFERENCE_CATEGORY,
        name: SERVER_PREFERENCE_FONT,
        value: font
      },
      {
        user_id: 'me',
        category: SERVER_PREFERENCE_CATEGORY,
        name: SERVER_PREFERENCE_FONT_SIZE,
        value: String(fontSize)
      }
    ]

    await withRetry(() =>
      fetch(SERVER_PREFERENCE_URL, {
        method: 'PUT',
        headers: {
          Authorization: `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(payload)
      })
    )
  },

  // Load appearance from server
  async loadFromServer(token: string): Promise<AppearancePreferences> {
    return withRetry(async () => {
      const response = await fetch(SERVER_PREFERENCE_URL, {
        headers: { Authorization: `Bearer ${token}` }
      })

      if (!response.ok) {
        return {}
      }

      const rows: ServerPreference[] = await response.json()

      const getValue = (name: string): string | undefined =>
        rows.find(p => p.category === SERVER_PREFERENCE_CATEGORY && p.name === name)?.value

      const rawTheme = getValue(SERVER_PREFERENCE_THEME)
      const rawFont = getValue(SERVER_PREFERENCE_FONT)
      const rawFontSize = getValue(SERVER_PREFERENCE_FONT_SIZE)

      return {
        theme: rawTheme as Theme | undefined,
        font: rawFont as ChatFont | undefined,
        fontSize: rawFontSize ? (Number(rawFontSize) as ChatFontSize) : undefined
      }
    })
  }
}
