import { Show, For } from 'solid-js';
import { useNavigate, useLocation } from '@solidjs/router';
import { logout } from '../stores/auth';
import { userStore, updatePreferences, resetPreferences } from '../stores/user';
import { useTheme, AVAILABLE_THEMES } from '../stores/theme';
import Button from '../components/ui/Button';

interface SettingsSection {
  id: string;
  label: string;
  icon: string;
}

const sections: SettingsSection[] = [
  { id: 'profile', label: 'Profile', icon: '👤' },
  { id: 'notifications', label: 'Notifications', icon: '🔔' },
  { id: 'display', label: 'Display', icon: '🎨' },
  { id: 'sidebar', label: 'Sidebar', icon: '📑' },
  { id: 'advanced', label: 'Advanced', icon: '⚙️' },
];

export default function Settings() {
  const navigate = useNavigate();
  const location = useLocation();

  // Get current section from URL
  const currentSection = () => {
    const match = location.pathname.match(/\/settings\/(\w+)/);
    return match?.[1] || 'profile';
  };

  const handleSectionClick = (sectionId: string) => {
    navigate(`/settings/${sectionId}`);
  };

  const handleLogout = async () => {
    await logout('manual');
    navigate('/login');
  };

  return (
    <div class="flex h-screen bg-bg-app">
      {/* Settings Sidebar */}
      <aside class="w-64 border-r border-border-1 bg-bg-panel flex flex-col">
        <div class="p-4 border-b border-border-1">
          <h1 class="font-semibold text-text-1">Settings</h1>
        </div>

        <nav class="flex-1 p-2 space-y-1">
          <For each={sections}>
            {(section) => (
              <button
                onClick={() => handleSectionClick(section.id)}
                class={`
                  w-full flex items-center gap-3 px-3 py-2 rounded-lg text-left transition-colors
                  ${currentSection() === section.id
                    ? 'bg-primary-50 text-primary-700'
                    : 'text-text-1 hover:bg-bg-hover'
                  }
                `}
              >
                <span>{section.icon}</span>
                <span class="font-medium">{section.label}</span>
              </button>
            )}
          </For>
        </nav>

        <div class="p-4 border-t border-border-1">
          <Button variant="danger" size="sm" class="w-full" onClick={handleLogout}>
            Sign Out
          </Button>
        </div>
      </aside>

      {/* Settings Content */}
      <main class="flex-1 overflow-y-auto p-8">
        <div class="max-w-2xl">
          <Show when={currentSection() === 'profile'}>
            <ProfileSettings />
          </Show>

          <Show when={currentSection() === 'notifications'}>
            <NotificationSettings />
          </Show>

          <Show when={currentSection() === 'display'}>
            <DisplaySettings />
          </Show>

          <Show when={currentSection() === 'sidebar'}>
            <SidebarSettings />
          </Show>

          <Show when={currentSection() === 'advanced'}>
            <AdvancedSettings />
          </Show>
        </div>
      </main>
    </div>
  );
}

import { authStore } from '../stores/auth';

function ProfileSettings() {
  const user = authStore.user;

  return (
    <div class="space-y-6">
      <div>
        <h2 class="text-2xl font-bold text-text-1">Profile</h2>
        <p class="text-text-3 mt-1">Manage your personal information</p>
      </div>

      <div class="space-y-4">
        <div class="flex items-center gap-4">
          <Show
            when={user()?.avatar_url}
            fallback={
              <div class="w-20 h-20 rounded-full bg-primary-100 flex items-center justify-center text-primary-700 text-2xl font-bold">
                {user()?.username?.slice(0, 2).toUpperCase() || '?'}
              </div>
            }
          >
            <img
              src={user()?.avatar_url}
              alt={user()?.username}
              class="w-20 h-20 rounded-full"
            />
          </Show>
          <div>
            <Button variant="secondary" size="sm">
              Change Avatar
            </Button>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-text-2 mb-1">Username</label>
            <input
              type="text"
              value={user()?.username || ''}
              disabled
              class="w-full px-3 py-2 bg-bg-app border border-border-1 rounded-lg text-text-1"
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-text-2 mb-1">Email</label>
            <input
              type="email"
              value={user()?.email || ''}
              disabled
              class="w-full px-3 py-2 bg-bg-app border border-border-1 rounded-lg text-text-1"
            />
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium text-text-2 mb-1">Display Name</label>
          <input
            type="text"
            value={user()?.display_name || ''}
            placeholder="Add a display name"
            disabled
            class="w-full px-3 py-2 bg-bg-app border border-border-1 rounded-lg text-text-1"
          />
        </div>
      </div>

      <div class="pt-4 border-t border-border-1">
        <Button variant="primary" disabled>
          Save Changes
        </Button>
      </div>
    </div>
  );
}

