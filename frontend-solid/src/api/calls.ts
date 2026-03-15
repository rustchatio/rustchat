import { authStore } from '@/stores/auth';

const CALLS_BASE = '/api/v4/plugins/com.mattermost.calls';

interface ApiErrorPayload {
  message?: string;
  detailed_error?: string;
  error?: string;
}

interface RawIceServer {
  urls?: unknown;
  username?: unknown;
  credential?: unknown;
}

interface RawCallsConfig {
  ice_servers_configs?: unknown;
  ICEServersConfigs?: unknown;
  ice_servers?: unknown;
  needs_turn_credentials?: unknown;
  NeedsTURNCredentials?: unknown;
  allow_screen_sharing?: unknown;
  AllowScreenSharing?: unknown;
  enable_simulcast?: unknown;
  EnableSimulcast?: unknown;
}

interface RawCallSession {
  session_id?: unknown;
  session_id_raw?: unknown;
  user_id?: unknown;
  user_id_raw?: unknown;
  username?: unknown;
  display_name?: unknown;
  unmuted?: unknown;
  raised_hand?: unknown;
}

interface RawCallState {
  id?: unknown;
  id_raw?: unknown;
  channel_id?: unknown;
  channel_id_raw?: unknown;
  start_at?: unknown;
  owner_id?: unknown;
  owner_id_raw?: unknown;
  host_id?: unknown;
  host_id_raw?: unknown;
  screen_sharing_id?: unknown;
  screen_sharing_id_raw?: unknown;
  screen_sharing_session_id?: unknown;
  screen_sharing_session_id_raw?: unknown;
  sessions?: unknown;
}

export interface CallSession {
  sessionId: string;
  userId: string;
  username?: string;
  displayName?: string;
  unmuted: boolean;
  raisedHand: number;
}

export interface CallState {
  id: string;
  channelId: string;
  startAt: number;
  ownerId: string;
  hostId: string;
  screenSharingUserId?: string;
  screenSharingSessionId?: string;
  sessions: Record<string, CallSession>;
}

export interface CallsConfig {
  iceServers: RTCIceServer[];
  needsTurnCredentials: boolean;
  allowScreenSharing: boolean;
  enableSimulcast: boolean;
}

export interface OfferAnswer {
  sdp: string;
  type: string;
}

type JsonRecord = Record<string, unknown>;

function isRecord(value: unknown): value is JsonRecord {
  return typeof value === 'object' && value !== null;
}

function asNonEmptyString(value: unknown): string | undefined {
  return typeof value === 'string' && value.trim().length > 0 ? value : undefined;
}

function asBoolean(value: unknown, fallback: boolean): boolean {
  return typeof value === 'boolean' ? value : fallback;
}

function asNumber(value: unknown, fallback: number): number {
  return typeof value === 'number' && Number.isFinite(value) ? value : fallback;
}

function normalizeIceServer(raw: unknown): RTCIceServer | null {
  if (!isRecord(raw)) return null;

  const urlsRaw = raw.urls;
  const urls = Array.isArray(urlsRaw)
    ? urlsRaw.filter((item): item is string => typeof item === 'string' && item.length > 0)
    : typeof urlsRaw === 'string' && urlsRaw.length > 0
      ? [urlsRaw]
      : [];

  if (urls.length === 0) return null;

  const server: RTCIceServer = {
    urls,
  };

  const username = asNonEmptyString(raw.username);
  const credential = asNonEmptyString(raw.credential);
  if (username) server.username = username;
  if (credential) server.credential = credential;
  return server;
}

function normalizeIceServers(config: RawCallsConfig): RTCIceServer[] {
  const candidates = [
    config.ice_servers_configs,
    config.ICEServersConfigs,
    config.ice_servers,
  ];

  for (const entry of candidates) {
    if (!Array.isArray(entry)) continue;
    const servers = entry.map(normalizeIceServer).filter((item): item is RTCIceServer => item !== null);
    if (servers.length > 0) return servers;
  }

  return [];
}

