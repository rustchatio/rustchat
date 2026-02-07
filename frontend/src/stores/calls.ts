import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import callsApi, { type CallState, type CallsConfig } from '../api/calls'
import { useWebSocket } from '../composables/useWebSocket'
import { useToast } from '../composables/useToast'
import { useAuthStore } from './auth'

export interface CurrentCall {
    channelId: string
    call: CallState
    mySessionId: string
    peerConnection: RTCPeerConnection | null
    localStream: MediaStream | null
    remoteStreams: Map<string, MediaStream>
}

export const useCallsStore = defineStore('calls', () => {
    const { onEvent } = useWebSocket()
    const toast = useToast()
    const authStore = useAuthStore()
    
    // State
    const callsConfig = ref<CallsConfig | null>(null)
    const activeCalls = ref<Map<string, CallState>>(new Map())
    const currentCall = ref<CurrentCall | null>(null)
    const isExpanded = ref(false)
    const incomingCall = ref<{ channelId: string, callerId: string } | null>(null)
    const isMuted = ref(true)
    const isHandRaised = ref(false)
    const isScreenSharing = ref(false)

    // Getters
    const isInCall = computed(() => !!currentCall.value)
    const currentCallParticipants = computed(() => {
        if (!currentCall.value) return []
        return Object.values(currentCall.value.call.sessions || {})
    })
    
    const currentChannelCall = computed(() => (channelId: string) => {
        return activeCalls.value.get(channelId)
    })

    // WebSocket Event Listeners for Call Events
    onEvent('custom_com.mattermost.calls_call_start', (data) => {
        console.log('Call started:', data)
        // Reload calls to get updated state
        loadCalls()
    })

    onEvent('custom_com.mattermost.calls_call_end', (data) => {
        console.log('Call ended:', data)
        const eventChannelId = data.channel_id_raw || data.channel_id
        if (currentCall.value?.channelId === eventChannelId) {
            leaveCall()
        }
        if (eventChannelId) {
            activeCalls.value.delete(eventChannelId)
        }
    })

    onEvent('custom_com.mattermost.calls_user_joined', (data) => {
        console.log('User joined call:', data)
        const eventChannelId = data.channel_id_raw || data.channel_id
        if (currentCall.value?.channelId === eventChannelId) {
            // Update current call sessions
            loadCallForChannel(eventChannelId)
        }
    })

    onEvent('custom_com.mattermost.calls_user_left', (data) => {
        console.log('User left call:', data)
        const eventChannelId = data.channel_id_raw || data.channel_id
        if (currentCall.value?.channelId === eventChannelId) {
            loadCallForChannel(eventChannelId)
        }
    })

    onEvent('custom_com.mattermost.calls_user_muted', (data) => {
        console.log('User muted:', data)
    })

    onEvent('custom_com.mattermost.calls_user_unmuted', (data) => {
        console.log('User unmuted:', data)
    })

    onEvent('custom_com.mattermost.calls_raise_hand', (data) => {
        console.log('Hand raised:', data)
    })

    onEvent('custom_com.mattermost.calls_lower_hand', (data) => {
        console.log('Hand lowered:', data)
    })

    onEvent('custom_com.mattermost.calls_screen_on', (data) => {
        console.log('Screen share on:', data)
        const eventChannelId = data.channel_id_raw || data.channel_id
        if (currentCall.value?.channelId === eventChannelId) {
            isScreenSharing.value = true
        }
    })

    onEvent('custom_com.mattermost.calls_screen_off', (data) => {
        console.log('Screen share off:', data)
        const eventChannelId = data.channel_id_raw || data.channel_id
        if (currentCall.value?.channelId === eventChannelId) {
            isScreenSharing.value = false
        }
    })

    onEvent('custom_com.mattermost.calls_signal', async (data) => {
        const active = currentCall.value
        if (!active?.peerConnection) return

        const eventChannelId = data.channel_id_raw || data.channel_id
        if (eventChannelId !== active.channelId) return

        const signal = data.signal
        if (!signal?.type) return

        try {
            if (signal.type === 'ice-candidate' && signal.candidate) {
                await active.peerConnection.addIceCandidate({
                    candidate: signal.candidate,
                    sdpMid: signal.sdp_mid ?? null,
                    sdpMLineIndex: signal.sdp_mline_index ?? null
                })
            }
        } catch (error) {
            console.error('Failed to handle signaling event', error)
        }
    })

    // Actions
    async function loadConfig() {
        try {
            const { data } = await callsApi.getConfig()
            callsConfig.value = data
            return data
        } catch (error) {
            console.error('Failed to load calls config', error)
            return null
        }
    }

    async function loadCalls() {
        try {
            const { data } = await callsApi.getCalls()
            activeCalls.value.clear()
            for (const channel of data) {
                if (channel.call) {
                    activeCalls.value.set(channel.channel_id, channel.call)
                }
            }
            return data
        } catch (error) {
            console.error('Failed to load calls', error)
            return []
        }
    }

    async function loadCallForChannel(channelId: string) {
        try {
            const { data } = await callsApi.getCallForChannel(channelId)
            if (data.call) {
                activeCalls.value.set(channelId, data.call)
                if (currentCall.value?.channelId === channelId) {
                    currentCall.value.call = data.call
                }
            } else {
                activeCalls.value.delete(channelId)
            }
            return data
        } catch (error) {
            console.error('Failed to load call for channel', error)
            return null
        }
    }

    async function startCall(channelId: string) {
        try {
            // First get config for ICE servers
            const config = await loadConfig()
            if (!config) {
                throw new Error('Calls plugin not available')
            }

            // Start the call on the server
            const { data: callData } = await callsApi.startCall(channelId)

            const selfId = authStore.user?.id || crypto.randomUUID()
            const mySessionId = crypto.randomUUID()
            currentCall.value = {
                channelId,
                call: {
                    id: callData.id,
                    channel_id: channelId,
                    start_at: callData.start_at,
                    owner_id: callData.owner_id,
                    host_id: callData.owner_id,
                    sessions: {
                        [mySessionId]: {
                            session_id: mySessionId,
                            user_id: selfId,
                            unmuted: false,
                            raised_hand: 0,
                        },
                    }
                },
                mySessionId,
                peerConnection: null,
                localStream: null,
                remoteStreams: new Map()
            }

            // Initialize WebRTC
            const rtc = await initializeWebRTC(channelId, config.ICEServersConfigs)
            if (currentCall.value) {
                currentCall.value.peerConnection = rtc.peerConnection
                currentCall.value.localStream = rtc.localStream
            }
            
            isExpanded.value = true
            toast.success('Call started', 'You are now in a call')
            
            return callData
        } catch (error: any) {
            cleanupWebRTC()
            currentCall.value = null
            console.error('Failed to start call', error)
            toast.error('Failed to start call', error.message || 'Unknown error')
            throw error
        }
    }

    async function joinCall(channelId: string) {
        try {
            // Check if there's an active call
            const channelState = await loadCallForChannel(channelId)
            if (!channelState?.call) {
                throw new Error('No active call in this channel')
            }

            // Get config for ICE servers
            const config = await loadConfig()
            if (!config) {
                throw new Error('Calls plugin not available')
            }

            // Join the call on the server
            await callsApi.joinCall(channelId)

            const mySessionId = crypto.randomUUID()
            currentCall.value = {
                channelId,
                call: channelState.call,
                mySessionId,
                peerConnection: null,
                localStream: null,
                remoteStreams: new Map()
            }

            if (!currentCall.value.call.sessions[mySessionId]) {
                currentCall.value.call.sessions[mySessionId] = {
                    session_id: mySessionId,
                    user_id: authStore.user?.id || mySessionId,
                    unmuted: false,
                    raised_hand: 0,
                }
            }

            // Initialize WebRTC
            const rtc = await initializeWebRTC(channelId, config.ICEServersConfigs)
            if (currentCall.value) {
                currentCall.value.peerConnection = rtc.peerConnection
                currentCall.value.localStream = rtc.localStream
            }
            
            isExpanded.value = true
            toast.success('Joined call', 'You are now in the call')
            
        } catch (error: any) {
            cleanupWebRTC()
            currentCall.value = null
            console.error('Failed to join call', error)
            toast.error('Failed to join call', error.message || 'Unknown error')
            throw error
        }
    }

    async function leaveCall() {
        if (!currentCall.value) return
        
        const channelId = currentCall.value.channelId
        
        try {
            // Clean up WebRTC
            cleanupWebRTC()
            
            // Leave on server
            await callsApi.leaveCall(channelId)
            
            currentCall.value = null
            isMuted.value = true
            isHandRaised.value = false
            isScreenSharing.value = false
            isExpanded.value = false
            
        } catch (error) {
            console.error('Failed to leave call', error)
        }
    }

    async function endCall() {
        if (!currentCall.value) return
        
        const channelId = currentCall.value.channelId
        
        try {
            // Clean up WebRTC
            cleanupWebRTC()
            
            // End on server
            await callsApi.endCall(channelId)
            
            currentCall.value = null
            isMuted.value = true
            isHandRaised.value = false
            isScreenSharing.value = false
            isExpanded.value = false
            
        } catch (error) {
            console.error('Failed to end call', error)
            toast.error('Failed to end call', 'Only the host can end the call')
        }
    }

    // WebRTC
    async function initializeWebRTC(channelId: string, iceServers: RTCIceServer[]) {
        try {
            // Get user media
            const stream = await navigator.mediaDevices.getUserMedia({ 
                audio: true, 
                video: false // Audio only for now
            })
            // Calls start muted server-side, keep local tracks consistent.
            stream.getAudioTracks().forEach(track => {
                track.enabled = false
            })
            
            // Create peer connection
            const pc = new RTCPeerConnection({
                iceServers: (iceServers || []).length > 0 ? iceServers : [
                    { urls: 'stun:stun.l.google.com:19302' }
                ]
            })
            
            // Add local stream tracks
            stream.getTracks().forEach(track => {
                pc.addTrack(track, stream)
            })
            
            // Handle ICE candidates
            pc.onicecandidate = async (event) => {
                if (event.candidate) {
                    await callsApi.sendIceCandidate(
                        channelId,
                        JSON.stringify(event.candidate),
                        event.candidate.sdpMid || undefined,
                        event.candidate.sdpMLineIndex || undefined
                    )
                }
            }
            
            // Create and send offer
            const offer = await pc.createOffer()
            await pc.setLocalDescription(offer)
            
            const { data: answer } = await callsApi.sendOffer(channelId, offer.sdp!)
            
            // Set remote description
            await pc.setRemoteDescription(new RTCSessionDescription({
                type: 'answer',
                sdp: answer.sdp
            }))

            return {
                peerConnection: pc,
                localStream: stream,
            }
            
        } catch (error) {
            console.error('WebRTC initialization failed', error)
            throw error
        }
    }

    function cleanupWebRTC() {
        if (currentCall.value?.peerConnection) {
            currentCall.value.peerConnection.close()
        }
        if (currentCall.value?.localStream) {
            currentCall.value.localStream.getTracks().forEach(track => track.stop())
        }
    }

    // Call controls
    async function toggleMute() {
        if (!currentCall.value) return
        
        const channelId = currentCall.value.channelId
        try {
            if (isMuted.value) {
                await callsApi.unmute(channelId)
                // Enable audio tracks
                currentCall.value.localStream?.getAudioTracks().forEach(track => {
                    track.enabled = true
                })
            } else {
                await callsApi.mute(channelId)
                // Disable audio tracks
                currentCall.value.localStream?.getAudioTracks().forEach(track => {
                    track.enabled = false
                })
            }
            isMuted.value = !isMuted.value
        } catch (error) {
            console.error('Failed to toggle mute', error)
        }
    }

    async function toggleHand() {
        if (!currentCall.value) return
        
        const channelId = currentCall.value.channelId
        try {
            if (isHandRaised.value) {
                await callsApi.lowerHand(channelId)
            } else {
                await callsApi.raiseHand(channelId)
            }
            isHandRaised.value = !isHandRaised.value
        } catch (error) {
            console.error('Failed to toggle hand', error)
        }
    }

    async function toggleScreenShare() {
        if (!currentCall.value) return
        
        const channelId = currentCall.value.channelId
        try {
            await callsApi.toggleScreenShare(channelId)
            // Screen sharing state is updated via WebSocket events
        } catch (error) {
            console.error('Failed to toggle screen share', error)
        }
    }

    async function sendReaction(emoji: string) {
        if (!currentCall.value) return
        
        try {
            await callsApi.sendReaction(currentCall.value.channelId, emoji)
        } catch (error) {
            console.error('Failed to send reaction', error)
        }
    }

    // Set state
    function setIncomingCall(call: { channelId: string, callerId: string } | null) {
        incomingCall.value = call
    }

    function toggleExpanded() {
        isExpanded.value = !isExpanded.value
    }

    return {
        // State
        callsConfig,
        activeCalls,
        currentCall,
        isExpanded,
        incomingCall,
        isMuted,
        isHandRaised,
        isScreenSharing,
        
        // Getters
        isInCall,
        currentCallParticipants,
        currentChannelCall,
        
        // Actions
        loadConfig,
        loadCalls,
        loadCallForChannel,
        startCall,
        joinCall,
        leaveCall,
        endCall,
        toggleMute,
        toggleHand,
        toggleScreenShare,
        sendReaction,
        setIncomingCall,
        toggleExpanded,
        initializeWebRTC,
        cleanupWebRTC
    }
})
