import { defineStore } from 'pinia'
import { ref } from 'vue'

export type Theme =
    | 'light'
    | 'dark'
    | 'modern'
    | 'metallic'
    | 'futuristic'
    | 'high-contrast'
    | 'simple'
    | 'dynamic'

export interface ThemeColors {
    sidebarBg: string
    sidebarText: string
    centerChannelBg: string
    centerChannelColor: string
    linkColor: string
    buttonBg: string
    buttonColor: string
}

export type ChatFont =
    | 'inter'
    | 'figtree'
    | 'jetbrains-mono'
    | 'quicksand'
    | 'montserrat'
    | 'source-sans-3'
    | 'nunito'
    | 'manrope'
    | 'work-sans'
    | 'ibm-plex-sans'

export type ChatFontSize = 13 | 14 | 16 | 18 | 20

export const THEME_OPTIONS: Array<{
    id: Theme
    label: string
    swatches: { primary: string; accent: string; background: string }
    colors: ThemeColors
}> = [
    { 
        id: 'light', 
        label: 'Light', 
        swatches: { primary: '#2563eb', accent: '#0ea5e9', background: '#f6f8fb' },
        colors: {
            sidebarBg: '#1e325c',
            sidebarText: '#ffffff',
            centerChannelBg: '#ffffff',
            centerChannelColor: '#3d3c40',
            linkColor: '#166de0',
            buttonBg: '#166de0',
            buttonColor: '#ffffff',
        }
    },
    { 
        id: 'dark', 
        label: 'Dark', 
        swatches: { primary: '#38bdf8', accent: '#22d3ee', background: '#0b1220' },
        colors: {
            sidebarBg: '#1f222a',
            sidebarText: '#ffffff',
            centerChannelBg: '#0b1220',
            centerChannelColor: '#dddddd',
            linkColor: '#38bdf8',
            buttonBg: '#38bdf8',
            buttonColor: '#ffffff',
        }
    },
    { 
        id: 'modern', 
        label: 'Modern', 
        swatches: { primary: '#0f766e', accent: '#14b8a6', background: '#f3f7f6' },
        colors: {
            sidebarBg: '#1a1d24',
            sidebarText: '#e2e8f0',
            centerChannelBg: '#f3f7f6',
            centerChannelColor: '#0f172a',
            linkColor: '#0f766e',
            buttonBg: '#0f766e',
            buttonColor: '#ffffff',
        }
    },
    { 
        id: 'metallic', 
        label: 'Metallic', 
        swatches: { primary: '#475569', accent: '#d97706', background: '#e7eaee' },
        colors: {
            sidebarBg: '#334155',
            sidebarText: '#f1f5f9',
            centerChannelBg: '#e7eaee',
            centerChannelColor: '#1e293b',
            linkColor: '#d97706',
            buttonBg: '#475569',
            buttonColor: '#ffffff',
        }
    },
    { 
        id: 'futuristic', 
        label: 'Futuristic', 
        swatches: { primary: '#06b6d4', accent: '#22c55e', background: '#030712' },
        colors: {
            sidebarBg: '#0f172a',
            sidebarText: '#22c55e',
            centerChannelBg: '#030712',
            centerChannelColor: '#06b6d4',
            linkColor: '#22c55e',
            buttonBg: '#06b6d4',
            buttonColor: '#000000',
        }
    },
    { 
        id: 'high-contrast', 
        label: 'High Contrast', 
        swatches: { primary: '#00e5ff', accent: '#ffd400', background: '#000000' },
        colors: {
            sidebarBg: '#000000',
            sidebarText: '#ffffff',
            centerChannelBg: '#000000',
            centerChannelColor: '#ffffff',
            linkColor: '#00e5ff',
            buttonBg: '#00e5ff',
            buttonColor: '#000000',
        }
    },
    { 
        id: 'simple', 
        label: 'Simple', 
        swatches: { primary: '#0369a1', accent: '#16a34a', background: '#fafaf9' },
        colors: {
            sidebarBg: '#44403c',
            sidebarText: '#fafaf9',
            centerChannelBg: '#fafaf9',
            centerChannelColor: '#292524',
            linkColor: '#0369a1',
            buttonBg: '#16a34a',
            buttonColor: '#ffffff',
        }
    },
    { 
        id: 'dynamic', 
        label: 'Dynamic', 
        swatches: { primary: '#e11d48', accent: '#f59e0b', background: '#111827' },
        colors: {
            sidebarBg: '#1f2937',
            sidebarText: '#f9fafb',
            centerChannelBg: '#111827',
            centerChannelColor: '#e5e7eb',
            linkColor: '#e11d48',
            buttonBg: '#f59e0b',
            buttonColor: '#000000',
        }
    },
]

export function getThemeColors(themeId: Theme): ThemeColors {
    const theme = THEME_OPTIONS.find(t => t.id === themeId)
    return theme?.colors ?? DEFAULT_THEME_COLORS
}

