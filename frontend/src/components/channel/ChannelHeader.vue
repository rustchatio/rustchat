<script setup lang="ts">
import { Users, Search, Hash, Lock, Phone, Bookmark, MoreVertical, LogOut, Info, Pin, PhoneCall } from 'lucide-vue-next'
import { ref, computed } from 'vue';
import { useCallsStore } from '../../stores/calls';
import { useChannelStore } from '../../stores/channels';
import { useAuthStore } from '../../stores/auth';
import { useUIStore } from '../../stores/ui';
import { useConfigStore } from '../../stores/config';
import callsApi from '../../api/calls';

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
const configStore = useConfigStore()

const showMenu = ref(false)

const hasActiveCall = computed(() => {
    return callsStore.currentChannelCall(props.channelId)
})

const isInCurrentCall = computed(() => {
    return callsStore.isInCall && callsStore.currentCall?.channelId === props.channelId
})

const startCall = async () => {
    if (configStore.siteConfig.mirotalk_enabled) {
        try {
             // Use 'channel' scope since we have channelId
            const { data } = await callsApi.createMeeting('channel', props.channelId);

            if (data.mode === 'embed_iframe') {
                uiStore.openVideoCall(data.meeting_url);
            } else {
                window.open(data.meeting_url, '_blank', 'noopener,noreferrer');
            }
        } catch (e) {
            console.error('Failed to start video call', e);
            alert('Failed to start video call');
        }
    }
}

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
  <header class="h-12 flex items-center justify-between px-4 shrink-0 bg-white/80 dark:bg-slate-900/80 backdrop-blur-md border-b border-gray-200/50 dark:border-white/5 sticky top-0 z-10 transition-colors duration-300">
    <div class="flex flex-col justify-center min-w-0">
        <div class="flex items-center">
            <component 
              :is="channelType === 'private' ? Lock : Hash" 
              class="w-4 h-4 text-indigo-500 mr-1.5" 
            />
            <h1 class="font-bold text-base text-gray-900 dark:text-white tracking-tight truncate">{{ name }}</h1>
        </div>
        <div v-if="topic" class="text-xs text-gray-500 dark:text-gray-400 truncate max-w-lg mt-0.5 font-medium">
            {{ topic }}
        </div>
    </div>
    
    <div class="flex items-center space-x-1 text-gray-400 dark:text-gray-400 shrink-0">
        <button 
          @click="toggleView('members')"
            class="w-8 h-8 flex items-center justify-center hover:bg-gray-100 dark:hover:bg-white/5 rounded-full transition-all duration-200"
          :class="{ 'bg-gray-100 dark:bg-white/5 text-slate-900 dark:text-white': uiStore.rhsView === 'members' }"
          title="Members"
        >
            <Users class="w-4 h-4" />
        </button>
        
        <div class="w-px h-4 bg-gray-200 dark:bg-white/10 mx-1.5"></div>

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

        <!-- MiroTalk Video Call Button -->
        <button 
          v-if="configStore.siteConfig.mirotalk_enabled"
          @click="startCall"
          class="w-8 h-8 flex items-center justify-center hover:bg-blue-50 dark:hover:bg-blue-500/10 text-blue-600 dark:text-blue-400 rounded-full transition-all duration-200"
          title="Start video call"
        >
            <PhoneCall class="w-4 h-4" />
        </button>
        <button 
          @click="toggleView('search')"
          class="w-8 h-8 flex items-center justify-center hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-600 dark:text-slate-400 rounded-full transition-all duration-200"
          :class="{ 'bg-slate-100 dark:bg-slate-800 text-slate-900 dark:text-white': uiStore.rhsView === 'search' }"
          title="Search"
        >
            <Search class="w-4 h-4" />
        </button>
        <button 
          @click="toggleView('pinned')"
          class="w-8 h-8 flex items-center justify-center hover:bg-indigo-50 dark:hover:bg-indigo-500/10 text-indigo-600 dark:text-indigo-400 rounded-full transition-all duration-200"
          :class="{ 'bg-indigo-50 dark:bg-indigo-500/10': uiStore.rhsView === 'pinned' }"
          title="Pinned items"
        >
            <Pin class="w-4 h-4" />
        </button>
        <button 
          @click="toggleView('saved')"
          class="w-8 h-8 flex items-center justify-center hover:bg-amber-50 dark:hover:bg-amber-500/10 text-amber-600 dark:text-amber-400 rounded-full transition-all duration-200"
          :class="{ 'bg-amber-50 dark:bg-amber-500/10': uiStore.rhsView === 'saved' }"
          title="Saved items"
        >
            <Bookmark class="w-4 h-4" />
        </button>
        <div class="relative">
             <button 
              @click="showMenu = !showMenu"
              class="w-8 h-8 flex items-center justify-center hover:bg-gray-100 dark:hover:bg-white/5 rounded-full transition-all duration-200"
              title="More options"
            >
                <MoreVertical class="w-4 h-4" />
            </button>
            
            <div 
                v-if="showMenu"
                class="absolute right-0 top-full mt-2 w-48 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-xl z-20 py-1 origin-top-right backdrop-blur-sm"
            >
                <!-- Close menu when clicking outside (handled by backdrop usually, or simple v-if logic for now) -->
                <div class="fixed inset-0 z-[-1]" @click="showMenu = false"></div>

                <button 
                    @click="$emit('openSettings'); showMenu = false"
                    class="w-full px-4 py-2 text-left text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 flex items-center"
                >
                    <Info class="w-4 h-4 mr-2" />
                    Channel Details
                </button>
                
                <hr class="my-1 border-gray-200 dark:border-gray-700" />
                
                <button 
                    @click="handleLeave"
                    class="w-full px-4 py-2 text-left text-sm text-red-600 hover:bg-red-50 dark:hover:bg-red-900/20 flex items-center"
                >
                    <LogOut class="w-4 h-4 mr-2" />
                    Leave Channel
                </button>
            </div>
        </div>
    </div>
  </header>
</template>