function NotificationSettings() {
  return (
    <div class="space-y-6">
      <div>
        <h2 class="text-2xl font-bold text-text-1">Notifications</h2>
        <p class="text-text-3 mt-1">Configure how you receive notifications</p>
      </div>

      <div class="space-y-4">
        <div class="flex items-center justify-between p-4 bg-bg-panel rounded-lg border border-border-1">
          <div>
            <h3 class="font-medium text-text-1">Desktop Notifications</h3>
            <p class="text-sm text-text-3">Show notifications on your desktop</p>
          </div>
          <label class="relative inline-flex items-center cursor-pointer">
            <input type="checkbox" class="sr-only peer" disabled />
            <div class="w-11 h-6 bg-bg-hover peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-500"></div>
          </label>
        </div>

        <div class="flex items-center justify-between p-4 bg-bg-panel rounded-lg border border-border-1">
          <div>
            <h3 class="font-medium text-text-1">Sound</h3>
            <p class="text-sm text-text-3">Play a sound for new messages</p>
          </div>
          <label class="relative inline-flex items-center cursor-pointer">
            <input type="checkbox" class="sr-only peer" disabled />
            <div class="w-11 h-6 bg-bg-hover peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-500"></div>
          </label>
        </div>

        <div class="flex items-center justify-between p-4 bg-bg-panel rounded-lg border border-border-1">
          <div>
            <h3 class="font-medium text-text-1">Email Notifications</h3>
            <p class="text-sm text-text-3">Receive email for mentions and DMs</p>
          </div>
          <label class="relative inline-flex items-center cursor-pointer">
            <input type="checkbox" class="sr-only peer" disabled />
            <div class="w-11 h-6 bg-bg-hover peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-500"></div>
          </label>
        </div>
      </div>
    </div>
  );
}

function DisplaySettings() {
  const themeContext = useTheme();
  const userPrefs = userStore.preferences;

  const handleThemeChange = (newTheme: string) => {
    themeContext.setTheme(newTheme as 'light' | 'dark' | 'modern' | 'metallic' | 'futuristic' | 'high-contrast' | 'simple' | 'dynamic');
  };

  return (
    <div class="space-y-6">
      <div>
        <h2 class="text-2xl font-bold text-text-1">Display</h2>
        <p class="text-text-3 mt-1">Customize your visual experience</p>
      </div>

      <div class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-text-2 mb-2">Theme</label>
          <div class="grid grid-cols-4 gap-3">
            <For each={AVAILABLE_THEMES}>
              {(themeOption) => (
                <button
                  onClick={() => handleThemeChange(themeOption)}
                  class={`
                    p-3 rounded-lg border-2 text-center transition-all
                    ${themeContext.theme() === themeOption
                      ? 'border-primary-500 bg-primary-50'
                      : 'border-border-1 hover:border-border-2'
                    }
                  `}
                >
                  <div class="text-2xl mb-1">
                    {themeOption === 'light' && '☀️'}
                    {themeOption === 'dark' && '🌙'}
                    {themeOption === 'modern' && '✨'}
                    {themeOption === 'metallic' && '🔩'}
                    {themeOption === 'futuristic' && '🚀'}
                    {themeOption === 'high-contrast' && '👁️'}
                    {themeOption === 'simple' && '📝'}
                    {themeOption === 'dynamic' && '🎭'}
                  </div>
                  <div class="text-sm font-medium capitalize text-text-1">
                    {themeOption.replace('-', ' ')}
                  </div>
                </button>
              )}
            </For>
          </div>
        </div>

        <div class="flex items-center justify-between p-4 bg-bg-panel rounded-lg border border-border-1">
          <div>
            <h3 class="font-medium text-text-1">Message Display</h3>
            <p class="text-sm text-text-3">How messages appear in channels</p>
          </div>
          <select
            value={userPrefs.message_display}
            onChange={(e) => updatePreferences({ message_display: e.currentTarget.value as 'clean' | 'compact' | 'standard' })}
            class="px-3 py-2 bg-bg-app border border-border-1 rounded-lg text-text-1"
          >
            <option value="clean">Clean</option>
            <option value="compact">Compact</option>
            <option value="standard">Standard</option>
          </select>
        </div>

        <div class="flex items-center justify-between p-4 bg-bg-panel rounded-lg border border-border-1">
          <div>
            <h3 class="font-medium text-text-1">Clock Display</h3>
            <p class="text-sm text-text-3">12-hour or 24-hour format</p>
          </div>
          <select
            value={userPrefs.clock_display}
            onChange={(e) => updatePreferences({ clock_display: e.currentTarget.value as '12hour' | '24hour' })}
            class="px-3 py-2 bg-bg-app border border-border-1 rounded-lg text-text-1"
          >
            <option value="12hour">12-hour</option>
            <option value="24hour">24-hour</option>
          </select>
        </div>
      </div>
    </div>
  );
}

