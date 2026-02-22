<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { 
    Check, 
    Star, 
    BellOff, 
    Bell, 
    FolderOpen, 
    Link, 
    UserPlus, 
    LogOut, 
    Trash2
} from 'lucide-vue-next';
import { useChannelPreferencesStore } from '../../stores/channelPreferences';
import { useUnreadStore } from '../../stores/unreads';
import { useTeamStore } from '../../stores/teams';
import { useAuthStore } from '../../stores/auth';
import { channelRepository } from '../../features/channels/repositories/channelRepository';
import type { SidebarCategory } from '../../api/channels';

interface ChannelMenuItem {
    id: string
    label: string
    icon?: any
    action: () => void
    disabled?: boolean
    danger?: boolean
    separator?: boolean
}

const props = defineProps<{
    channelId: string
    channelName: string
    channelType: 'public' | 'private' | 'dm' | 'group'
    isOwner?: boolean
    isAdmin?: boolean
    unreadCount?: number
}>()

const emit = defineEmits<{
    (e: 'close'): void
    (e: 'action', action: string): void
    (e: 'open-add-members'): void
    (e: 'open-move-to', categories: SidebarCategory[]): void
}>()

const channelPrefsStore = useChannelPreferencesStore()
const unreadStore = useUnreadStore()
const teamStore = useTeamStore()
const authStore = useAuthStore()

const menuRef = ref<HTMLElement | null>(null)
const categories = ref<SidebarCategory[]>([])

// Menu position state
interface MenuPosition {
    left: string
    marginLeft: string
    top?: string
    bottom?: string
}

const menuPosition = ref<MenuPosition>({
    left: '100%',
    marginLeft: '8px',
    top: '0'
})

// Calculate menu position to keep it on screen
async function calculatePosition() {
    await nextTick()
    if (!menuRef.value) return
    
    const rect = menuRef.value.getBoundingClientRect()
    const viewportHeight = window.innerHeight
    
    // Check if menu would go off the bottom of the screen
    if (rect.bottom > viewportHeight - 16) {
        // Position menu above the trigger instead, aligning bottom with trigger bottom
        menuPosition.value = {
            left: '100%',
            marginLeft: '8px',
            bottom: '0'
        }
    } else {
        // Default position - align top with trigger top
        menuPosition.value = {
            left: '100%',
            marginLeft: '8px',
            top: '0'
        }
    }
}

// Check if favorited
const isFavorite = computed(() => channelPrefsStore.isFavorite(props.channelId))

// Check if muted
const isMuted = computed(() => channelPrefsStore.isMuted(props.channelId))

// Check if has unread
const hasUnread = computed(() => (props.unreadCount || 0) > 0)

// Fetch categories for Move To submenu
async function fetchCategories() {
    if (!authStore.user?.id || !teamStore.currentTeamId) return
    try {
        categories.value = await channelRepository.getCategories(
            authStore.user.id, 
            teamStore.currentTeamId
        )
    } catch (e) {
        console.error('Failed to fetch categories:', e)
    }
}

// Handle mark as read/unread
async function handleMarkReadUnread() {
    if (hasUnread.value) {
        await unreadStore.markAsRead(props.channelId)
    } else {
        await unreadStore.markAsUnread(props.channelId)
    }
    emit('action', hasUnread.value ? 'mark-read' : 'mark-unread')
    emit('close')
}

// Handle favorite/unfavorite
async function handleFavorite() {
    await channelPrefsStore.toggleFavorite(props.channelId)
    emit('action', isFavorite.value ? 'unfavorite' : 'favorite')
    emit('close')
}

// Handle mute/unmute
async function handleMute() {
    await channelPrefsStore.toggleMute(props.channelId)
    emit('action', isMuted.value ? 'unmute' : 'mute')
    emit('close')
}

// Handle copy link
function handleCopyLink() {
    const url = `${window.location.origin}/channels/${props.channelId}`
    navigator.clipboard.writeText(url)
    emit('action', 'copy-link')
    emit('close')
}

// Handle add members
function handleAddMembers() {
    emit('open-add-members')
    emit('close')
}

// Handle leave channel
async function handleLeave() {
    if (!confirm(`Are you sure you want to leave #${props.channelName}?`)) {
        return
    }
    try {
        await channelRepository.leave(props.channelId)
        emit('action', 'leave')
    } catch (e) {
        console.error('Failed to leave channel:', e)
    }
    emit('close')
}

