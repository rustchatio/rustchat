import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import api from '../api/client'

// Mattermost preference structure
export interface MmPreference {
    user_id: string
    category: string
    name: string
    value: string
}

// Display settings (category: display_settings)
export type TeammateNameDisplay = 'username' | 'nickname' | 'full_name'
export type TimezoneMode = 'auto' | 'manual'
export type MessageDisplay = 'standard' | 'compact'
export type ChannelDisplayMode = 'full' | 'centered'

export interface DisplaySettings {
    collapsed_reply_threads: boolean
    use_military_time: boolean
    teammate_name_display: TeammateNameDisplay
    availability_status_visible: boolean
    show_last_active_time: boolean
    timezone_mode: TimezoneMode
    timezone_manual: string // e.g., 'UTC', 'America/New_York'
    link_previews_enabled: boolean
    image_previews_enabled: boolean
    message_display: MessageDisplay
    click_to_reply: boolean
    channel_display_mode: ChannelDisplayMode
    quick_reactions_enabled: boolean
    emoji_picker_enabled: boolean
    language: string
}

// Sidebar settings (category: sidebar_settings)
export type GroupUnreadChannels = 'never' | 'only_for_favorites' | 'always'
export type LimitVisibleDMs = 'all' | '10' | '20' | '40'

export interface SidebarSettings {
    group_unread_channels: GroupUnreadChannels
    limit_visible_dms_gms: LimitVisibleDMs
}

// Advanced settings (category: advanced_settings)
export type UnreadScrollPosition = 'start' | 'last' | 'end'

export interface AdvancedSettings {
    send_on_ctrl_enter: boolean
    enable_post_formatting: boolean
    enable_join_leave_messages: boolean
    enable_performance_debugging: boolean
    unread_scroll_position: UnreadScrollPosition
    sync_drafts: boolean
}

// Default values
const DEFAULT_DISPLAY_SETTINGS: DisplaySettings = {
    collapsed_reply_threads: false,
    use_military_time: false,
    teammate_name_display: 'username',
    availability_status_visible: true,
    show_last_active_time: true,
    timezone_mode: 'auto',
    timezone_manual: 'UTC',
    link_previews_enabled: true,
    image_previews_enabled: true,
    message_display: 'standard',
    click_to_reply: false,
    channel_display_mode: 'full',
    quick_reactions_enabled: true,
    emoji_picker_enabled: true,
    language: 'en',
}

const DEFAULT_SIDEBAR_SETTINGS: SidebarSettings = {
    group_unread_channels: 'never',
    limit_visible_dms_gms: 'all',
}

const DEFAULT_ADVANCED_SETTINGS: AdvancedSettings = {
    send_on_ctrl_enter: false,
    enable_post_formatting: true,
    enable_join_leave_messages: true,
    enable_performance_debugging: false,
    unread_scroll_position: 'last',
    sync_drafts: true,
}

// Category names (Mattermost-compatible)
const CAT_DISPLAY = 'display_settings'
const CAT_SIDEBAR = 'sidebar_settings'
const CAT_ADVANCED = 'advanced_settings'

// Helper to parse boolean from preference string
function parseBool(value: string | undefined, defaultValue: boolean): boolean {
    if (value === undefined) return defaultValue
    return value === 'true'
}

// Helper to parse string from preference with default
function parseString<T extends string>(value: string | undefined, defaultValue: T): T {
    if (value === undefined) return defaultValue
    return value as T
}

