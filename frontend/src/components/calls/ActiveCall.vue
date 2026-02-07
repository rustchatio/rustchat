<script setup lang="ts">
import { useCallsStore } from '../../stores/calls'
import { useAuthStore } from '../../stores/auth'
import { useChannelStore } from '../../stores/channels'
import { computed, ref } from 'vue'
import { 
    Maximize2, 
    Minimize2, 
    Mic, 
    MicOff, 
    PhoneOff, 
    Hand, 
    Monitor,
    MoreVertical,
    Users
} from 'lucide-vue-next'

const callsStore = useCallsStore()
const authStore = useAuthStore()
const channelStore = useChannelStore()

const activeCall = computed(() => callsStore.currentCall)
const isExpanded = computed(() => callsStore.isExpanded)
const isMuted = computed(() => callsStore.isMuted)
const isHandRaised = computed(() => callsStore.isHandRaised)
const isScreenSharing = computed(() => callsStore.isScreenSharing)
const participants = computed(() => callsStore.currentCallParticipants)

const showParticipants = ref(false)
const showMenu = ref(false)

const channelName = computed(() => {
    if (!activeCall.value) return ''
    const channel = channelStore.channels.find(c => c.id === activeCall.value?.channelId)
    return channel?.name || 'Unknown Channel'
})

const isHost = computed(() => {
    if (!activeCall.value || !authStore.user) return false
    return activeCall.value.call.host_id === authStore.user.id ||
           activeCall.value.call.owner_id === authStore.user.id
})

const toggleExpand = () => {
    callsStore.toggleExpanded()
}

const handleHangup = () => {
    callsStore.leaveCall()
}

const handleEndCall = () => {
    callsStore.endCall()
}

const toggleMute = () => {
    callsStore.toggleMute()
}

const toggleHand = () => {
    callsStore.toggleHand()
}

const toggleScreenShare = () => {
    callsStore.toggleScreenShare()
}

const formatDuration = (startAt: number) => {
    const elapsed = Math.floor((Date.now() - startAt) / 1000)
    const minutes = Math.floor(elapsed / 60)
    const seconds = elapsed % 60
    return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
}
</script>