// Handle delete channel
async function handleDelete() {
    if (!confirm(`Are you sure you want to delete #${props.channelName}? This cannot be undone.`)) {
        return
    }
    try {
        await channelRepository.delete(props.channelId)
        emit('action', 'delete')
    } catch (e) {
        console.error('Failed to delete channel:', e)
    }
    emit('close')
}

// Move to submenu handler
async function handleMoveTo() {
    if (categories.value.length === 0) {
        await fetchCategories()
    }
    emit('open-move-to', categories.value)
    emit('close')
}



// Menu items computed based on state
const menuItems = computed<ChannelMenuItem[]>(() => {
    const items: ChannelMenuItem[] = []

    // 1. Mark as Read / Mark as Unread (contextual)
    items.push({
        id: 'mark-read',
        label: hasUnread.value ? 'Mark as Read' : 'Mark as Unread',
        icon: Check,
        action: handleMarkReadUnread
    })

    // 2. Favorite / Unfavorite
    items.push({
        id: 'favorite',
        label: isFavorite.value ? 'Unfavorite' : 'Favorite',
        icon: Star,
        action: handleFavorite
    })

    // Separator after Favorite
    items.push({ id: 'sep1', label: '', action: () => {}, separator: true })

    // 3. Mute Channel / Unmute Channel
    items.push({
        id: 'mute',
        label: isMuted.value ? 'Unmute Channel' : 'Mute Channel',
        icon: isMuted.value ? Bell : BellOff,
        action: handleMute
    })

    // 4. Move to...
    items.push({
        id: 'move-to',
        label: 'Move to...',
        icon: FolderOpen,
        action: handleMoveTo
    })

    // Separator after Move to...
    items.push({ id: 'sep2', label: '', action: () => {}, separator: true })

    // 5. Copy Link
    items.push({
        id: 'copy-link',
        label: 'Copy Link',
        icon: Link,
        action: handleCopyLink
    })

    // 6. Add Members (not for DMs)
    if (props.channelType !== 'dm' && props.channelType !== 'group') {
        items.push({
            id: 'add-members',
            label: 'Add Members',
            icon: UserPlus,
            action: handleAddMembers
        })
    }

    // Separator before Leave/Delete
    items.push({ id: 'sep3', label: '', action: () => {}, separator: true })

    // 7. Leave Channel
    items.push({
        id: 'leave',
        label: 'Leave Channel',
        icon: LogOut,
        action: handleLeave,
        danger: true
    })

    // 8. Delete Channel (if owner/admin)
    if (props.isOwner || props.isAdmin) {
        items.push({
            id: 'delete',
            label: 'Delete Channel',
            icon: Trash2,
            action: handleDelete,
            danger: true
        })
    }

    return items
})

// Handle click outside
function handleClickOutside(event: MouseEvent) {
    if (menuRef.value && !menuRef.value.contains(event.target as Node)) {
        emit('close')
    }
}

onMounted(() => {
    document.addEventListener('click', handleClickOutside)
    fetchCategories()
    calculatePosition()
})

onUnmounted(() => {
    document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
    <div 
        ref="menuRef"
        class="absolute z-50 w-56 bg-bg-surface-1 rounded-lg shadow-2xl border border-border-1 py-1 animate-fade-in"
        :style="menuPosition"
        @click.stop
    >
        <template v-for="item in menuItems" :key="item.id">
            <!-- Separator -->
            <div v-if="item.separator" class="h-px bg-border-1 my-1"></div>
            
            <!-- Menu Item -->
            <button
                v-else
                @click="item.action"
                :disabled="item.disabled"
                class="w-full flex items-center px-3 py-2 text-sm text-left transition-standard"
                :class="[
                    item.danger 
                        ? 'text-danger hover:bg-danger/5' 
                        : 'text-text-2 hover:bg-bg-surface-2 hover:text-text-1',
                    item.disabled && 'opacity-50 cursor-not-allowed'
                ]"
            >
                <component v-if="item.icon" :is="item.icon" class="w-4 h-4 mr-3 opacity-70" />
                <span class="flex-1">{{ item.label }}</span>
            </button>
        </template>
    </div>
</template>

<style scoped>
.animate-fade-in {
    animation: fadeIn 0.1s ease-out;
}

@keyframes fadeIn {
    from {
        opacity: 0;
        transform: scale(0.95);
    }
    to {
        opacity: 1;
        transform: scale(1);
    }
}
</style>