const DEFAULT_THEME_COLORS: ThemeColors = {
    sidebarBg: '#1e325c',
    sidebarText: '#ffffff',
    centerChannelBg: '#ffffff',
    centerChannelColor: '#3d3c40',
    linkColor: '#166de0',
    buttonBg: '#166de0',
    buttonColor: '#ffffff',
}

export const FONT_OPTIONS: Array<{ id: ChatFont; label: string; cssVar: string }> = [
    { id: 'inter', label: 'Inter', cssVar: 'var(--font-inter)' },
    { id: 'figtree', label: 'Figtree', cssVar: 'var(--font-figtree)' },
    { id: 'jetbrains-mono', label: 'JetBrains Mono', cssVar: 'var(--font-jetbrains-mono)' },
    { id: 'quicksand', label: 'Quicksand', cssVar: 'var(--font-quicksand)' },
    { id: 'montserrat', label: 'Montserrat', cssVar: 'var(--font-montserrat)' },
    { id: 'source-sans-3', label: 'Source Sans 3', cssVar: 'var(--font-source-sans-3)' },
    { id: 'nunito', label: 'Nunito', cssVar: 'var(--font-nunito)' },
    { id: 'manrope', label: 'Manrope', cssVar: 'var(--font-manrope)' },
    { id: 'work-sans', label: 'Work Sans', cssVar: 'var(--font-work-sans)' },
    { id: 'ibm-plex-sans', label: 'IBM Plex Sans', cssVar: 'var(--font-ibm-plex-sans)' },
]

export const FONT_SIZE_OPTIONS: ChatFontSize[] = [13, 14, 16, 18, 20]

const STORAGE_THEME = 'theme'
const STORAGE_FONT = 'chat_font'
const STORAGE_FONT_SIZE = 'chat_font_size'
const AUTH_TOKEN_KEY = 'auth_token'

const SERVER_PREFERENCE_URL = '/api/v4/users/me/preferences'
const SERVER_PREFERENCE_CATEGORY = 'rustchat_display'
const SERVER_PREFERENCE_THEME = 'theme'
const SERVER_PREFERENCE_FONT = 'font'
const SERVER_PREFERENCE_FONT_SIZE = 'font_size'

const DARK_THEME_SET = new Set<Theme>(['dark', 'futuristic', 'high-contrast', 'dynamic'])

function isTheme(value: unknown): value is Theme {
    return typeof value === 'string' && THEME_OPTIONS.some((option) => option.id === value)
}

function isChatFont(value: unknown): value is ChatFont {
    return typeof value === 'string' && FONT_OPTIONS.some((option) => option.id === value)
}

function normalizeTheme(value: string | null): Theme {
    if (value === 'system') {
        return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
    }
    if (isTheme(value)) {
        return value
    }
    if (value === 'dark') {
        return 'dark'
    }
    if (value === 'light') {
        return 'light'
    }
    return 'light'
}

function normalizeFontSize(value: string | null): ChatFontSize {
    const parsed = Number(value)
    if (FONT_SIZE_OPTIONS.includes(parsed as ChatFontSize)) {
        return parsed as ChatFontSize
    }
    return 14
}

type MmPreference = {
    user_id: string
    category: string
    name: string
    value: string
}

type ServerAppearancePreferences = {
    theme?: Theme
    font?: ChatFont
    fontSize?: ChatFontSize
}

function getAuthToken(): string {
    if (typeof window === 'undefined') {
        return ''
    }
    return localStorage.getItem(AUTH_TOKEN_KEY) || ''
}

function buildPreferencePayload(theme: Theme, font: ChatFont, fontSize: ChatFontSize): MmPreference[] {
    return [
        {
            user_id: 'me',
            category: SERVER_PREFERENCE_CATEGORY,
            name: SERVER_PREFERENCE_THEME,
            value: theme,
        },
        {
            user_id: 'me',
            category: SERVER_PREFERENCE_CATEGORY,
            name: SERVER_PREFERENCE_FONT,
            value: font,
        },
        {
            user_id: 'me',
            category: SERVER_PREFERENCE_CATEGORY,
            name: SERVER_PREFERENCE_FONT_SIZE,
            value: String(fontSize),
        },
    ]
}

function parseServerAppearancePreferences(rows: unknown): ServerAppearancePreferences {
    if (!Array.isArray(rows)) {
        return {}
    }

    const prefs = rows as MmPreference[]
    const getValue = (name: string) =>
        prefs.find((p) => p.category === SERVER_PREFERENCE_CATEGORY && p.name === name)?.value

    const rawTheme = getValue(SERVER_PREFERENCE_THEME)
    const rawFont = getValue(SERVER_PREFERENCE_FONT)
    const rawFontSize = getValue(SERVER_PREFERENCE_FONT_SIZE)

    return {
        theme: isTheme(rawTheme) ? rawTheme : undefined,
        font: isChatFont(rawFont) ? rawFont : undefined,
        fontSize:
            typeof rawFontSize === 'string' &&
            FONT_SIZE_OPTIONS.includes(Number(rawFontSize) as ChatFontSize)
                ? (Number(rawFontSize) as ChatFontSize)
                : undefined,
    }
}

