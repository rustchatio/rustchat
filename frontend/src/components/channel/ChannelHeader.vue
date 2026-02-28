<script setup lang="ts">
import { Users, Search, Hash, Lock, Phone, Bookmark, MoreVertical, LogOut, Info, Pin, PhoneCall } from 'lucide-vue-next'
import { ref, computed } from 'vue';
import { useCallsStore } from '../../stores/calls';
import { useChannelStore } from '../../stores/channels';
import { useAuthStore } from '../../stores/auth';
import { useUIStore } from '../../stores/ui';

const props = defineProps<{
  name: string
  topic?: string
  channelType?: string
  channelId: string
}>()

defineEmits<{
  (e: 'openSettings'): void
  (e: 'openSaved'): void
}>()

const callsStore = useCallsStore()
const channelStore = useChannelStore()
const authStore = useAuthStore()
const uiStore = useUIStore()
const showMenu = ref(false)

const hasActiveCall = computed(() => {
    return callsStore.currentChannelCall(props.channelId)
})

const isInCurrentCall = computed(() => {
    return callsStore.isInCall && callsStore.currentCall?.channelId === props.channelId
})

const startNativeCall = async () => {
    if (!props.channelId) return
    
    // If already in this call, just expand it
    if (isInCurrentCall.value) {
        callsStore.isExpanded = true
        return
    }
    
    // If there's an existing call in this channel, join it
    if (hasActiveCall.value) {
        await callsStore.joinCall(props.channelId)
    } else {
        // Start a new call
        await callsStore.startCall(props.channelId)
    }
}

const joinExistingCall = async () => {
    if (hasActiveCall.value && props.channelId) {
        await callsStore.joinCall(props.channelId)
    }
}



const toggleView = (view: 'saved' | 'pinned' | 'search' | 'members') => {

    uiStore.toggleRhs(view)

}



const handleLeave = async () => {

    if (!confirm('Are you sure you want to leave this channel?')) return;

    

    const userId = authStore.user?.id

    

    if (props.channelId && userId) {

        try {

            await channelStore.leaveChannel(props.channelId, userId)

            showMenu.value = false

            // Navigate to general or first available channel

            const firstChannel = channelStore.channels[0]

            if (firstChannel) {

                channelStore.selectChannel(firstChannel.id)

            } else {

                channelStore.clearChannels()

            }

        } catch (e) {

            console.error('Failed to leave channel', e)

        }

    }

}
</script>