function SidebarSettings() {
  const prefs = userStore.preferences;

  return (
    <div class="space-y-6">
      <div>
        <h2 class="text-2xl font-bold text-text-1">Sidebar</h2>
        <p class="text-text-3 mt-1">Customize your sidebar layout</p>
      </div>

      <div class="space-y-4">
        <div class="flex items-center justify-between p-4 bg-bg-panel rounded-lg border border-border-1">
          <div>
            <h3 class="font-medium text-text-1">Auto-collapse Categories</h3>
            <p class="text-sm text-text-3">Collapse sidebar categories automatically</p>
          </div>
          <label class="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={prefs.sidebar_collapsed}
              onChange={(e) => updatePreferences({ sidebar_collapsed: e.currentTarget.checked })}
              class="sr-only peer"
            />
            <div class="w-11 h-6 bg-bg-hover peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-500"></div>
          </label>
        </div>

        <div class="flex items-center justify-between p-4 bg-bg-panel rounded-lg border border-border-1">
          <div>
            <h3 class="font-medium text-text-1">Channel Switcher</h3>
            <p class="text-sm text-text-3">Sort channels by recent or alphabetically</p>
          </div>
          <select
            value={prefs.channel_switcher_mode}
            onChange={(e) => updatePreferences({ channel_switcher_mode: e.currentTarget.value as 'recent' | 'alpha' })}
            class="px-3 py-2 bg-bg-app border border-border-1 rounded-lg text-text-1"
          >
            <option value="recent">Recent</option>
            <option value="alpha">Alphabetical</option>
          </select>
        </div>

        <div class="flex items-center justify-between p-4 bg-bg-panel rounded-lg border border-border-1">
          <div>
            <h3 class="font-medium text-text-1">Unread Scrollbar</h3>
            <p class="text-sm text-text-3">Show scrollbar indicators for unread messages</p>
          </div>
          <label class="relative inline-flex items-center cursor-pointer">
            <input
              type="checkbox"
              checked={prefs.enable_unread_scrollbar}
              onChange={(e) => updatePreferences({ enable_unread_scrollbar: e.currentTarget.checked })}
              class="sr-only peer"
            />
            <div class="w-11 h-6 bg-bg-hover peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-500"></div>
          </label>
        </div>
      </div>
    </div>
  );
}

function AdvancedSettings() {
  return (
    <div class="space-y-6">
      <div>
        <h2 class="text-2xl font-bold text-text-1">Advanced</h2>
        <p class="text-text-3 mt-1">Advanced settings and options</p>
      </div>

      <div class="space-y-4">
        <div class="p-4 bg-bg-panel rounded-lg border border-border-1">
          <h3 class="font-medium text-text-1 mb-2">Data & Storage</h3>
          <p class="text-sm text-text-3 mb-3">Manage cached data and storage usage</p>
          <Button variant="secondary" size="sm">
            Clear Cache
          </Button>
        </div>

        <div class="p-4 bg-bg-panel rounded-lg border border-border-1">
          <h3 class="font-medium text-text-1 mb-2">Developer Mode</h3>
          <p class="text-sm text-text-3 mb-3">Enable developer features and debugging</p>
          <label class="relative inline-flex items-center cursor-pointer">
            <input type="checkbox" class="sr-only peer" disabled />
            <div class="w-11 h-6 bg-bg-hover peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-500"></div>
          </label>
        </div>

        <div class="p-4 bg-danger-50 rounded-lg border border-danger-200">
          <h3 class="font-medium text-danger-700 mb-2">Reset Settings</h3>
          <p class="text-sm text-danger-600 mb-3">Reset all settings to default values</p>
          <Button variant="danger" size="sm" onClick={resetPreferences}>
            Reset All Settings
          </Button>
        </div>
      </div>
    </div>
  );
}
