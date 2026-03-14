// ============================================
// Connection Status Indicator
// Shows WebSocket connection state
// ============================================

import { Show, createMemo } from 'solid-js';
import { connectionState, isConnected, isReconnecting } from '../realtime/websocket';
import type { ConnectionState } from '../realtime/events';

// ============================================
// Status Configuration
// ============================================

type ConnectionStatus = ConnectionState['status'];

interface StatusConfig {
  label: string;
  icon: string;
  color: string;
  bgColor: string;
  animate: boolean;
}

const STATUS_CONFIG: Record<ConnectionStatus, StatusConfig> = {
  connected: {
    label: 'Connected',
    icon: '●',
    color: 'text-green-500',
    bgColor: 'bg-green-500/10',
    animate: false,
  },
  connecting: {
    label: 'Connecting...',
    icon: '◐',
    color: 'text-yellow-500',
    bgColor: 'bg-yellow-500/10',
    animate: true,
  },
  reconnecting: {
    label: 'Reconnecting...',
    icon: '↻',
    color: 'text-orange-500',
    bgColor: 'bg-orange-500/10',
    animate: true,
  },
  disconnected: {
    label: 'Disconnected',
    icon: '○',
    color: 'text-gray-400',
    bgColor: 'bg-gray-400/10',
    animate: false,
  },
  error: {
    label: 'Connection Error',
    icon: '✕',
    color: 'text-red-500',
    bgColor: 'bg-red-500/10',
    animate: false,
  },
};

// ============================================
// Compact Indicator (for header)
// ============================================

export function ConnectionIndicator() {
  const config = createMemo(() => STATUS_CONFIG[connectionState().status as ConnectionStatus]);
  const showIndicator = createMemo(() => !isConnected());

  return (
    <Show when={showIndicator()}>
      <div
        class={`flex items-center gap-1.5 px-2 py-1 rounded-full text-xs font-medium ${config().bgColor} ${config().color} ${
          config().animate ? 'animate-pulse' : ''
        }`}
        title={config().label}
      >
        <span class={config().animate ? 'animate-spin' : ''}>{config().icon}</span>
        <Show when={isReconnecting()}>
          <span>Reconnecting ({connectionState().reconnectAttempts})</span>
        </Show>
      </div>
    </Show>
  );
}

// ============================================
// Full Status Badge (for settings/debug)
// ============================================

export function ConnectionBadge() {
  const config = createMemo(() => STATUS_CONFIG[connectionState().status as ConnectionStatus]);
  const state = connectionState;

  return (
    <div class={`inline-flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm font-medium ${config().bgColor} ${config().color}`}>
      <span class={`text-lg ${config().animate ? 'animate-spin' : ''}`}>{config().icon}</span>
      <span>{config().label}</span>
      <Show when={state().reconnectAttempts > 0}>
        <span class="opacity-60">(Attempt {state().reconnectAttempts})</span>
      </Show>
    </div>
  );
}

// ============================================
// Connection Status Panel (detailed view)
// ============================================

export function ConnectionStatusPanel() {
  const config = createMemo(() => STATUS_CONFIG[connectionState().status as ConnectionStatus]);
  const state = connectionState;

  const formatTime = (date: Date | undefined) => {
    if (!date) return 'Never';
    return new Date(date).toLocaleTimeString();
  };

  return (
    <div class="bg-surface-0 border border-surface-2 rounded-lg p-4 space-y-3">
      <div class="flex items-center justify-between">
        <h3 class="font-semibold text-fg">Connection Status</h3>
        <ConnectionBadge />
      </div>

      <div class="grid grid-cols-2 gap-4 text-sm">
        <div>
          <span class="text-fg-2">Status:</span>
          <span class={`ml-2 font-medium ${config().color}`}>
            {state().status.charAt(0).toUpperCase() + state().status.slice(1)}
          </span>
        </div>

        <div>
          <span class="text-fg-2">Reconnect Attempts:</span>
          <span class="ml-2 font-medium">{state().reconnectAttempts}</span>
        </div>

        <div>
          <span class="text-fg-2">Connected At:</span>
          <span class="ml-2 font-medium">{formatTime(state().connectedAt)}</span>
        </div>

        <div>
          <span class="text-fg-2">Last Error:</span>
          <span class="ml-2 font-medium text-red-500">
            {state().lastError || 'None'}
          </span>
        </div>
      </div>

      <Show when={state().status === 'error' && state().lastError}>
        <div class="bg-red-500/10 border border-red-500/20 rounded p-3 text-sm text-red-600">
          <p class="font-medium">Connection Error</p>
          <p class="mt-1">{state().lastError}</p>
          <p class="mt-2 text-xs opacity-80">
            Check your network connection and try refreshing the page.
          </p>
        </div>
      </Show>
    </div>
  );
}

// ============================================
// Toast Notification for Connection Changes
// ============================================

import { createEffect } from 'solid-js';
import { toast } from '../hooks/useToast';
import { playConnectedSound, playDisconnectedSound } from '../utils/sounds';

let lastStatus: ConnectionState['status'] = 'disconnected';

export function ConnectionToastNotifier() {
  createEffect(() => {
    const status = connectionState().status;

    if (status !== lastStatus) {
      switch (status) {
        case 'connected':
          if (lastStatus === 'reconnecting' || lastStatus === 'connecting') {
            toast.success('Connection restored');
            playConnectedSound();
          }
          break;

        case 'reconnecting':
          toast.info('Reconnecting to server...');
          break;

        case 'error':
          toast.error('Connection lost. Trying to reconnect...');
          playDisconnectedSound();
          break;

        case 'disconnected':
          if (lastStatus !== 'error') {
            toast.warning('Disconnected from server');
            playDisconnectedSound();
          }
          break;
      }

      lastStatus = status;
    }
  });

  return null;
}
