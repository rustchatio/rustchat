// ============================================
// Notification Preferences Component
// ============================================

import { createSignal, createEffect, Show } from 'solid-js';
import { useDesktopNotifications } from '../hooks/useDesktopNotifications';

// ============================================
// Types
// ============================================

export interface NotificationPreferencesData {
  desktopEnabled: boolean;
  soundEnabled: boolean;
  messageNotifications: boolean;
  mentionNotifications: boolean;
  callNotifications: boolean;
  muteDnd: boolean;
}

// ============================================
// Storage Keys
// ============================================

const STORAGE_KEYS = {
  desktopEnabled: 'notifications_enabled',
  soundEnabled: 'notifications_sound_enabled',
  messageNotifications: 'notify_messages',
  mentionNotifications: 'notify_mentions',
  callNotifications: 'notify_calls',
  muteDnd: 'notifications_mute_dnd',
};

// ============================================
// Component
// ============================================

interface NotificationPreferencesProps {
  onChange?: (prefs: NotificationPreferencesData) => void;
}

export function NotificationPreferences(props: NotificationPreferencesProps) {
  const { permissionState, requestPermission, enableNotifications, disableNotifications } =
    useDesktopNotifications();

  const [prefs, setPrefs] = createSignal<NotificationPreferencesData>({
    desktopEnabled: false,
    soundEnabled: true,
    messageNotifications: true,
    mentionNotifications: true,
    callNotifications: true,
    muteDnd: true,
  });

  // Load preferences from localStorage
  createEffect(() => {
    setPrefs({
      desktopEnabled: localStorage.getItem(STORAGE_KEYS.desktopEnabled) === 'true',
      soundEnabled: localStorage.getItem(STORAGE_KEYS.soundEnabled) !== 'false',
      messageNotifications: localStorage.getItem(STORAGE_KEYS.messageNotifications) !== 'false',
      mentionNotifications: localStorage.getItem(STORAGE_KEYS.mentionNotifications) !== 'false',
      callNotifications: localStorage.getItem(STORAGE_KEYS.callNotifications) !== 'false',
      muteDnd: localStorage.getItem(STORAGE_KEYS.muteDnd) !== 'false',
    });
  });

  const updatePref = <K extends keyof NotificationPreferencesData>(
    key: K,
    value: NotificationPreferencesData[K]
  ) => {
    const newPrefs = { ...prefs(), [key]: value };
    setPrefs(newPrefs);
    localStorage.setItem(STORAGE_KEYS[key], String(value));
    props.onChange?.(newPrefs);
  };

  const handleDesktopToggle = async (enabled: boolean) => {
    if (enabled) {
      const permission = await requestPermission();
      if (permission === 'granted') {
        enableNotifications();
        updatePref('desktopEnabled', true);
      }
    } else {
      disableNotifications();
      updatePref('desktopEnabled', false);
    }
  };

  const state = permissionState();

  return (
    <div class="space-y-6">
      {/* Header */}
      <div>
        <h3 class="text-lg font-semibold">Notifications</h3>
        <p class="text-sm text-base-content/60">Configure how you receive notifications</p>
      </div>

      {/* Desktop Notifications */}
      <section class="space-y-3">
        <h4 class="font-medium text-sm uppercase tracking-wide text-base-content/50">Desktop</h4>

        <Show
          when={state.supported}
          fallback={
            <div class="p-3 rounded-lg bg-warning/10 text-warning text-sm">
              Desktop notifications are not supported in your browser.
            </div>
          }
        >
          <div class="flex items-center justify-between py-2">
            <div>
              <div class="font-medium">Enable desktop notifications</div>
              <div class="text-sm text-base-content/60">
                Show notifications even when RustChat is not focused
              </div>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={prefs().desktopEnabled && state.permission === 'granted'}
                onChange={(e) => handleDesktopToggle(e.target.checked)}
                class="sr-only peer"
                disabled={state.permission === 'denied'}
              />
              <div class="w-11 h-6 bg-base-300 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-primary/50 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary" />
            </label>
          </div>

          <Show when={state.permission === 'denied'}>
            <div class="p-3 rounded-lg bg-error/10 text-error text-sm">
              Notification permission was denied. Please enable it in your browser settings to receive
              desktop notifications.
            </div>
          </Show>

          <div class="flex items-center justify-between py-2">
            <div>
              <div class="font-medium">Sound</div>
              <div class="text-sm text-base-content/60">Play a sound for new notifications</div>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                checked={prefs().soundEnabled}
                onChange={(e) => updatePref('soundEnabled', e.target.checked)}
                class="sr-only peer"
              />
              <div class="w-11 h-6 bg-base-300 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-primary/50 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary" />
            </label>
          </div>
        </Show>
      </section>

      <div class="divider" />

      {/* Notification Types */}
      <section class="space-y-3">
        <h4 class="font-medium text-sm uppercase tracking-wide text-base-content/50">Notify me about</h4>

        <div class="flex items-center justify-between py-2">
          <div>
            <div class="font-medium">Messages</div>
            <div class="text-sm text-base-content/60">New messages in channels</div>
          </div>
          <label class="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={prefs().messageNotifications}
              onChange={(e) => updatePref('messageNotifications', e.target.checked)}
              class="sr-only peer"
            />
            <div class="w-11 h-6 bg-base-300 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-primary/50 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary" />
          </label>
        </div>

        <div class="flex items-center justify-between py-2">
          <div>
            <div class="font-medium">Mentions</div>
            <div class="text-sm text-base-content/60">When someone mentions @you</div>
          </div>
          <label class="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={prefs().mentionNotifications}
              onChange={(e) => updatePref('mentionNotifications', e.target.checked)}
              class="sr-only peer"
            />
            <div class="w-11 h-6 bg-base-300 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-primary/50 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary" />
          </label>
        </div>

        <div class="flex items-center justify-between py-2">
          <div>
            <div class="font-medium">Calls</div>
            <div class="text-sm text-base-content/60">Incoming voice and video calls</div>
          </div>
          <label class="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={prefs().callNotifications}
              onChange={(e) => updatePref('callNotifications', e.target.checked)}
              class="sr-only peer"
            />
            <div class="w-11 h-6 bg-base-300 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-primary/50 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary" />
          </label>
        </div>
      </section>

      <div class="divider" />

      {/* DND Settings */}
      <section class="space-y-3">
        <h4 class="font-medium text-sm uppercase tracking-wide text-base-content/50">Do Not Disturb</h4>

        <div class="flex items-center justify-between py-2">
          <div>
            <div class="font-medium">Mute during DND</div>
            <div class="text-sm text-base-content/60">
              Don't show notifications when status is Do Not Disturb
            </div>
          </div>
          <label class="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={prefs().muteDnd}
              onChange={(e) => updatePref('muteDnd', e.target.checked)}
              class="sr-only peer"
            />
            <div class="w-11 h-6 bg-base-300 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-primary/50 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary" />
          </label>
        </div>
      </section>
    </div>
  );
}

export default NotificationPreferences;
