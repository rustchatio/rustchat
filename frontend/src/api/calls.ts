// Mattermost Calls Plugin API Client
// Routes are mounted under /api/v4/plugins/com.mattermost.calls

import axios from 'axios'
import { useAuthStore } from '../stores/auth'

// Create a separate axios instance for v4 API calls
// (the default client uses /api/v1 as base)
const apiClient = axios.create({
    baseURL: '/api/v4',
})

// Add auth interceptor
apiClient.interceptors.request.use(config => {
    const authStore = useAuthStore()
    if (authStore.token) {
        config.headers.Authorization = `Bearer ${authStore.token}`
    }
    return config
})

// Types from Mattermost Calls
export interface CallsConfig {
    ICEServersConfigs: RTCIceServer[]
    AllowEnableCalls: boolean
    DefaultEnabled: boolean
    NeedsTURNCredentials: boolean
    MaxCallParticipants: number
    AllowScreenSharing: boolean
    EnableSimulcast: boolean
    EnableRinging: boolean
    EnableLiveCaptions: boolean
    HostControlsAllowed: boolean
    EnableRecordings: boolean
    MaxRecordingDuration: number
    GroupCallsAllowed: boolean
}

export interface CallsVersionInfo {
    version?: string
    rtcd?: boolean
}

export interface CallState {
    id: string
    channel_id: string
    start_at: number
    owner_id: string
    host_id: string
    thread_id?: string
    screen_sharing_session_id?: string
    recording?: CallJobState
    dismissed_notification?: Record<string, boolean>
    sessions: Record<string, CallSession>
}

export interface CallSession {
    session_id: string
    user_id: string
    unmuted: boolean
    raised_hand: number
}

export interface CallJobState {
    start_at: number
    end_at: number
}

export interface CallChannelState {
    channel_id: string
    enabled: boolean
    call?: CallState
}

export interface StartCallResponse {
    id: string
    channel_id: string
    start_at: number
    owner_id: string
}

export interface ApiResp {
    message?: string
    detailed_error?: string
    status_code: number
}

export interface CreateMeetingResponse {
    meeting_url: string
    mode: 'new_tab' | 'embed_iframe'
}

const CALLS_ROUTE = '/plugins/com.mattermost.calls'

export default {
    // Check if calls plugin is enabled
    async getEnabled(): Promise<boolean> {
        try {
            await apiClient.get(`${CALLS_ROUTE}/version`)
            return true
        } catch (e) {
            return false
        }
    },

    // Get calls plugin version
    getVersion() {
        return apiClient.get<CallsVersionInfo>(`${CALLS_ROUTE}/version`)
    },

    // Get calls config (ICE servers, etc)
    getConfig() {
        return apiClient.get<CallsConfig>(`${CALLS_ROUTE}/config`)
    },

    // Get all active calls
    getCalls() {
        return apiClient.get<CallChannelState[]>(`${CALLS_ROUTE}/channels?mobilev2=true`)
    },

    // Get call for specific channel
    getCallForChannel(channelId: string) {
        return apiClient.get<CallChannelState>(`${CALLS_ROUTE}/${channelId}?mobilev2=true`)
    },

    // Start a new call in a channel
    startCall(channelId: string) {
        return apiClient.post<StartCallResponse>(`${CALLS_ROUTE}/calls/${channelId}/start`)
    },

    // Join an existing call
    joinCall(channelId: string) {
        return apiClient.post<ApiResp>(`${CALLS_ROUTE}/calls/${channelId}/join`)
    },

    // Leave a call
    leaveCall(channelId: string) {
        return apiClient.post<ApiResp>(`${CALLS_ROUTE}/calls/${channelId}/leave`)
    },

    // End a call (host only)
    endCall(channelId: string) {
        return apiClient.post<ApiResp>(`${CALLS_ROUTE}/calls/${channelId}/end`)
    },

    // Mute self
    mute(channelId: string) {
        return apiClient.post<ApiResp>(`${CALLS_ROUTE}/calls/${channelId}/mute`)
    },

    // Unmute self
    unmute(channelId: string) {
        return apiClient.post<ApiResp>(`${CALLS_ROUTE}/calls/${channelId}/unmute`)
    },

    // Raise hand
    raiseHand(channelId: string) {
        return apiClient.post<ApiResp>(`${CALLS_ROUTE}/calls/${channelId}/raise-hand`)
    },

    // Lower hand
    lowerHand(channelId: string) {
        return apiClient.post<ApiResp>(`${CALLS_ROUTE}/calls/${channelId}/lower-hand`)
    },

    // Send reaction
    sendReaction(channelId: string, emoji: string) {
        return apiClient.post<ApiResp>(`${CALLS_ROUTE}/calls/${channelId}/react`, { emoji })
    },

    // Toggle screen share
    toggleScreenShare(channelId: string) {
        return apiClient.post<ApiResp>(`${CALLS_ROUTE}/calls/${channelId}/screen-share`)
    },

    // WebRTC Signaling
    sendOffer(channelId: string, sdp: string) {
        return apiClient.post<{ sdp: string; type_: string }>(`${CALLS_ROUTE}/calls/${channelId}/offer`, { sdp })
    },

    sendIceCandidate(channelId: string, candidate: string, sdpMid?: string, sdpMLineIndex?: number) {
        return apiClient.post<ApiResp>(`${CALLS_ROUTE}/calls/${channelId}/ice`, {
            candidate,
            sdp_mid: sdpMid,
            sdp_mline_index: sdpMLineIndex
        })
    },

    // Host controls
    hostMakeHost(channelId: string, newHostId: string) {
        return apiClient.post<ApiResp>(`${CALLS_ROUTE}/calls/${channelId}/host/make`, { new_host_id: newHostId })
    },

    hostMute(channelId: string, sessionId: string) {
        return apiClient.post<ApiResp>(`${CALLS_ROUTE}/calls/${channelId}/host/mute`, { session_id: sessionId })
    },

    hostMuteOthers(channelId: string) {
        return apiClient.post<ApiResp>(`${CALLS_ROUTE}/calls/${channelId}/host/mute-others`)
    },

    hostScreenOff(channelId: string, sessionId: string) {
        return apiClient.post<ApiResp>(`${CALLS_ROUTE}/calls/${channelId}/host/screen-off`, { session_id: sessionId })
    },

    hostLowerHand(channelId: string, sessionId: string) {
        return apiClient.post<ApiResp>(`${CALLS_ROUTE}/calls/${channelId}/host/lower-hand`, { session_id: sessionId })
    },

    hostRemove(channelId: string, sessionId: string) {
        return apiClient.post<ApiResp>(`${CALLS_ROUTE}/calls/${channelId}/host/remove`, { session_id: sessionId })
    },

    // Recording
    startRecording(channelId: string) {
        return apiClient.post<ApiResp>(`${CALLS_ROUTE}/calls/${channelId}/recording/start`)
    },

    stopRecording(channelId: string) {
        return apiClient.post<ApiResp>(`${CALLS_ROUTE}/calls/${channelId}/recording/stop`)
    },

    // Enable/disable calls in channel (admin)
    enableChannelCalls(channelId: string, enable: boolean) {
        return apiClient.post<CallChannelState>(`${CALLS_ROUTE}/${channelId}`, { enabled: enable })
    },

    // Legacy MiroTalk video meetings (kept for backward compatibility)
    createMeeting(scope: 'channel' | 'dm', channelId?: string, dmUserId?: string) {
        return apiClient.post<CreateMeetingResponse>('/api/v4/video/meetings', {
            scope,
            channel_id: channelId,
            dm_user_id: dmUserId
        })
    },
}