function normalizeCallState(raw: unknown, fallbackChannelId: string): CallState | null {
  if (!isRecord(raw)) return null;
  const input = raw as RawCallState;

  const channelId =
    asNonEmptyString(input.channel_id_raw) ||
    asNonEmptyString(input.channel_id) ||
    fallbackChannelId;

  const id = asNonEmptyString(input.id_raw) || asNonEmptyString(input.id) || '';
  const startAt = asNumber(input.start_at, Date.now());
  const ownerId = asNonEmptyString(input.owner_id_raw) || asNonEmptyString(input.owner_id) || '';
  const hostId = asNonEmptyString(input.host_id_raw) || asNonEmptyString(input.host_id) || '';

  const sessions: Record<string, CallSession> = {};
  if (isRecord(input.sessions)) {
    Object.entries(input.sessions).forEach(([key, value]) => {
      if (!isRecord(value)) return;
      const session = value as RawCallSession;
      const sessionId = asNonEmptyString(session.session_id) || asNonEmptyString(session.session_id_raw) || key;
      const userId = asNonEmptyString(session.user_id_raw) || asNonEmptyString(session.user_id) || '';
      sessions[sessionId] = {
        sessionId,
        userId,
        username: asNonEmptyString(session.username),
        displayName: asNonEmptyString(session.display_name),
        unmuted: asBoolean(session.unmuted, false),
        raisedHand: asNumber(session.raised_hand, 0),
      };
    });
  }

  return {
    id,
    channelId,
    startAt,
    ownerId,
    hostId,
    screenSharingUserId:
      asNonEmptyString(input.screen_sharing_id_raw) || asNonEmptyString(input.screen_sharing_id),
    screenSharingSessionId:
      asNonEmptyString(input.screen_sharing_session_id_raw) ||
      asNonEmptyString(input.screen_sharing_session_id),
    sessions,
  };
}

async function parseError(response: Response): Promise<string> {
  const fallback = `Calls request failed (${response.status})`;
  try {
    const payload = (await response.json()) as ApiErrorPayload;
    return payload.message || payload.detailed_error || payload.error || fallback;
  } catch {
    return fallback;
  }
}

async function requestJson<T>(path: string, init?: RequestInit): Promise<T> {
  const token = authStore.token;
  const headers = new Headers(init?.headers || {});
  if (token) headers.set('Authorization', `Bearer ${token}`);
  if (init?.body && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json');
  }

  const response = await fetch(`${CALLS_BASE}${path}`, {
    ...init,
    headers,
  });

  if (!response.ok) {
    throw new Error(await parseError(response));
  }

  return (await response.json()) as T;
}

export async function fetchCallsConfig(): Promise<CallsConfig> {
  const raw = await requestJson<RawCallsConfig>('/config');
  return {
    iceServers: normalizeIceServers(raw),
    needsTurnCredentials: asBoolean(raw.needs_turn_credentials ?? raw.NeedsTURNCredentials, false),
    allowScreenSharing: asBoolean(raw.allow_screen_sharing ?? raw.AllowScreenSharing, true),
    enableSimulcast: asBoolean(raw.enable_simulcast ?? raw.EnableSimulcast, false),
  };
}

export async function fetchTurnCredentials(): Promise<RTCIceServer[]> {
  const raw = await requestJson<RawIceServer[]>('/turn-credentials');
  if (!Array.isArray(raw)) return [];
  return raw.map(normalizeIceServer).filter((server): server is RTCIceServer => server !== null);
}

export async function fetchCallState(channelId: string): Promise<CallState | null> {
  const token = authStore.token;
  const headers = new Headers();
  if (token) headers.set('Authorization', `Bearer ${token}`);

  const response = await fetch(
    `${CALLS_BASE}/calls/${encodeURIComponent(channelId)}?mobilev2=true`,
    { headers }
  );

  if (response.status === 404) {
    return null;
  }

  if (!response.ok) {
    throw new Error(await parseError(response));
  }

  const raw = (await response.json()) as unknown;
  return normalizeCallState(raw, channelId);
}

