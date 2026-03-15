// ============================================
// UI State Store - Layout & UI Preferences
// ============================================

import { createStore } from 'solid-js/store';
import { createSignal, createEffect, createMemo } from 'solid-js';

// ============================================
// Types
// ============================================

export interface UIPreferences {
  sidebarCollapsed: boolean;
  rightPanelOpen: boolean;
  mobileSidebarOpen: boolean;
  channelSectionsExpanded: {
    favorites: boolean;
    channels: boolean;
    directMessages: boolean;
  };
  sidebarWidth: number;
  rightPanelWidth: number;
}

export type PanelTab = 'members' | 'pinned' | 'files' | 'info' | 'search' | 'saved';

// ============================================
// Constants
// ============================================

const STORAGE_KEY = 'rustchat_ui_prefs';
const MIN_SIDEBAR_WIDTH = 220;
const MAX_SIDEBAR_WIDTH = 400;
const DEFAULT_SIDEBAR_WIDTH = 260;
const DEFAULT_RIGHT_PANEL_WIDTH = 280;

// ============================================
// Store State
// ============================================

const getStoredPreferences = (): Partial<UIPreferences> => {
  if (typeof window === 'undefined') return {};
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored ? JSON.parse(stored) : {};
  } catch {
    return {};
  }
};

const stored = getStoredPreferences();

const [preferences, setPreferences] = createStore<UIPreferences>({
  sidebarCollapsed: stored.sidebarCollapsed ?? false,
  rightPanelOpen: stored.rightPanelOpen ?? false,
  mobileSidebarOpen: false,
  channelSectionsExpanded: stored.channelSectionsExpanded ?? {
    favorites: true,
    channels: true,
    directMessages: true,
  },
  sidebarWidth: stored.sidebarWidth ?? DEFAULT_SIDEBAR_WIDTH,
  rightPanelWidth: stored.rightPanelWidth ?? DEFAULT_RIGHT_PANEL_WIDTH,
});

const [activePanelTab, setActivePanelTab] = createSignal<PanelTab>('members');
const [isResizing, setIsResizing] = createSignal(false);

// ============================================
// Persistence Effects
// ============================================

createEffect(() => {
  if (typeof window !== 'undefined') {
    const prefsToStore = {
      sidebarCollapsed: preferences.sidebarCollapsed,
      rightPanelOpen: preferences.rightPanelOpen,
      channelSectionsExpanded: preferences.channelSectionsExpanded,
      sidebarWidth: preferences.sidebarWidth,
      rightPanelWidth: preferences.rightPanelWidth,
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(prefsToStore));
  }
});

// ============================================
// Computed
// ============================================

export const effectiveSidebarWidth = createMemo(() => {
  if (preferences.sidebarCollapsed) return 64;
  return preferences.sidebarWidth;
});

export const effectiveRightPanelWidth = createMemo(() => {
  if (!preferences.rightPanelOpen) return 0;
  return preferences.rightPanelWidth;
});

// ============================================
// Actions
// ============================================

export function toggleSidebar(): void {
  setPreferences('sidebarCollapsed', (prev) => !prev);
}

export function setSidebarCollapsed(collapsed: boolean): void {
  setPreferences('sidebarCollapsed', collapsed);
}

export function toggleRightPanel(tab?: PanelTab): void {
  setPreferences('rightPanelOpen', (prev) => {
    const newState = !prev;
    if (newState && tab) {
      setActivePanelTab(tab);
    }
    return newState;
  });
}

export function setRightPanelOpen(open: boolean): void {
  setPreferences('rightPanelOpen', open);
}

export function setActivePanel(tab: PanelTab): void {
  setActivePanelTab(tab);
  if (!preferences.rightPanelOpen) {
    setPreferences('rightPanelOpen', true);
  }
}

export function toggleMobileSidebar(): void {
  setPreferences('mobileSidebarOpen', (prev) => !prev);
}

export function setMobileSidebarOpen(open: boolean): void {
  setPreferences('mobileSidebarOpen', open);
}

export function toggleChannelSection(section: keyof UIPreferences['channelSectionsExpanded']): void {
  setPreferences(
    'channelSectionsExpanded',
    section,
    (prev) => !prev
  );
}

export function setChannelSectionExpanded(
  section: keyof UIPreferences['channelSectionsExpanded'],
  expanded: boolean
): void {
  setPreferences('channelSectionsExpanded', section, expanded);
}

export function setSidebarWidth(width: number): void {
  const clamped = Math.max(MIN_SIDEBAR_WIDTH, Math.min(MAX_SIDEBAR_WIDTH, width));
  setPreferences('sidebarWidth', clamped);
}

export function setRightPanelWidth(width: number): void {
  const clamped = Math.max(200, Math.min(400, width));
  setPreferences('rightPanelWidth', clamped);
}

export function startResizing(): void {
  setIsResizing(true);
}

export function stopResizing(): void {
  setIsResizing(false);
}

export function resetLayout(): void {
  setPreferences({
    sidebarCollapsed: false,
    rightPanelOpen: false,
    mobileSidebarOpen: false,
    channelSectionsExpanded: {
      favorites: true,
      channels: true,
      directMessages: true,
    },
    sidebarWidth: DEFAULT_SIDEBAR_WIDTH,
    rightPanelWidth: DEFAULT_RIGHT_PANEL_WIDTH,
  });
}

// ============================================
// Responsive Helpers
// ============================================

export function isMobileViewport(): boolean {
  if (typeof window === 'undefined') return false;
  return window.innerWidth < 768;
}

export function isTabletViewport(): boolean {
  if (typeof window === 'undefined') return false;
  return window.innerWidth >= 768 && window.innerWidth < 1024;
}

export function isDesktopViewport(): boolean {
  if (typeof window === 'undefined') return false;
  return window.innerWidth >= 1024;
}

// ============================================
// Exports
// ============================================

export const uiStore = {
  // State
  preferences,
  activePanelTab,
  isResizing,

  // Computed
  effectiveSidebarWidth,
  effectiveRightPanelWidth,

  // Actions
  toggleSidebar,
  setSidebarCollapsed,
  toggleRightPanel,
  setRightPanelOpen,
  setActivePanel,
  toggleMobileSidebar,
  setMobileSidebarOpen,
  toggleChannelSection,
  setChannelSectionExpanded,
  setSidebarWidth,
  setRightPanelWidth,
  startResizing,
  stopResizing,
  resetLayout,

  // Responsive helpers
  isMobileViewport,
  isTabletViewport,
  isDesktopViewport,
};

export default uiStore;
