import { createSignal, batch, createEffect, onCleanup } from 'solid-js';
import { authStore } from '../stores/auth';
import type {
  WsEnvelope,
  ClientEnvelope,
  ServerEvent,
  ConnectionState,
  EventHandler,
} from './events';

// ============================================
// Configuration
// ============================================

const WS_URL = import.meta.env.VITE_WS_URL || '/api/v4/websocket';
const MAX_RECONNECT_ATTEMPTS = 10;
const INITIAL_RECONNECT_DELAY = 1000;
const MAX_RECONNECT_DELAY = 30000;

// ============================================
// State
// ============================================

const [socket, setSocket] = createSignal<WebSocket | null>(null);
const [connectionState, setConnectionState] = createSignal<ConnectionState>({
  status: 'disconnected',
  reconnectAttempts: 0,
});

const [subscriptions, setSubscriptions] = createSignal<Set<string>>(new Set());
const eventListeners = new Map<ServerEvent, Set<EventHandler>>();
const genericListeners = new Set<EventHandler>();

let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let actionSeq = 1;
let isIntentionalClose = false;

// ============================================
// Connection State Helpers
// ============================================

export const isConnected = () => connectionState().status === 'connected';
export const isConnecting = () => connectionState().status === 'connecting';
export const isReconnecting = () => connectionState().status === 'reconnecting';
export { connectionState };

// ============================================
// Reconnection Logic
// ============================================

function getReconnectDelay(attempt: number): number {
  // Exponential backoff with jitter
  const baseDelay = Math.min(INITIAL_RECONNECT_DELAY * Math.pow(1.5, attempt), MAX_RECONNECT_DELAY);
  const jitter = Math.random() * 1000;
  return baseDelay + jitter;
}

function clearReconnectTimer() {
  if (reconnectTimer) {
    clearTimeout(reconnectTimer);
    reconnectTimer = null;
  }
}

function scheduleReconnect() {
  clearReconnectTimer();

  const state = connectionState();
  if (state.reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) {
    setConnectionState({
      status: 'error',
      reconnectAttempts: state.reconnectAttempts,
      lastError: 'Max reconnection attempts reached',
    });
    return;
  }

  const delay = getReconnectDelay(state.reconnectAttempts);

  batch(() => {
    setConnectionState({
      status: 'reconnecting',
      reconnectAttempts: state.reconnectAttempts + 1,
    });
  });

  console.log(`[WebSocket] Reconnecting in ${Math.round(delay)}ms (attempt ${state.reconnectAttempts + 1})`);

  reconnectTimer = setTimeout(() => {
    connect();
  }, delay);
}

// ============================================
// Event Handlers
// ============================================

function isAuthExpiryCloseEvent(event: CloseEvent): boolean {
  const reason = (event.reason || '').toLowerCase();
  return (
    (event.code === 1008 && reason.includes('token')) ||
    reason.includes('authentication token expired') ||
    reason.includes('token expired')
  );
}

function normalizeEnvelope(envelope: WsEnvelope): WsEnvelope {
  // Ensure data is always an object
  if (typeof envelope.data !== 'object' || envelope.data === null) {
    envelope.data = {};
  }

  // Normalize channel_id from broadcast if not present
  if (!envelope.channel_id && envelope.broadcast?.channel_id) {
    envelope.channel_id = envelope.broadcast.channel_id;
  }

  return envelope;
}

function handleMessage(event: MessageEvent) {
  try {
    const rawEnvelope: WsEnvelope = JSON.parse(event.data);
    const envelope = normalizeEnvelope(rawEnvelope);

    // Notify generic listeners first
    genericListeners.forEach((handler) => {
      try {
        handler(envelope.data, envelope);
      } catch (err) {
        console.error('[WebSocket] Generic listener error:', err);
      }
    });

    // Notify event-specific listeners
    const listeners = getEventListeners(envelope.event as ServerEvent);
    listeners.forEach((handler) => {
      try {
        handler(envelope.data, envelope);
      } catch (err) {
        console.error(`[WebSocket] Event listener error for ${envelope.event}:`, err);
      }
    });

    // Handle special events
    handleSpecialEvents(envelope);
  } catch (err) {
    console.error('[WebSocket] Failed to parse message:', err);
  }
}