<template>
    <div v-if="activeCall" 
         class="fixed transition-all duration-300 bg-slate-900 border border-slate-700 shadow-2xl rounded-xl overflow-hidden z-50 flex flex-col"
         :class="[
             isExpanded ? 'inset-4' : 'bottom-4 right-4 w-80'
         ]">
        
        <!-- Header -->
        <div class="flex items-center justify-between px-4 py-3 bg-slate-950 border-b border-white/10 shrink-0">
            <div class="flex items-center space-x-3 min-w-0">
                <span class="w-2.5 h-2.5 rounded-full bg-green-500 animate-pulse shrink-0"></span>
                <div class="min-w-0">
                    <h3 class="text-white font-medium text-sm truncate">{{ channelName }}</h3>
                    <p class="text-xs text-slate-400">
                        {{ participants.length }} participant{{ participants.length !== 1 ? 's' : '' }}
                        • {{ formatDuration(activeCall.call.start_at) }}
                    </p>
                </div>
            </div>
            <div class="flex items-center space-x-1 shrink-0">
                <button 
                    @click="toggleExpand" 
                    class="p-1.5 text-slate-400 hover:text-white rounded hover:bg-white/10 transition-colors"
                    :title="isExpanded ? 'Minimize' : 'Maximize'"
                >
                    <Minimize2 v-if="isExpanded" class="w-4 h-4" />
                    <Maximize2 v-else class="w-4 h-4" />
                </button>
            </div>
        </div>

        <!-- Participants List (Expanded) -->
        <div v-if="isExpanded" class="flex-1 overflow-hidden flex">
            <!-- Main Area - Could show active speaker or screen share here -->
            <div class="flex-1 bg-slate-950 flex items-center justify-center">
                <div class="text-center">
                    <div class="w-24 h-24 rounded-full bg-slate-800 flex items-center justify-center mb-4 mx-auto">
                        <Users class="w-12 h-12 text-slate-400" />
                    </div>
                    <p class="text-slate-400 text-sm">Audio Call in Progress</p>
                    <p class="text-slate-500 text-xs mt-1">{{ participants.length }} participants</p>
                </div>
            </div>
            
            <!-- Participants Sidebar -->
            <div v-if="showParticipants" class="w-64 bg-slate-900 border-l border-white/10 overflow-y-auto">
                <div class="p-3 border-b border-white/10">
                    <h4 class="text-white font-medium text-sm">Participants</h4>
                </div>
                <div class="p-2 space-y-1">
                    <div v-for="participant in participants" :key="participant.session_id"
                         class="flex items-center space-x-2 p-2 rounded hover:bg-white/5">
                        <div class="w-8 h-8 rounded-full bg-indigo-500/20 flex items-center justify-center">
                            <span class="text-indigo-400 text-xs font-medium">
                                {{ participant.user_id.slice(0, 2).toUpperCase() }}
                            </span>
                        </div>
                        <div class="flex-1 min-w-0">
                            <p class="text-white text-sm truncate">
                                {{ participant.user_id === authStore.user?.id ? 'You' : participant.user_id.slice(0, 8) }}
                            </p>
                        </div>
                        <div class="flex items-center space-x-1">
                            <MicOff v-if="!participant.unmuted" class="w-3.5 h-3.5 text-slate-500" />
                            <Hand v-if="participant.raised_hand > 0" class="w-3.5 h-3.5 text-yellow-500" />
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- Compact Mode - Participants Preview -->
        <div v-else class="flex-1 bg-slate-950 p-3 overflow-hidden">
            <div class="flex items-center space-x-2">
                <div v-for="(participant, idx) in participants.slice(0, 5)" :key="participant.session_id"
                     class="w-10 h-10 rounded-full bg-indigo-500/20 flex items-center justify-center shrink-0"
                     :style="{ marginLeft: idx > 0 ? '-0.5rem' : '0', zIndex: 10 - idx }">
                    <span class="text-indigo-400 text-xs font-medium">
                        {{ participant.user_id.slice(0, 2).toUpperCase() }}
                    </span>
                </div>
                <div v-if="participants.length > 5" 
                     class="w-10 h-10 rounded-full bg-slate-800 flex items-center justify-center shrink-0 -ml-2">
                    <span class="text-slate-400 text-xs">+{{ participants.length - 5 }}</span>
                </div>
            </div>
            <div class="mt-2 text-xs text-slate-500">
                {{ isMuted ? 'Muted' : 'Unmuted' }}
                <span v-if="isHandRaised" class="ml-2 text-yellow-500">Hand raised</span>
            </div>
        </div>

        <!-- Controls -->
        <div class="flex items-center justify-center space-x-3 px-4 py-3 bg-slate-950 border-t border-white/10 shrink-0">
            <!-- Mute/Unmute -->
            <button 
                @click="toggleMute"
                :class="[
                    'w-12 h-12 rounded-full flex items-center justify-center transition-all',
                    isMuted 
                        ? 'bg-red-500/20 text-red-500 hover:bg-red-500/30' 
                        : 'bg-slate-800 text-white hover:bg-slate-700'
                ]"
                :title="isMuted ? 'Unmute' : 'Mute'"
            >
                <MicOff v-if="isMuted" class="w-5 h-5" />
                <Mic v-else class="w-5 h-5" />
            </button>

            <!-- Raise Hand -->
            <button 
                @click="toggleHand"
                :class="[
                    'w-10 h-10 rounded-full flex items-center justify-center transition-all',
                    isHandRaised 
                        ? 'bg-yellow-500/20 text-yellow-500' 
                        : 'bg-slate-800 text-slate-400 hover:bg-slate-700'
                ]"
                :title="isHandRaised ? 'Lower hand' : 'Raise hand'"
            >
                <Hand class="w-4 h-4" />
            </button>

            <!-- Screen Share -->
            <button 
                @click="toggleScreenShare"
                :class="[
                    'w-10 h-10 rounded-full flex items-center justify-center transition-all',
                    isScreenSharing 
                        ? 'bg-green-500/20 text-green-500' 
                        : 'bg-slate-800 text-slate-400 hover:bg-slate-700'
                ]"
                :title="isScreenSharing ? 'Stop sharing' : 'Share screen'"
            >
                <Monitor class="w-4 h-4" />
            </button>

            <!-- Participants Toggle (Expanded mode only) -->
            <button 
                v-if="isExpanded"
                @click="showParticipants = !showParticipants"
                :class="[
                    'w-10 h-10 rounded-full flex items-center justify-center transition-all',
                    showParticipants 
                        ? 'bg-indigo-500/20 text-indigo-400' 
                        : 'bg-slate-800 text-slate-400 hover:bg-slate-700'
                ]"
                :title="showParticipants ? 'Hide participants' : 'Show participants'"
            >
                <Users class="w-4 h-4" />
            </button>

            <!-- More Options -->
            <div class="relative">
                <button 
                    @click="showMenu = !showMenu"
                    class="w-10 h-10 rounded-full flex items-center justify-center bg-slate-800 text-slate-400 hover:bg-slate-700 transition-all"
                    title="More options"
                >
                    <MoreVertical class="w-4 h-4" />
                </button>
                
                <div v-if="showMenu" 
                     class="absolute bottom-full mb-2 right-0 w-48 bg-slate-800 border border-slate-700 rounded-lg shadow-xl py-1 z-50">
                    <div class="fixed inset-0 z-[-1]" @click="showMenu = false"></div>
                    
                    <button 
                        v-if="isHost"
                        @click="handleEndCall(); showMenu = false"
                        class="w-full px-4 py-2 text-left text-sm text-red-400 hover:bg-white/5 flex items-center"
                    >
                        <PhoneOff class="w-4 h-4 mr-2" />
                        End Call for Everyone
                    </button>
                </div>
            </div>

            <!-- Hangup -->
            <button 
                @click="handleHangup"
                class="w-12 h-12 rounded-full bg-red-600 hover:bg-red-500 text-white flex items-center justify-center transition-all"
                title="Leave call"
            >
                <PhoneOff class="w-5 h-5" />
            </button>
        </div>
    </div>
</template>
