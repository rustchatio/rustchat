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

export type ChatFont =
    | 'inter'
    | 'figtree'
    | 'jetbrains-mono'
    | 'quicksand'
    | 'source-sans-3'
    | 'nunito'
    | 'manrope'
    | 'public-sans'
    | 'work-sans'
    | 'ibm-plex-sans'

export type ChatFontSize = 13 | 14 | 16 | 18 | 20

export const THEME_OPTIONS: Array<{
    id: Theme
    label: string
    swatches: { primary: string; accent: string; background: string }
}> = [
    { id: 'light', label: 'Light', swatches: { primary: '#2563eb', accent: '#0ea5e9', background: '#f6f8fb' } },
    { id: 'dark', label: 'Dark', swatches: { primary: '#38bdf8', accent: '#22d3ee', background: '#0b1220' } },
    { id: 'modern', label: 'Modern', swatches: { primary: '#0f766e', accent: '#14b8a6', background: '#f3f7f6' } },
    { id: 'metallic', label: 'Metallic', swatches: { primary: '#475569', accent: '#d97706', background: '#e7eaee' } },
    { id: 'futuristic', label: 'Futuristic', swatches: { primary: '#06b6d4', accent: '#22c55e', background: '#030712' } },
    { id: 'high-contrast', label: 'High Contrast', swatches: { primary: '#00e5ff', accent: '#ffd400', background: '#000000' } },
    { id: 'simple', label: 'Simple', swatches: { primary: '#0369a1', accent: '#16a34a', background: '#fafaf9' } },
    { id: 'dynamic', label: 'Dynamic', swatches: { primary: '#e11d48', accent: '#f59e0b', background: '#111827' } },
]

export const FONT_OPTIONS: Array<{ id: ChatFont; label: string; cssVar: string }> = [
    { id: 'inter', label: 'Inter', cssVar: 'var(--font-inter)' },
    { id: 'figtree', label: 'Figtree', cssVar: 'var(--font-figtree)' },
    { id: 'jetbrains-mono', label: 'JetBrains Mono', cssVar: 'var(--font-jetbrains-mono)' },
    { id: 'quicksand', label: 'Quicksand', cssVar: 'var(--font-quicksand)' },
    { id: 'source-sans-3', label: 'Source Sans 3', cssVar: 'var(--font-source-sans-3)' },
    { id: 'nunito', label: 'Nunito', cssVar: 'var(--font-nunito)' },
    { id: 'manrope', label: 'Manrope', cssVar: 'var(--font-manrope)' },
    { id: 'public-sans', label: 'Public Sans', cssVar: 'var(--font-public-sans)' },
    { id: 'work-sans', label: 'Work Sans', cssVar: 'var(--font-work-sans)' },
    { id: 'ibm-plex-sans', label: 'IBM Plex Sans', cssVar: 'var(--font-ibm-plex-sans)' },
]

export const FONT_SIZE_OPTIONS: ChatFontSize[] = [13, 14, 16, 18, 20]

const STORAGE_THEME = 'theme'
const STORAGE_FONT = 'chat_font'
const STORAGE_FONT_SIZE = 'chat_font_size'

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

    function setTheme(newTheme: Theme | 'system' | 'light' | 'dark') {
        const normalized = newTheme === 'system' ? normalizeTheme('system') : normalizeTheme(newTheme)
        theme.value = normalized
        if (typeof window !== 'undefined') {
            localStorage.setItem(STORAGE_THEME, normalized)
        }
        applyTheme()
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
    }
})