function handleSpecialEvents(envelope: WsEnvelope) {
  switch (envelope.event) {
    case 'hello':
      console.log('[WebSocket] Server hello received:', envelope.data);
      break;

    case 'error':
      console.error('[WebSocket] Server error:', envelope.data);
      break;

    case 'initial_load':
      // Resubscribe to channels after initial load
      resubscribeAll();
      break;
  }
}

// ============================================
// Connection Management
// ============================================

export function connect(): void {
  // Don't connect if no auth token
  if (!authStore.token) {
    console.log('[WebSocket] No auth token, skipping connection');
    return;
  }

  // Don't connect if already connected
  if (socket()?.readyState === WebSocket.OPEN) {
    return;
  }

  // Don't connect if already connecting
  if (socket()?.readyState === WebSocket.CONNECTING) {
    return;
  }

  isIntentionalClose = false;

  batch(() => {
    setConnectionState((prev) => ({
      ...prev,
      status: 'connecting',
    }));
  });

  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const host = window.location.host;
  const url = `${protocol}//${host}${WS_URL}`;

  try {
    // Browsers can't set Authorization headers for WebSocket handshakes
    // Pass bearer token via Sec-WebSocket-Protocol
    const newSocket = new WebSocket(url, [authStore.token]);

    newSocket.onopen = () => {
      const wasReconnecting = connectionState().reconnectAttempts > 0;

      batch(() => {
        setConnectionState({
          status: 'connected',
          reconnectAttempts: 0,
          connectedAt: new Date(),
        });
      });

      console.log('[WebSocket] Connected');

      // Resubscribe to channels
      resubscribeAll();

      // If this was a reconnect, send reconnect action
      if (wasReconnecting) {
        sendAction('reconnect', {});
      }
    };

    newSocket.onclose = (event) => {
      setSocket(null);

      // Check for auth expiry
      if (isAuthExpiryCloseEvent(event)) {
        clearReconnectTimer();
        batch(() => {
          setConnectionState({
            status: 'error',
            reconnectAttempts: 0,
            lastError: 'Authentication expired',
          });
        });
        // Logout
        import('../stores/auth').then(({ logout }) => logout('expired'));
        return;
      }

      // If intentional close, don't reconnect
      if (isIntentionalClose) {
        batch(() => {
          setConnectionState({
            status: 'disconnected',
            reconnectAttempts: 0,
          });
        });
        return;
      }

      // Schedule reconnect
      scheduleReconnect();
    };

    newSocket.onerror = (error) => {
      console.error('[WebSocket] Error:', error);
      batch(() => {
        setConnectionState((prev) => ({
          ...prev,
          lastError: 'Connection error',
        }));
      });
    };

    newSocket.onmessage = handleMessage;

    setSocket(newSocket);
  } catch (err) {
    console.error('[WebSocket] Failed to create connection:', err);
    batch(() => {
      setConnectionState({
        status: 'error',
        reconnectAttempts: connectionState().reconnectAttempts,
        lastError: 'Failed to create connection',
      });
    });
  }
}

export function disconnect(): void {
  isIntentionalClose = true;
  clearReconnectTimer();

  const ws = socket();
  if (ws) {
    ws.close();
    setSocket(null);
  }

  batch(() => {
    setConnectionState({
      status: 'disconnected',
      reconnectAttempts: 0,
    });
    setSubscriptions(new Set<string>());
  });
}

export function reconnect(): void {
  disconnect();
  connect();
}

// ============================================
// Subscription Management
// ============================================

function resubscribeAll() {
  const subs = subscriptions();
  subs.forEach((channelId) => {
    send({
      type: 'command',
      event: 'subscribe_channel',
      channel_id: channelId,
      data: {},
    });
  });
}

export function subscribe(channelId: string): void {
  const subs = subscriptions();
  if (subs.has(channelId)) return;

  const newSubs = new Set(subs);
  newSubs.add(channelId);
  setSubscriptions(newSubs);

  send({
    type: 'command',
    event: 'subscribe_channel',
    channel_id: channelId,
    data: {},
  });
}

