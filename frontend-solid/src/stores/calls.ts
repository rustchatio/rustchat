import { createMemo, createSignal } from 'solid-js';
import { authStore } from '@/stores/auth';
import {
  type CallState,
  type CallsConfig,
  fetchCallState,
  fetchCallsConfig,
  fetchTurnCredentials,
  joinCall,
  leaveCall,
  lowerHand,
  muteCall,
  raiseHand,
  sendIceCandidate,
  sendOffer,
  startCall,
  unmuteCall,
  endCall,
} from '@/api/calls';
import { toast } from '@/hooks/useToast';

export type CallMode = 'voice' | 'video';

export interface RemoteMediaStream {
  id: string;
  stream: MediaStream;
  hasVideo: boolean;
}

export interface ActiveCallSession {
  channelId: string;
  mode: CallMode;
  call: CallState;
  mySessionId: string;
  peerConnection: RTCPeerConnection | null;
  localStream: MediaStream | null;
  remoteStreams: RemoteMediaStream[];
}

interface CallsSignalPayload {
  type?: string;
  candidate?: string;
  sdp?: string;
  sdp_mid?: string | null;
  sdp_mline_index?: number | null;
  sdpMid?: string | null;
  sdpMLineIndex?: number | null;
}

const [config, setConfig] = createSignal<CallsConfig | null>(null);
const [activeSession, setActiveSession] = createSignal<ActiveCallSession | null>(null);
const [isStarting, setIsStarting] = createSignal(false);
const [error, setError] = createSignal<string | null>(null);
const [isMuted, setIsMuted] = createSignal(true);
const [isHandRaised, setIsHandRaised] = createSignal(false);
const [isExpanded, setIsExpanded] = createSignal(false);

let callStateRefreshTimer: ReturnType<typeof setTimeout> | null = null;

function asRecord(value: unknown): Record<string, unknown> | null {
  return typeof value === 'object' && value !== null ? (value as Record<string, unknown>) : null;
}

function readEventChannelId(data: unknown, fallback?: string): string | null {
  const record = asRecord(data);
  if (!record) return fallback || null;

  const rawChannel =
    (typeof record.channel_id_raw === 'string' && record.channel_id_raw) ||
    (typeof record.channel_id === 'string' && record.channel_id) ||
    (typeof record.channelID === 'string' && record.channelID);

  return rawChannel || fallback || null;
}

function readSignalPayload(data: unknown): CallsSignalPayload | null {
  const record = asRecord(data);
  if (!record) return null;

  const directSignal = asRecord(record.signal);
  if (directSignal) {
    return directSignal as CallsSignalPayload;
  }

  if (typeof record.data === 'string') {
    try {
      const parsed = JSON.parse(record.data);
      const parsedRecord = asRecord(parsed);
      return parsedRecord ? (parsedRecord as CallsSignalPayload) : null;
    } catch {
      return null;
    }
  }

  return null;
}

function getMySessionId(call: CallState): string {
  const currentUserId = authStore.user()?.id;
  if (!currentUserId) return '';

  const session = Object.values(call.sessions).find((entry) => entry.userId === currentUserId);
  return session?.sessionId || '';
}

function syncSelfFlags(call: CallState, mySessionId: string): void {
  const mySession = mySessionId ? call.sessions[mySessionId] : undefined;
  setIsMuted(mySession ? !mySession.unmuted : true);
  setIsHandRaised(Boolean(mySession && mySession.raisedHand > 0));
}

function removeRemoteStream(streamId: string): void {
  setActiveSession((prev) => {
    if (!prev) return prev;
    return {
      ...prev,
      remoteStreams: prev.remoteStreams.filter((entry) => entry.id !== streamId),
    };
  });
}

function upsertRemoteStream(stream: MediaStream): void {
  setActiveSession((prev) => {
    if (!prev) return prev;
    const hasVideo = stream.getVideoTracks().length > 0;
    const index = prev.remoteStreams.findIndex((entry) => entry.id === stream.id);
    const next = [...prev.remoteStreams];
    const entry: RemoteMediaStream = {
      id: stream.id,
      stream,
      hasVideo,
    };
    if (index >= 0) {
      next[index] = entry;
    } else {
      next.push(entry);
    }

    return {
      ...prev,
      remoteStreams: next,
    };
  });
}

async function loadConfig(): Promise<CallsConfig> {
  const loaded = await fetchCallsConfig();
  let iceServers = loaded.iceServers;

  if (loaded.needsTurnCredentials) {
    try {
      const turnServers = await fetchTurnCredentials();
      if (turnServers.length > 0) {
        iceServers = [...iceServers, ...turnServers];
      }
    } catch (err) {
      console.warn('Failed to fetch TURN credentials', err);
    }
  }

  const resolved: CallsConfig = {
    ...loaded,
    iceServers,
  };
  setConfig(resolved);
  return resolved;
}