export const useThemeStore = defineStore('theme', () => {
    const initialTheme =
        typeof window !== 'undefined' ? normalizeTheme(localStorage.getItem(STORAGE_THEME)) : 'light'
    const initialFont =
        typeof window !== 'undefined' && isChatFont(localStorage.getItem(STORAGE_FONT))
            ? (localStorage.getItem(STORAGE_FONT) as ChatFont)
            : 'inter'
    const initialFontSize =
        typeof window !== 'undefined' ? normalizeFontSize(localStorage.getItem(STORAGE_FONT_SIZE)) : 14

    const theme = ref<Theme>(initialTheme)
    const chatFont = ref<ChatFont>(initialFont)
    const chatFontSize = ref<ChatFontSize>(initialFontSize)
    const syncedServerToken = ref<string | null>(null)

    function applyTheme() {
        if (typeof window === 'undefined') {
            return
        }

        const root = window.document.documentElement
        root.setAttribute('data-theme', theme.value)
        if (DARK_THEME_SET.has(theme.value)) {
            root.classList.add('dark')
        } else {
            root.classList.remove('dark')
        }

        // Apply theme color CSS variables
        const colors = getThemeColors(theme.value)
        root.style.setProperty('--theme-sidebar-bg', colors.sidebarBg)
        root.style.setProperty('--theme-sidebar-text', colors.sidebarText)
        root.style.setProperty('--theme-center-channel-bg', colors.centerChannelBg)
        root.style.setProperty('--theme-center-channel-color', colors.centerChannelColor)
        root.style.setProperty('--theme-link-color', colors.linkColor)
        root.style.setProperty('--theme-button-bg', colors.buttonBg)
        root.style.setProperty('--theme-button-color', colors.buttonColor)
    }

    function applyTypography() {
        if (typeof window === 'undefined') {
            return
        }

        const root = window.document.documentElement
        root.style.setProperty('--chat-font-family', `var(--font-${chatFont.value})`)
        root.style.setProperty('--chat-font-size', `${chatFontSize.value}px`)
    }

    function applyAppearance() {
        applyTheme()
        applyTypography()
    }

    async function persistToServer() {
        if (typeof window === 'undefined') {
            return
        }

        const token = getAuthToken()
        if (!token) {
            return
        }

        const payload = buildPreferencePayload(theme.value, chatFont.value, chatFontSize.value)

        try {
            await fetch(SERVER_PREFERENCE_URL, {
                method: 'PUT',
                headers: {
                    Authorization: `Bearer ${token}`,
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(payload),
            })
        } catch (error) {
            console.debug('Failed to persist appearance preferences to server', error)
        }
    }

    async function syncFromServer(force = false) {
        if (typeof window === 'undefined') {
            return
        }

        const token = getAuthToken()
        if (!token) {
            syncedServerToken.value = null
            return
        }
        if (!force && syncedServerToken.value === token) {
            return
        }

        try {
            const response = await fetch(SERVER_PREFERENCE_URL, {
                headers: { Authorization: `Bearer ${token}` },
            })
            if (!response.ok) {
                return
            }

            const rows = await response.json()
            const serverPrefs = parseServerAppearancePreferences(rows)

            if (serverPrefs.theme) {
                theme.value = serverPrefs.theme
                localStorage.setItem(STORAGE_THEME, serverPrefs.theme)
            }
            if (serverPrefs.font) {
                chatFont.value = serverPrefs.font
                localStorage.setItem(STORAGE_FONT, serverPrefs.font)
            }
            if (serverPrefs.fontSize) {
                chatFontSize.value = serverPrefs.fontSize
                localStorage.setItem(STORAGE_FONT_SIZE, String(serverPrefs.fontSize))
            }

            applyAppearance()
            syncedServerToken.value = token
        } catch (error) {
            console.debug('Failed to sync appearance preferences from server', error)
        }
    }

    function setTheme(newTheme: Theme | 'system' | 'light' | 'dark') {
        const normalized = newTheme === 'system' ? normalizeTheme('system') : normalizeTheme(newTheme)
        theme.value = normalized
        if (typeof window !== 'undefined') {
            localStorage.setItem(STORAGE_THEME, normalized)
        }
        applyTheme()
        void persistToServer()
    }

    function setChatFont(newFont: ChatFont) {
        if (!isChatFont(newFont)) {
            return
        }

        chatFont.value = newFont
        if (typeof window !== 'undefined') {
            localStorage.setItem(STORAGE_FONT, newFont)
        }
        applyTypography()
        void persistToServer()
    }

    function setChatFontSize(newSize: ChatFontSize) {
        if (!FONT_SIZE_OPTIONS.includes(newSize)) {
            return
        }

        chatFontSize.value = newSize
        if (typeof window !== 'undefined') {
            localStorage.setItem(STORAGE_FONT_SIZE, String(newSize))
        }
        applyTypography()
        void persistToServer()
    }

    if (typeof window !== 'undefined') {
        applyAppearance()
    }

    return {
        theme,
        chatFont,
        chatFontSize,
        setTheme,
        setChatFont,
        setChatFontSize,
        applyTheme,
        applyAppearance,
        syncFromServer,
    }
})
