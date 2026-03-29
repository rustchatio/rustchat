import { beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'

const getMembersMock = vi.fn()

const authStore = {
    user: {
        id: 'user-1',
        role: 'member',
    } as { id: string; role: string } | null,
}

vi.mock('../../api/channels', () => ({
    channelsApi: {
        getMembers: getMembersMock,
    },
}))

vi.mock('../../stores/auth', () => ({
    useAuthStore: () => authStore,
}))

async function flushPromises() {
    await Promise.resolve()
    await nextTick()
    await Promise.resolve()
}

describe('permission capabilities', () => {
    beforeEach(async () => {
        vi.clearAllMocks()
        authStore.user = {
            id: 'user-1',
            role: 'member',
        }

        const module = await import('./capabilities')
        module.clearChannelPermissionCache()
    })

    it('allows team creation only for team-managing roles', async () => {
        const { canCreateTeam } = await import('./capabilities')

        expect(canCreateTeam('member')).toBe(false)
        expect(canCreateTeam('guest')).toBe(false)
        expect(canCreateTeam('org_admin')).toBe(true)
        expect(canCreateTeam('system_admin')).toBe(true)
    })

    it('allows channel management for the channel creator without fetching members', async () => {
        const { useChannelManagementPermission } = await import('./capabilities')

        const { canManageChannel } = useChannelManagementPermission(
            () => 'channel-1',
            () => 'user-1',
        )

        await flushPromises()

        expect(canManageChannel.value).toBe(true)
        expect(getMembersMock).not.toHaveBeenCalled()
    })

    it('loads membership once and grants channel management to channel admins', async () => {
        getMembersMock.mockResolvedValue({
            data: [
                { user_id: 'user-1', role: 'admin' },
                { user_id: 'user-2', role: 'member' },
            ],
        })

        const { useChannelManagementPermission } = await import('./capabilities')
        const { canManageChannel, isLoading } = useChannelManagementPermission(
            () => 'channel-2',
            () => 'user-9',
        )

        expect(isLoading.value).toBe(true)
        await flushPromises()

        expect(getMembersMock).toHaveBeenCalledTimes(1)
        expect(getMembersMock).toHaveBeenCalledWith('channel-2')
        expect(isLoading.value).toBe(false)
        expect(canManageChannel.value).toBe(true)
    })

    it('keeps channel management disabled for a regular member', async () => {
        getMembersMock.mockResolvedValue({
            data: [
                { user_id: 'user-1', role: 'member' },
            ],
        })

        const { useChannelManagementPermission } = await import('./capabilities')
        const { canManageChannel } = useChannelManagementPermission(
            () => 'channel-3',
            () => 'user-9',
        )

        await flushPromises()

        expect(canManageChannel.value).toBe(false)
    })
})
