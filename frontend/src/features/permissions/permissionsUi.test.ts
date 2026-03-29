// @vitest-environment jsdom

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

const mocks = vi.hoisted(() => ({
    authStore: {
        user: {
            id: 'user-1',
            role: 'member',
        } as { id: string; role: string } | null,
    },
    teamStore: {
        teams: [],
        loading: false,
        currentTeamId: 'team-1',
        fetchTeams: vi.fn(),
        selectTeam: vi.fn(),
        members: [],
    },
    unreadStore: {
        getTeamUnreadCount: vi.fn(() => 0),
        markAsRead: vi.fn(),
        markAsUnread: vi.fn(),
    },
    channelStore: {
        updateChannel: vi.fn(),
    },
    channelPrefsStore: {
        isFavorite: vi.fn(() => false),
        isMuted: vi.fn(() => false),
        toggleFavorite: vi.fn(),
        toggleMute: vi.fn(),
    },
    channelRepository: {
        getCategories: vi.fn(async () => []),
        leave: vi.fn(),
    },
    postsApi: {
        list: vi.fn(async () => ({ data: { messages: [] } })),
    },
    channelsApi: {
        getMembers: vi.fn(async () => ({ data: [] })),
        update: vi.fn(),
        delete: vi.fn(),
        join: vi.fn(),
        removeMember: vi.fn(),
    },
    toast: {
        success: vi.fn(),
        error: vi.fn(),
    },
    canManageChannel: { value: false, __v_isRef: true },
    isLoading: { value: false, __v_isRef: true },
}))

vi.mock('../../stores/auth', () => ({
    useAuthStore: () => mocks.authStore,
}))

vi.mock('../../stores/teams', () => ({
    useTeamStore: () => mocks.teamStore,
}))

vi.mock('../../stores/unreads', () => ({
    useUnreadStore: () => mocks.unreadStore,
}))

vi.mock('../../stores/channels', () => ({
    useChannelStore: () => mocks.channelStore,
}))

vi.mock('../../stores/channelPreferences', () => ({
    useChannelPreferencesStore: () => mocks.channelPrefsStore,
}))

vi.mock('../../features/channels/repositories/channelRepository', () => ({
    channelRepository: mocks.channelRepository,
}))

vi.mock('../../api/posts', () => ({
    postsApi: mocks.postsApi,
}))

vi.mock('../../api/channels', () => ({
    channelsApi: mocks.channelsApi,
}))

vi.mock('../../composables/useToast', () => ({
    useToast: () => mocks.toast,
}))

vi.mock('./capabilities', () => ({
    canCreateTeam: (role?: string | null) =>
        ['system_admin', 'org_admin', 'team_admin', 'admin'].includes(role ?? ''),
    useChannelManagementPermission: () => ({
        canManageChannel: mocks.canManageChannel,
        isLoading: mocks.isLoading,
    }),
    clearChannelPermissionCache: vi.fn(),
}))

describe('permission UI guardrails', () => {
    beforeEach(() => {
        vi.clearAllMocks()
        mocks.authStore.user = {
            id: 'user-1',
            role: 'member',
        }
        mocks.canManageChannel.value = false
        mocks.isLoading.value = false
    })

    it('hides the create team affordance from members', async () => {
        const TeamRail = (await import('../../components/layout/TeamRail.vue')).default

        const wrapper = mount(TeamRail, {
            global: {
                stubs: {
                    CreateTeamModal: true,
                },
            },
        })

        expect(wrapper.find('button[title="Create Team"]').exists()).toBe(false)
    })

    it('shows the create team affordance for org admins', async () => {
        mocks.authStore.user = {
            id: 'user-1',
            role: 'org_admin',
        }

        const TeamRail = (await import('../../components/layout/TeamRail.vue')).default

        const wrapper = mount(TeamRail, {
            global: {
                stubs: {
                    CreateTeamModal: true,
                },
            },
        })

        expect(wrapper.find('button[title="Create Team"]').exists()).toBe(true)
    })

    it('hides add-members actions in the channel context menu for unauthorized members', async () => {
        const ChannelContextMenu = (await import('../../components/channels/ChannelContextMenu.vue')).default

        const wrapper = mount(ChannelContextMenu, {
            props: {
                channelId: 'channel-1',
                channelName: 'town-square',
                channelType: 'public',
                creatorId: 'user-9',
                unreadCount: 0,
            },
        })

        expect(wrapper.text()).not.toContain('Add Members')
    })

    it('shows a locked state instead of edit controls in channel settings for unauthorized members', async () => {
        const ChannelSettingsModal = (await import('../../components/modals/ChannelSettingsModal.vue')).default

        const wrapper = mount(ChannelSettingsModal, {
            props: {
                isOpen: true,
                channel: {
                    id: 'channel-1',
                    team_id: 'team-1',
                    name: 'town-square',
                    display_name: 'Town Square',
                    channel_type: 'public',
                    creator_id: 'user-9',
                    created_at: new Date().toISOString(),
                },
            },
            global: {
                stubs: {
                    teleport: true,
                    BaseButton: {
                        template: '<button><slot /></button>',
                    },
                    BaseInput: {
                        template: '<input />',
                    },
                },
            },
        })

        expect(wrapper.text()).toContain('You do not have permission to manage this channel.')
        expect(wrapper.text()).not.toContain('Save Changes')
        expect(wrapper.text()).not.toContain('Add Member')
    })
})