export function unsubscribe(channelId: string): void {
  const subs = subscriptions();
  if (!subs.has(channelId)) return;

  const newSubs = new Set(subs);
  newSubs.delete(channelId);
  setSubscriptions(newSubs);

  send({
    type: 'command',
    event: 'unsubscribe_channel',
    channel_id: channelId,
    data: {},
  });
}

export function unsubscribeAll(): void {
  const subs = subscriptions();
  subs.forEach((channelId) => {
    send({
      type: 'command',
      event: 'unsubscribe_channel',
      channel_id: channelId,
      data: {},
    });
  });
  setSubscriptions(new Set<string>());
}

// ============================================
// Event Listeners
// ============================================

function getEventListeners(event: ServerEvent): Set<EventHandler> {
  if (!eventListeners.has(event)) {
    eventListeners.set(event, new Set());
  }
  return eventListeners.get(event)!;
}

export function on<T = unknown>(event: ServerEvent, handler: EventHandler<T>): () => void {
  const listeners = getEventListeners(event);
  listeners.add(handler as EventHandler);

  // Return unsubscribe function
  return () => {
    listeners.delete(handler as EventHandler);
  };
}

export function off<T = unknown>(event: ServerEvent, handler: EventHandler<T>): void {
  const listeners = getEventListeners(event);
  listeners.delete(handler as EventHandler);
}

export function onAny(handler: EventHandler): () => void {
  genericListeners.add(handler);
  return () => {
    genericListeners.delete(handler);
  };
}

export function offAny(handler: EventHandler): void {
  genericListeners.delete(handler);
}

// ============================================
// Message Sending
// ============================================

export function send(envelope: ClientEnvelope): boolean {
  const ws = socket();
  if (!ws || ws.readyState !== WebSocket.OPEN) {
    console.warn('[WebSocket] Cannot send, not connected');
    return false;
  }

  try {
    ws.send(JSON.stringify(envelope));
    return true;
  } catch (err) {
    console.error('[WebSocket] Failed to send message:', err);
    return false;
  }
}

export function sendAction(action: string, data: Record<string, unknown>): boolean {
  const ws = socket();
  if (!ws || ws.readyState !== WebSocket.OPEN) {
    return false;
  }

  try {
    ws.send(
      JSON.stringify({
        action,
        seq: actionSeq++,
        data,
      })
    );
    return true;
  } catch (err) {
    console.error('[WebSocket] Failed to send action:', err);
    return false;
  }
}

// ============================================
// Typing Indicators
// ============================================

export function sendTyping(channelId: string, threadRootId?: string): boolean {
  return sendAction('user_typing', {
    channel_id: channelId,
    parent_id: threadRootId,
  });
}

export function sendStopTyping(channelId: string, threadRootId?: string): boolean {
  return sendAction('user_typing_stop', {
    channel_id: channelId,
    parent_id: threadRootId,
  });
}

// ============================================
// Presence
// ============================================

export function sendPresence(status: string): boolean {
  return send({
    type: 'command',
    event: 'presence',
    data: { status },
  });
}

// ============================================
// Auto-connect Effect
// ============================================

export function createAutoConnect() {
  createEffect(() => {
    const isAuth = authStore.isAuthenticated;

    if (isAuth) {
      connect();
    } else {
      disconnect();
    }
  });

  onCleanup(() => {
    disconnect();
  });
}

// ============================================
// Exports
// ============================================

export const websocket = {
  // State
  socket,
  connectionState,
  subscriptions,
  isConnected,
  isConnecting,
  isReconnecting,

  // Connection
  connect,
  disconnect,
  reconnect,

  // Subscriptions
  subscribe,
  unsubscribe,
  unsubscribeAll,

  // Events
  on,
  off,
  onAny,
  offAny,

  // Sending
  send,
  sendAction,
  sendTyping,
  sendStopTyping,
  sendPresence,

  // Lifecycle
  createAutoConnect,
};

export default websocket;
