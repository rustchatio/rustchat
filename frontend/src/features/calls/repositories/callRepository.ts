// Call Repository - Data access for calls
// Maps API responses to domain entities

import callsApi from '../../../api/calls'
import type { 
  CallState, 
  CallConfig, 
  CallId, 
  CallParticipant, 
  SessionId,
  createCallId,
  createSessionId
} from '../../../core/entities/Call'
import type { ChannelId } from '../../../core/entities/Channel'
import type { UserId } from '../../../core/entities/User'
import { withRetry } from '../../../core/services/retry'
import { AppError } from '../../../core/errors/AppError'

export interface CallChannelState {
  channelId: ChannelId
  enabled: boolean
  call?: CallState
}

export const callRepository = {
  // Config
  async getConfig(): Promise<CallConfig> {
    return withRetry(async () => {
      const response = await callsApi.getConfig()
      const data = response.data
      
      // If TURN is needed, fetch credentials
      let iceServers = data.ICEServersConfigs
      if (data.NeedsTURNCredentials) {
        try {
          const turnResponse = await callsApi.getTurnCredentials()
          iceServers = [
            ...data.ICEServersConfigs.filter(s => {
              const urls = Array.isArray(s.urls) ? s.urls : [s.urls]
              return !urls.some(url => url.toString().startsWith('turn:'))
            }),
            ...turnResponse.data
          ]
        } catch (error) {
          console.error('Failed to fetch TURN credentials', error)
        }
      }

      return {
        iceServers,
        allowEnableCalls: data.AllowEnableCalls,
        defaultEnabled: data.DefaultEnabled,
        needsTURNCredentials: data.NeedsTURNCredentials,
        maxParticipants: data.MaxCallParticipants,
        allowScreenSharing: data.AllowScreenSharing,
        enableSimulcast: data.EnableSimulcast,
        enableRinging: data.EnableRinging,
        enableLiveCaptions: data.EnableLiveCaptions,
        hostControlsAllowed: data.HostControlsAllowed,
        enableRecordings: data.EnableRecordings,
        maxRecordingDuration: data.MaxRecordingDuration,
        groupCallsAllowed: data.GroupCallsAllowed
      }
    })
  },

  // Active calls
  async getActiveCalls(): Promise<CallChannelState[]> {
    return withRetry(async () => {
      const response = await callsApi.getCalls()
      return response.data.map(channelState => ({
        channelId: channelState.channel_id as ChannelId,
        enabled: channelState.enabled,
        call: channelState.call ? normalizeCallState(channelState.call) : undefined
      }))
    })
  },

  async getCallForChannel(channelId: ChannelId): Promise<CallChannelState | null> {
    return withRetry(async () => {
      try {
        const response = await callsApi.getCallForChannel(channelId)
        if (!response.data) return null
        
        return {
          channelId: response.data.channel_id as ChannelId,
          enabled: response.data.enabled,
          call: response.data.call ? normalizeCallState(response.data.call) : undefined
        }
      } catch (error: any) {
        if (error?.response?.status === 404) {
          return null
        }
        throw error
      }
    })
  },

  // Call lifecycle
  async startCall(channelId: ChannelId): Promise<CallState> {
    return withRetry(async () => {
      const response = await callsApi.startCall(channelId)
      
      // Fetch the full call state after starting
      const channelState = await this.getCallForChannel(channelId)
      if (!channelState?.call) {
        throw new AppError('Call started but state could not be loaded', 'CALL_STATE_ERROR')
      }
      
      return channelState.call
    }, { maxAttempts: 2 })
  },

  async joinCall(channelId: ChannelId): Promise<void> {
    await withRetry(() => callsApi.joinCall(channelId))
  },

  async leaveCall(channelId: ChannelId): Promise<void> {
    await withRetry(() => callsApi.leaveCall(channelId))
  },

  async endCall(channelId: ChannelId): Promise<void> {
    await withRetry(() => callsApi.endCall(channelId))
  },

  // Media controls
  async mute(channelId: ChannelId): Promise<void> {
    await withRetry(() => callsApi.mute(channelId))
  },

  async unmute(channelId: ChannelId): Promise<void> {
    await withRetry(() => callsApi.unmute(channelId))
  },

  async raiseHand(channelId: ChannelId): Promise<void> {
    await withRetry(() => callsApi.raiseHand(channelId))
  },

  async lowerHand(channelId: ChannelId): Promise<void> {
    await withRetry(() => callsApi.lowerHand(channelId))
  },

  async toggleScreenShare(channelId: ChannelId): Promise<void> {
    await withRetry(() => callsApi.toggleScreenShare(channelId))
  },

  async sendReaction(channelId: ChannelId, emoji: string): Promise<void> {
    await withRetry(() => callsApi.sendReaction(channelId, emoji))
  },

  // WebRTC Signaling
  async sendOffer(channelId: ChannelId, sdp: string): Promise<{ sdp: string; type: string }> {
    const response = await withRetry(() => callsApi.sendOffer(channelId, sdp))
    return response.data
  },

  async sendIceCandidate(
    channelId: ChannelId, 
    candidate: string, 
    sdpMid?: string, 
    sdpMLineIndex?: number
  ): Promise<void> {
    await withRetry(() => callsApi.sendIceCandidate(channelId, candidate, sdpMid, sdpMLineIndex))
  },

  // Host controls
  async hostMute(channelId: ChannelId, sessionId: SessionId): Promise<void> {
    await withRetry(() => callsApi.hostMute(channelId, sessionId))
  },

  async hostMuteOthers(channelId: ChannelId): Promise<void> {
    await withRetry(() => callsApi.hostMuteOthers(channelId))
  },

  async hostRemove(channelId: ChannelId, sessionId: SessionId): Promise<void> {
    await withRetry(() => callsApi.hostRemove(channelId, sessionId))
  },

  async hostMakeHost(channelId: ChannelId, newHostId: UserId): Promise<void> {
    await withRetry(() => callsApi.hostMakeHost(channelId, newHostId))
  },

  // Ringing
  async ringUsers(channelId: ChannelId): Promise<void> {
    await withRetry(() => callsApi.ringUsers(channelId))
  }
}

// Normalize API CallState to domain entity
function normalizeCallState(raw: any): CallState {
  const participants = new Map<SessionId, CallParticipant>()
  
  if (raw.sessions) {
    for (const [key, session] of Object.entries(raw.sessions)) {
      const s = session as any
      const sessionId = createSessionId(s.session_id || key)
      participants.set(sessionId, {
        sessionId,
        userId: (s.user_id_raw || s.user_id) as UserId,
        username: s.username || '',
        displayName: s.display_name,
        isMuted: !s.unmuted,
        isSpeaking: false, // Will be updated by WebSocket events
        isScreenSharing: false, // Will be updated by WebSocket events
        raisedHandAt: s.raised_hand || 0,
        joinedAt: new Date(raw.start_at || Date.now())
      })
    }
  }

  return {
    id: createCallId(raw.id || raw.id_raw || ''),
    channelId: (raw.channel_id_raw || raw.channel_id) as ChannelId,
    startedAt: new Date(raw.start_at || Date.now()),
    startedBy: (raw.owner_id_raw || raw.owner_id) as UserId,
    hostId: (raw.host_id_raw || raw.host_id) as UserId,
    participants,
    screenSharingSessionId: raw.screen_sharing_session_id 
      ? createSessionId(raw.screen_sharing_session_id)
      : undefined,
    threadId: raw.thread_id,
    recording: raw.recording ? {
      isRecording: true,
      startedAt: new Date(raw.recording.start_at),
      startedBy: raw.owner_id as UserId
    } : undefined
  }
}