<template>
  <header 
    class="h-12 flex items-center justify-between px-4 shrink-0 backdrop-blur-md border-b sticky top-0 z-10 transition-colors duration-300"
    :style="{ 
      backgroundColor: 'color-mix(in srgb, var(--bg-surface-1) 95%, transparent)', 
      borderColor: 'var(--border-1)',
    }"
  >
    <div class="flex flex-col justify-center min-w-0">
        <div class="flex items-center">
            <component 
              :is="channelType === 'private' ? Lock : Hash" 
              class="w-4 h-4 mr-1.5"
              style="color: var(--brand);"
            />
            <h1 
              class="font-bold text-base tracking-tight truncate"
              style="color: var(--text-1);"
            >{{ name }}</h1>
        </div>
        <div 
          v-if="topic" 
          class="text-xs truncate max-w-lg mt-0.5 font-medium opacity-60"
          style="color: var(--text-2);"
        >
            {{ topic }}
        </div>
    </div>
    
    <div 
      class="flex items-center space-x-1 shrink-0"
      style="color: var(--text-2);"
    >
        <button 
          @click="toggleView('members')"
          class="w-8 h-8 flex items-center justify-center rounded-full transition-all duration-200"
          :class="{ 
            'bg-brand text-white': uiStore.rhsView === 'members',
            'hover:bg-surface-2 text-text-2': uiStore.rhsView !== 'members'
          }"
          title="Members"
        >
            <Users class="w-4 h-4" />
        </button>
        
        <div class="w-px h-4 bg-border-2 mx-1.5"></div>

        <!-- Native Audio Call Button -->
        <button 
          v-if="hasActiveCall && !isInCurrentCall"
          @click="joinExistingCall"
          class="w-8 h-8 flex items-center justify-center bg-green-500/20 text-green-600 dark:text-green-400 hover:bg-green-500/30 rounded-full transition-all duration-200 animate-pulse"
          title="Join active call"
        >
            <PhoneCall class="w-4 h-4" />
        </button>
        <button 
          v-else-if="isInCurrentCall"
          @click="callsStore.isExpanded = true"
          class="w-8 h-8 flex items-center justify-center bg-green-500/20 text-green-600 dark:text-green-400 hover:bg-green-500/30 rounded-full transition-all duration-200"
          title="Show call"
        >
            <Phone class="w-4 h-4" />
        </button>
        <button 
          v-else
          @click="startNativeCall"
          class="w-8 h-8 flex items-center justify-center hover:bg-green-50 dark:hover:bg-green-500/10 text-green-600 dark:text-green-400 rounded-full transition-all duration-200"
          title="Start audio call"
        >
            <Phone class="w-4 h-4" />
        </button>

        <button 
          @click="toggleView('search')"
          class="w-8 h-8 flex items-center justify-center rounded-full transition-all duration-200"
          :class="{ 
            'bg-brand text-white': uiStore.rhsView === 'search',
            'hover:bg-surface-2 text-text-2': uiStore.rhsView !== 'search'
          }"
          title="Search"
        >
            <Search class="w-4 h-4" />
        </button>
        <button 
          @click="toggleView('pinned')"
          class="w-8 h-8 flex items-center justify-center rounded-full transition-all duration-200"
          :class="{ 
            'bg-brand text-white': uiStore.rhsView === 'pinned',
            'hover:bg-surface-2 text-brand': uiStore.rhsView !== 'pinned'
          }"
          title="Pinned items"
        >
            <Pin class="w-4 h-4" />
        </button>
        <button 
          @click="toggleView('saved')"
          class="w-8 h-8 flex items-center justify-center rounded-full transition-all duration-200"
          :class="{ 
            'bg-brand text-white': uiStore.rhsView === 'saved',
            'hover:bg-surface-2 text-amber-500': uiStore.rhsView !== 'saved'
          }"
          title="Saved items"
        >
            <Bookmark class="w-4 h-4" />
        </button>
        <div class="relative">
             <button 
              @click="showMenu = !showMenu"
              class="w-8 h-8 flex items-center justify-center rounded-full transition-all duration-200 text-text-2 hover:bg-surface-2"
              title="More options"
            >
                <MoreVertical class="w-4 h-4" />
            </button>
            
            <div 
                v-if="showMenu"
                class="absolute right-0 top-full mt-2 w-48 rounded-lg shadow-xl z-20 py-1 origin-top-right backdrop-blur-sm bg-bg-surface-1 border border-border-1"
            >
                <!-- Close menu when clicking outside (handled by backdrop usually, or simple v-if logic for now) -->
                <div class="fixed inset-0 z-[-1]" @click="showMenu = false"></div>

                <button 
                    @click="uiStore.toggleRhs('info'); showMenu = false"
                    class="w-full px-4 py-2 text-left text-sm flex items-center transition-colors duration-200 text-text-2 hover:bg-surface-2"
                >
                    <Info class="w-4 h-4 mr-2" />
                    Channel Details
                </button>
                
                <hr class="my-1 border-border-1" />
                
                <button 
                    @click="handleLeave"
                    class="w-full px-4 py-2 text-left text-sm flex items-center transition-colors duration-200 text-danger hover:bg-danger/10"
                >
                    <LogOut class="w-4 h-4 mr-2" />
                    Leave Channel
                </button>
            </div>
        </div>
    </div>
  </header>
</template>