export async function startCall(channelId: string): Promise<void> {
  await requestJson<unknown>(`/calls/${encodeURIComponent(channelId)}/start`, { method: 'POST' });
}

export async function joinCall(channelId: string): Promise<void> {
  await requestJson<unknown>(`/calls/${encodeURIComponent(channelId)}/join`, { method: 'POST' });
}

export async function leaveCall(channelId: string): Promise<void> {
  await requestJson<unknown>(`/calls/${encodeURIComponent(channelId)}/leave`, { method: 'POST' });
}

export async function endCall(channelId: string): Promise<void> {
  await requestJson<unknown>(`/calls/${encodeURIComponent(channelId)}/end`, { method: 'POST' });
}

export async function muteCall(channelId: string): Promise<void> {
  await requestJson<unknown>(`/calls/${encodeURIComponent(channelId)}/mute`, { method: 'POST' });
}

export async function unmuteCall(channelId: string): Promise<void> {
  await requestJson<unknown>(`/calls/${encodeURIComponent(channelId)}/unmute`, { method: 'POST' });
}

export async function raiseHand(channelId: string): Promise<void> {
  await requestJson<unknown>(`/calls/${encodeURIComponent(channelId)}/raise-hand`, { method: 'POST' });
}

export async function lowerHand(channelId: string): Promise<void> {
  await requestJson<unknown>(`/calls/${encodeURIComponent(channelId)}/lower-hand`, { method: 'POST' });
}

export async function toggleScreenShare(channelId: string): Promise<void> {
  await requestJson<unknown>(`/calls/${encodeURIComponent(channelId)}/screen-share`, { method: 'POST' });
}

export async function hostMuteParticipant(channelId: string, sessionId: string): Promise<void> {
  await requestJson<unknown>(`/calls/${encodeURIComponent(channelId)}/host/mute`, {
    method: 'POST',
    body: JSON.stringify({ session_id: sessionId }),
  });
}

export async function hostMuteOthers(channelId: string): Promise<void> {
  await requestJson<unknown>(`/calls/${encodeURIComponent(channelId)}/host/mute-others`, {
    method: 'POST',
  });
}

export async function hostRemoveParticipant(channelId: string, sessionId: string): Promise<void> {
  await requestJson<unknown>(`/calls/${encodeURIComponent(channelId)}/host/remove`, {
    method: 'POST',
    body: JSON.stringify({ session_id: sessionId }),
  });
}

export async function hostLowerHandParticipant(channelId: string, sessionId: string): Promise<void> {
  await requestJson<unknown>(`/calls/${encodeURIComponent(channelId)}/host/lower-hand`, {
    method: 'POST',
    body: JSON.stringify({ session_id: sessionId }),
  });
}

export async function dismissCallNotification(channelId: string): Promise<void> {
  await requestJson<unknown>(`/calls/${encodeURIComponent(channelId)}/dismiss-notification`, {
    method: 'POST',
  });
}

export async function ringCall(channelId: string): Promise<void> {
  await requestJson<unknown>(`/calls/${encodeURIComponent(channelId)}/ring`, {
    method: 'POST',
  });
}

export async function sendOffer(channelId: string, sdp: string): Promise<OfferAnswer> {
  const response = await requestJson<{ sdp?: string; type_?: string }>(
    `/calls/${encodeURIComponent(channelId)}/offer`,
    {
      method: 'POST',
      body: JSON.stringify({ sdp }),
    }
  );

  return {
    sdp: response.sdp || '',
    type: response.type_ || 'answer',
  };
}

export async function sendIceCandidate(
  channelId: string,
  candidate: string,
  sdpMid?: string,
  sdpMLineIndex?: number | null
): Promise<void> {
  await requestJson<unknown>(`/calls/${encodeURIComponent(channelId)}/ice`, {
    method: 'POST',
    body: JSON.stringify({
      candidate,
      sdp_mid: sdpMid,
      sdp_mline_index:
        typeof sdpMLineIndex === 'number' ? sdpMLineIndex : undefined,
    }),
  });
}