async function refreshCurrentCallState(channelId: string): Promise<void> {
  const current = activeSession();
  if (!current || current.channelId !== channelId) return;

  try {
    const updated = await fetchCallState(channelId);
    if (!updated) {
      cleanupCallSession(false);
      return;
    }

    const resolvedSessionId = getMySessionId(updated) || current.mySessionId;
    setActiveSession((prev) => {
      if (!prev || prev.channelId !== channelId) return prev;
      return {
        ...prev,
        call: updated,
        mySessionId: resolvedSessionId,
      };
    });
    syncSelfFlags(updated, resolvedSessionId);
  } catch (err) {
    console.warn('Failed to refresh call state', err);
  }
}

function scheduleCallStateRefresh(channelId: string): void {
  if (callStateRefreshTimer) {
    clearTimeout(callStateRefreshTimer);
  }
  callStateRefreshTimer = setTimeout(() => {
    callStateRefreshTimer = null;
    void refreshCurrentCallState(channelId);
  }, 150);
}

async function initializeWebRtc(
  channelId: string,
  requestedMode: CallMode,
  iceServers: RTCIceServer[]
): Promise<{ peerConnection: RTCPeerConnection; localStream: MediaStream; mode: CallMode }> {
  let mode = requestedMode;
  let localStream: MediaStream;

  try {
    localStream = await navigator.mediaDevices.getUserMedia({
      audio: true,
      video: requestedMode === 'video',
    });
  } catch (err) {
    if (requestedMode === 'video') {
      localStream = await navigator.mediaDevices.getUserMedia({ audio: true, video: false });
      mode = 'voice';
      toast.warning('Camera unavailable', 'Joined as voice-only because camera access failed.');
    } else {
      throw err;
    }
  }

  localStream.getAudioTracks().forEach((track) => {
    track.enabled = false;
  });

  const pc = new RTCPeerConnection({
    iceServers: iceServers.length > 0 ? iceServers : [{ urls: 'stun:stun.l.google.com:19302' }],
  });

  localStream.getTracks().forEach((track) => {
    pc.addTrack(track, localStream);
  });

  pc.ontrack = (event) => {
    const stream = event.streams[0] || new MediaStream([event.track]);
    upsertRemoteStream(stream);

    event.track.onended = () => {
      const current = activeSession();
      if (!current) return;

      const target = current.remoteStreams.find((entry) => entry.id === stream.id);
      if (!target) return;

      const remainingTracks = target.stream
        .getTracks()
        .filter((track) => track.id !== event.track.id);
      if (remainingTracks.length === 0) {
        removeRemoteStream(stream.id);
        return;
      }

      const replacement = new MediaStream(remainingTracks);
      upsertRemoteStream(replacement);
    };
  };

  pc.onicecandidate = (event) => {
    const candidate = event.candidate;
    if (!candidate) return;
    void sendIceCandidate(
      channelId,
      candidate.candidate,
      candidate.sdpMid || undefined,
      candidate.sdpMLineIndex
    ).catch((err) => {
      console.warn('Failed to send ICE candidate', err);
    });
  };

  const offer = await pc.createOffer();
  await pc.setLocalDescription(offer);
  const answer = await sendOffer(channelId, offer.sdp || '');
  await pc.setRemoteDescription({
    type: answer.type as RTCSdpType,
    sdp: answer.sdp,
  });

  return {
    peerConnection: pc,
    localStream,
    mode,
  };
}

function stopMediaTracks(stream: MediaStream | null): void {
  stream?.getTracks().forEach((track) => {
    track.stop();
  });
}

function cleanupCallSession(resetUiState = true): void {
  const current = activeSession();
  if (current?.peerConnection) {
    current.peerConnection.close();
  }

  stopMediaTracks(current?.localStream || null);
  if (callStateRefreshTimer) {
    clearTimeout(callStateRefreshTimer);
    callStateRefreshTimer = null;
  }

  setActiveSession(null);
  if (resetUiState) {
    setIsMuted(true);
    setIsHandRaised(false);
    setIsExpanded(false);
  }
}

async function startOrJoinCall(channelId: string, requestedMode: CallMode): Promise<void> {
  if (isStarting()) return;

  const current = activeSession();
  if (current?.channelId === channelId) {
    setIsExpanded(true);
    return;
  }
  if (current?.channelId && current.channelId !== channelId) {
    await leaveCurrentCall();
  }

  setIsStarting(true);
  setError(null);

  try {
    const resolvedConfig = config() || (await loadConfig());
    const existing = await fetchCallState(channelId);

    if (existing) {
      await joinCall(channelId);
    } else {
      await startCall(channelId);
      await joinCall(channelId).catch(() => {
        // start_call adds owner to participants; join can no-op or 404 in some race windows.
      });
    }

    const call = await fetchCallState(channelId);
    if (!call) {
      throw new Error('Call started but state was not available.');
    }

    const mySessionId = getMySessionId(call);
    if (!mySessionId) {
      throw new Error('Joined call, but local session could not be resolved.');
    }

    setActiveSession({
      channelId,
      mode: requestedMode,
      call,
      mySessionId,
      peerConnection: null,
      localStream: null,
      remoteStreams: [],
    });
    syncSelfFlags(call, mySessionId);

    const rtc = await initializeWebRtc(channelId, requestedMode, resolvedConfig.iceServers);
    setActiveSession((prev) => {
      if (!prev || prev.channelId !== channelId) {
        stopMediaTracks(rtc.localStream);
        rtc.peerConnection.close();
        return prev;
      }
      return {
        ...prev,
        mode: rtc.mode,
        peerConnection: rtc.peerConnection,
        localStream: rtc.localStream,
      };
    });

    setIsExpanded(true);
    toast.success(
      rtc.mode === 'video' ? 'Video call connected' : 'Voice call connected',
      `You are now in a call for this channel.`
    );
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Unable to start call';
    setError(message);
    cleanupCallSession(true);
    toast.error('Unable to connect call', message);
    throw err;
  } finally {
    setIsStarting(false);
  }
}

