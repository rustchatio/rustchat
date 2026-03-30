import { beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'

const getMembersMock = vi.fn()

const authStore = {
    user: {
        id: 'user-1',
        role: 'member',
    } as { id: string; role: string } | null,
}

const teamStore = {
    currentTeamId: 'team-1',
    members: [] as Array<{ user_id: string; role: string }>,
}

vi.mock('../../api/channels', () => ({
    channelsApi: {
        getMembers: getMembersMock,
    },
}))

vi.mock('../../stores/auth', () => ({
    useAuthStore: () => authStore,
}))

vi.mock('../../stores/teams', () => ({
    useTeamStore: () => teamStore,
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
        teamStore.currentTeamId = 'team-1'
        teamStore.members = []

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

    it('allows channel creation for members and admins but not guests', async () => {
        const { canCreateChannel } = await import('./capabilities')

        expect(canCreateChannel('guest')).toBe(false)
        expect(canCreateChannel('member')).toBe(true)
        expect(canCreateChannel('org_admin')).toBe(true)
    })

    it('allows team management for team admins through membership role', async () => {
        const { canManageTeam, useCurrentTeamManagementPermission } = await import('./capabilities')

        expect(
            canManageTeam({
                currentUserRole: 'member',
                currentTeamMembershipRole: 'admin',
            }),
        ).toBe(true)

        teamStore.members = [
            { user_id: 'user-1', role: 'admin' },
            { user_id: 'user-2', role: 'member' },
        ]

        const { canManageTeam: canManageCurrentTeam } = useCurrentTeamManagementPermission(
            () => 'team-1',
        )

        await flushPromises()

        expect(canManageCurrentTeam.value).toBe(true)
    })

    it('keeps team management disabled for regular team members', async () => {
        const { useCurrentTeamManagementPermission } = await import('./capabilities')

        teamStore.members = [
            { user_id: 'user-1', role: 'member' },
        ]

        const { canManageTeam: canManageCurrentTeam } = useCurrentTeamManagementPermission(
            () => 'team-1',
        )

        await flushPromises()

        expect(canManageCurrentTeam.value).toBe(false)
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

    it('retries loading permissions after a failure', async () => {
        getMembersMock.mockRejectedValueOnce(new Error('Network Error'))
        getMembersMock.mockResolvedValueOnce({
            data: [
                { user_id: 'user-1', role: 'admin' },
            ],
        })

        const { useChannelManagementPermission } = await import('./capabilities')
        const { canManageChannel, isLoading } = useChannelManagementPermission(
            () => 'channel-retry',
            () => 'user-9',
        )

        // First attempt: failure
        await flushPromises()
        expect(getMembersMock).toHaveBeenCalledTimes(1)
        expect(isLoading.value).toBe(false)
        expect(canManageChannel.value).toBe(false)

        // Reset or re-trigger: it should retry because loaded is false
        // We can simulate this by clearing and re-watching or just calling the internal logic if accessible
        // In our current setup, it will retry if the component is re-mounted or if we clear cache.
        // Actually, watchEffect will re-run if any dependency changes.
        // But since channelId and userId didn't change, we need to manually trigger ensureChannelPermissionLoaded
        // or clear the cache entry.

        const module = await import('./capabilities')
        module.clearChannelPermissionCache()

        // After clearing cache, useChannelManagementPermission's watchEffect should trigger again
        // but it only triggers if channelId source changes.
        // Let's use a dynamic channelId.
        let channelId = 'channel-retry-1'
        const { canManageChannel: canManageDynamic, isLoading: isDynamicLoading } = useChannelManagementPermission(
            () => channelId,
            () => 'user-9',
        )

        // Failure 1
        getMembersMock.mockRejectedValueOnce(new Error('Network Error'))
        await flushPromises()
        expect(getMembersMock).toHaveBeenCalledTimes(2) // 1 from previous test + 1 now
        expect(canManageDynamic.value).toBe(false)

        // Trigger retry by changing channel ID (simulating navigation back or something)
        // or just by the fact that loaded: false allows another call if something else triggers it.
        // In the real UI, re-opening the panel would trigger this.

        channelId = 'channel-retry-2'
        getMembersMock.mockResolvedValueOnce({
            data: [
                { user_id: 'user-1', role: 'admin' },
            ],
        })
        await flushPromises()
        expect(getMembersMock).toHaveBeenCalledTimes(3)
        expect(canManageDynamic.value).toBe(true)
    })
})