export const useMmPreferencesStore = defineStore('mmPreferences', () => {
    // State
    const displaySettings = ref<DisplaySettings>({ ...DEFAULT_DISPLAY_SETTINGS })
    const sidebarSettings = ref<SidebarSettings>({ ...DEFAULT_SIDEBAR_SETTINGS })
    const advancedSettings = ref<AdvancedSettings>({ ...DEFAULT_ADVANCED_SETTINGS })
    const loading = ref(false)
    const loaded = ref(false)

    // Getters
    const isLoading = computed(() => loading.value)
    const hasLoaded = computed(() => loaded.value)

    // Actions - parse preferences from server
    function parsePreferences(prefs: MmPreference[]) {
        for (const pref of prefs) {
            const { category, name, value } = pref

            // Display settings
            if (category === CAT_DISPLAY) {
                switch (name) {
                    case 'collapsed_reply_threads':
                        displaySettings.value.collapsed_reply_threads = parseBool(value, DEFAULT_DISPLAY_SETTINGS.collapsed_reply_threads)
                        break
                    case 'use_military_time':
                        displaySettings.value.use_military_time = parseBool(value, DEFAULT_DISPLAY_SETTINGS.use_military_time)
                        break
                    case 'teammate_name_display':
                        displaySettings.value.teammate_name_display = parseString(value as TeammateNameDisplay, DEFAULT_DISPLAY_SETTINGS.teammate_name_display)
                        break
                    case 'availability_status_visible':
                        displaySettings.value.availability_status_visible = parseBool(value, DEFAULT_DISPLAY_SETTINGS.availability_status_visible)
                        break
                    case 'show_last_active_time':
                        displaySettings.value.show_last_active_time = parseBool(value, DEFAULT_DISPLAY_SETTINGS.show_last_active_time)
                        break
                    case 'timezone':
                        if (value === 'auto') {
                            displaySettings.value.timezone_mode = 'auto'
                        } else {
                            displaySettings.value.timezone_mode = 'manual'
                            displaySettings.value.timezone_manual = value || 'UTC'
                        }
                        break
                    case 'link_previews_enabled':
                        displaySettings.value.link_previews_enabled = parseBool(value, DEFAULT_DISPLAY_SETTINGS.link_previews_enabled)
                        break
                    case 'image_previews_enabled':
                        displaySettings.value.image_previews_enabled = parseBool(value, DEFAULT_DISPLAY_SETTINGS.image_previews_enabled)
                        break
                    case 'message_display':
                        displaySettings.value.message_display = parseString(value as MessageDisplay, DEFAULT_DISPLAY_SETTINGS.message_display)
                        break
                    case 'click_to_reply':
                        displaySettings.value.click_to_reply = parseBool(value, DEFAULT_DISPLAY_SETTINGS.click_to_reply)
                        break
                    case 'channel_display_mode':
                        displaySettings.value.channel_display_mode = parseString(value as ChannelDisplayMode, DEFAULT_DISPLAY_SETTINGS.channel_display_mode)
                        break
                    case 'quick_reactions_enabled':
                        displaySettings.value.quick_reactions_enabled = parseBool(value, DEFAULT_DISPLAY_SETTINGS.quick_reactions_enabled)
                        break
                    case 'emoji_picker_enabled':
                        displaySettings.value.emoji_picker_enabled = parseBool(value, DEFAULT_DISPLAY_SETTINGS.emoji_picker_enabled)
                        break
                    case 'language':
                        displaySettings.value.language = parseString(value, DEFAULT_DISPLAY_SETTINGS.language)
                        break
                }
            }

            // Sidebar settings
            if (category === CAT_SIDEBAR) {
                switch (name) {
                    case 'group_unread_channels':
                        sidebarSettings.value.group_unread_channels = parseString(value as GroupUnreadChannels, DEFAULT_SIDEBAR_SETTINGS.group_unread_channels)
                        break
                    case 'limit_visible_dms_gms':
                        sidebarSettings.value.limit_visible_dms_gms = parseString(value as LimitVisibleDMs, DEFAULT_SIDEBAR_SETTINGS.limit_visible_dms_gms)
                        break
                }
            }

            // Advanced settings
            if (category === CAT_ADVANCED) {
                switch (name) {
                    case 'send_on_ctrl_enter':
                        advancedSettings.value.send_on_ctrl_enter = parseBool(value, DEFAULT_ADVANCED_SETTINGS.send_on_ctrl_enter)
                        break
                    case 'enable_post_formatting':
                        advancedSettings.value.enable_post_formatting = parseBool(value, DEFAULT_ADVANCED_SETTINGS.enable_post_formatting)
                        break
                    case 'enable_join_leave_messages':
                        advancedSettings.value.enable_join_leave_messages = parseBool(value, DEFAULT_ADVANCED_SETTINGS.enable_join_leave_messages)
                        break
                    case 'enable_performance_debugging':
                        advancedSettings.value.enable_performance_debugging = parseBool(value, DEFAULT_ADVANCED_SETTINGS.enable_performance_debugging)
                        break
                    case 'unread_scroll_position':
                        advancedSettings.value.unread_scroll_position = parseString(value as UnreadScrollPosition, DEFAULT_ADVANCED_SETTINGS.unread_scroll_position)
                        break
                    case 'sync_drafts':
                        advancedSettings.value.sync_drafts = parseBool(value, DEFAULT_ADVANCED_SETTINGS.sync_drafts)
                        break
                }
            }
        }
    }

    // Build MM preference payload from current settings
    function buildDisplayPreferences(): MmPreference[] {
        const timezoneValue = displaySettings.value.timezone_mode === 'auto' 
            ? 'auto' 
            : displaySettings.value.timezone_manual

        return [
            { user_id: 'me', category: CAT_DISPLAY, name: 'collapsed_reply_threads', value: String(displaySettings.value.collapsed_reply_threads) },
            { user_id: 'me', category: CAT_DISPLAY, name: 'use_military_time', value: String(displaySettings.value.use_military_time) },
            { user_id: 'me', category: CAT_DISPLAY, name: 'teammate_name_display', value: displaySettings.value.teammate_name_display },
            { user_id: 'me', category: CAT_DISPLAY, name: 'availability_status_visible', value: String(displaySettings.value.availability_status_visible) },
            { user_id: 'me', category: CAT_DISPLAY, name: 'show_last_active_time', value: String(displaySettings.value.show_last_active_time) },
            { user_id: 'me', category: CAT_DISPLAY, name: 'timezone', value: timezoneValue },
            { user_id: 'me', category: CAT_DISPLAY, name: 'link_previews_enabled', value: String(displaySettings.value.link_previews_enabled) },
            { user_id: 'me', category: CAT_DISPLAY, name: 'image_previews_enabled', value: String(displaySettings.value.image_previews_enabled) },
            { user_id: 'me', category: CAT_DISPLAY, name: 'message_display', value: displaySettings.value.message_display },
            { user_id: 'me', category: CAT_DISPLAY, name: 'click_to_reply', value: String(displaySettings.value.click_to_reply) },
            { user_id: 'me', category: CAT_DISPLAY, name: 'channel_display_mode', value: displaySettings.value.channel_display_mode },
            { user_id: 'me', category: CAT_DISPLAY, name: 'quick_reactions_enabled', value: String(displaySettings.value.quick_reactions_enabled) },
            { user_id: 'me', category: CAT_DISPLAY, name: 'emoji_picker_enabled', value: String(displaySettings.value.emoji_picker_enabled) },
            { user_id: 'me', category: CAT_DISPLAY, name: 'language', value: displaySettings.value.language },
        ]
    }

    function buildSidebarPreferences(): MmPreference[] {
        return [
            { user_id: 'me', category: CAT_SIDEBAR, name: 'group_unread_channels', value: sidebarSettings.value.group_unread_channels },
            { user_id: 'me', category: CAT_SIDEBAR, name: 'limit_visible_dms_gms', value: sidebarSettings.value.limit_visible_dms_gms },
        ]
    }

    function buildAdvancedPreferences(): MmPreference[] {
        return [
            { user_id: 'me', category: CAT_ADVANCED, name: 'send_on_ctrl_enter', value: String(advancedSettings.value.send_on_ctrl_enter) },
            { user_id: 'me', category: CAT_ADVANCED, name: 'enable_post_formatting', value: String(advancedSettings.value.enable_post_formatting) },
            { user_id: 'me', category: CAT_ADVANCED, name: 'enable_join_leave_messages', value: String(advancedSettings.value.enable_join_leave_messages) },
            { user_id: 'me', category: CAT_ADVANCED, name: 'enable_performance_debugging', value: String(advancedSettings.value.enable_performance_debugging) },
            { user_id: 'me', category: CAT_ADVANCED, name: 'unread_scroll_position', value: advancedSettings.value.unread_scroll_position },
            { user_id: 'me', category: CAT_ADVANCED, name: 'sync_drafts', value: String(advancedSettings.value.sync_drafts) },
        ]
    }

    // Fetch preferences from server
    async function fetchPreferences(): Promise<void> {
        loading.value = true
        try {
            const response = await api.get<MmPreference[]>('/users/me/preferences')
            parsePreferences(response.data)
            loaded.value = true
        } catch (error) {
            console.error('Failed to fetch MM preferences:', error)
        } finally {
            loading.value = false
        }
    }

    // Update a single display setting
    async function updateDisplaySetting<K extends keyof DisplaySettings>(
        key: K,
        value: DisplaySettings[K]
    ): Promise<void> {
        displaySettings.value[key] = value
        await persistDisplaySettings()
    }

    // Update a single sidebar setting
    async function updateSidebarSetting<K extends keyof SidebarSettings>(
        key: K,
        value: SidebarSettings[K]
    ): Promise<void> {
        sidebarSettings.value[key] = value
        await persistSidebarSettings()
    }

    // Update a single advanced setting
    async function updateAdvancedSetting<K extends keyof AdvancedSettings>(
        key: K,
        value: AdvancedSettings[K]
    ): Promise<void> {
        advancedSettings.value[key] = value
        await persistAdvancedSettings()
    }

    // Persist to server
    async function persistDisplaySettings(): Promise<void> {
        try {
            const prefs = buildDisplayPreferences()
            await api.put('/users/me/preferences', prefs)
        } catch (error) {
            console.error('Failed to persist display settings:', error)
        }
    }

    async function persistSidebarSettings(): Promise<void> {
        try {
            const prefs = buildSidebarPreferences()
            await api.put('/users/me/preferences', prefs)
        } catch (error) {
            console.error('Failed to persist sidebar settings:', error)
        }
    }

    async function persistAdvancedSettings(): Promise<void> {
        try {
            const prefs = buildAdvancedPreferences()
            await api.put('/users/me/preferences', prefs)
        } catch (error) {
            console.error('Failed to persist advanced settings:', error)
        }
    }

    // Update all settings at once (for batch updates)
    async function updateAllSettings(
        display: Partial<DisplaySettings>,
        sidebar: Partial<SidebarSettings>,
        advanced: Partial<AdvancedSettings>
    ): Promise<void> {
        Object.assign(displaySettings.value, display)
        Object.assign(sidebarSettings.value, sidebar)
        Object.assign(advancedSettings.value, advanced)

        const allPrefs = [
            ...buildDisplayPreferences(),
            ...buildSidebarPreferences(),
            ...buildAdvancedPreferences(),
        ]

        try {
            await api.put('/users/me/preferences', allPrefs)
        } catch (error) {
            console.error('Failed to persist all settings:', error)
        }
    }

    return {
        // State
        displaySettings,
        sidebarSettings,
        advancedSettings,
        loading,
        loaded,
        // Getters
        isLoading,
        hasLoaded,
        // Actions
        fetchPreferences,
        updateDisplaySetting,
        updateSidebarSetting,
        updateAdvancedSetting,
        updateAllSettings,
        persistDisplaySettings,
        persistSidebarSettings,
        persistAdvancedSettings,
    }
})