async function leaveCurrentCall(): Promise<void> {
  const current = activeSession();
  if (!current) return;

  cleanupCallSession(true);
  try {
    await leaveCall(current.channelId);
  } catch (err) {
    console.warn('Failed to leave call cleanly', err);
  }
}

async function endCurrentCall(): Promise<void> {
  const current = activeSession();
  if (!current) return;

  cleanupCallSession(true);
  try {
    await endCall(current.channelId);
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Failed to end call';
    toast.error('Failed to end call', message);
  }
}

async function toggleMute(): Promise<void> {
  const current = activeSession();
  if (!current) return;

  const shouldUnmute = isMuted();
  if (shouldUnmute) {
    await unmuteCall(current.channelId);
  } else {
    await muteCall(current.channelId);
  }

  current.localStream?.getAudioTracks().forEach((track) => {
    track.enabled = shouldUnmute;
  });
  setIsMuted(!shouldUnmute);
}

async function toggleHandRaised(): Promise<void> {
  const current = activeSession();
  if (!current) return;

  if (isHandRaised()) {
    await lowerHand(current.channelId);
  } else {
    await raiseHand(current.channelId);
  }
  setIsHandRaised(!isHandRaised());
}

function handleSignalEvent(data: unknown): void {
  const current = activeSession();
  if (!current?.peerConnection) return;

  const signal = readSignalPayload(data);
  if (!signal?.type) return;

  if (signal.type === 'ice-candidate' && typeof signal.candidate === 'string') {
    void current.peerConnection.addIceCandidate({
      candidate: signal.candidate,
      sdpMid:
        (typeof signal.sdp_mid === 'string' && signal.sdp_mid) ||
        (typeof signal.sdpMid === 'string' && signal.sdpMid) ||
        null,
      sdpMLineIndex:
        typeof signal.sdp_mline_index === 'number'
          ? signal.sdp_mline_index
          : typeof signal.sdpMLineIndex === 'number'
            ? signal.sdpMLineIndex
            : null,
    }).catch((err) => {
      console.warn('Failed to handle ICE candidate signal', err);
    });
    return;
  }

  if (signal.type === 'answer' && typeof signal.sdp === 'string') {
    void current.peerConnection.setRemoteDescription({
      type: 'answer',
      sdp: signal.sdp,
    }).catch((err) => {
      console.warn('Failed to set remote answer from signaling event', err);
    });
  }
}

export function handleCallWebsocketEvent(
  eventName: string,
  data: unknown,
  fallbackChannelId?: string
): void {
  if (!eventName.startsWith('custom_com.mattermost.calls_')) return;

  if (eventName === 'custom_com.mattermost.calls_signal') {
    handleSignalEvent(data);
    return;
  }

  const current = activeSession();
  if (!current) return;

  const eventChannelId = readEventChannelId(data, fallbackChannelId);
  if (!eventChannelId || eventChannelId !== current.channelId) return;

  if (eventName === 'custom_com.mattermost.calls_call_end') {
    cleanupCallSession(true);
    toast.info('Call ended', 'The active call has ended.');
    return;
  }

  scheduleCallStateRefresh(eventChannelId);
}

const participants = createMemo(() => {
  const call = activeSession()?.call;
  return call ? Object.values(call.sessions) : [];
});

const durationSeconds = createMemo(() => {
  const call = activeSession()?.call;
  if (!call) return 0;
  return Math.max(0, Math.floor((Date.now() - call.startAt) / 1000));
});

const isHost = createMemo(() => {
  const current = activeSession();
  const userId = authStore.user()?.id;
  if (!current || !userId) return false;
  return current.call.hostId === userId || current.call.ownerId === userId;
});

const isInCall = createMemo(() => Boolean(activeSession()));

if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', () => {
    const current = activeSession();
    if (current) {
      void leaveCall(current.channelId).catch(() => {
        // noop
      });
    }
  });

  window.addEventListener('rustchat:logout', () => {
    cleanupCallSession(false);
  });
}

export const callsStore = {
  config,
  activeSession,
  isStarting,
  error,
  isMuted,
  isHandRaised,
  isExpanded,
  isInCall,
  isHost,
  participants,
  durationSeconds,
  loadConfig,
  startOrJoinCall,
  leaveCurrentCall,
  endCurrentCall,
  toggleMute,
  toggleHandRaised,
  setExpanded: (next: boolean) => setIsExpanded(next),
};

export default callsStore;
