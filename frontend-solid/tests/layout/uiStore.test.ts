// ============================================
// UI Store Tests
// ============================================

import { describe, it, expect, beforeEach } from 'vitest';
import {
  uiStore,
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
  resetLayout,
} from '../../src/stores/ui';

describe('uiStore', () => {
  beforeEach(() => {
    // Reset layout to default state before each test
    resetLayout();
  });

  describe('sidebar state', () => {
    it('should toggle sidebar collapsed state', () => {
      const initialState = uiStore.preferences.sidebarCollapsed;
      toggleSidebar();
      expect(uiStore.preferences.sidebarCollapsed).toBe(!initialState);
    });

    it('should set sidebar collapsed state directly', () => {
      setSidebarCollapsed(true);
      expect(uiStore.preferences.sidebarCollapsed).toBe(true);
      setSidebarCollapsed(false);
      expect(uiStore.preferences.sidebarCollapsed).toBe(false);
    });
  });

  describe('right panel state', () => {
    it('should toggle right panel open state', () => {
      const initialState = uiStore.preferences.rightPanelOpen;
      toggleRightPanel();
      expect(uiStore.preferences.rightPanelOpen).toBe(!initialState);
    });

    it('should set right panel open state directly', () => {
      setRightPanelOpen(true);
      expect(uiStore.preferences.rightPanelOpen).toBe(true);
      setRightPanelOpen(false);
      expect(uiStore.preferences.rightPanelOpen).toBe(false);
    });

    it('should set active panel and open right panel', () => {
      setRightPanelOpen(false);
      setActivePanel('members');
      expect(uiStore.preferences.rightPanelOpen).toBe(true);
      expect(uiStore.activePanelTab()).toBe('members');
    });
  });

  describe('mobile sidebar state', () => {
    it('should toggle mobile sidebar open state', () => {
      const initialState = uiStore.preferences.mobileSidebarOpen;
      toggleMobileSidebar();
      expect(uiStore.preferences.mobileSidebarOpen).toBe(!initialState);
    });

    it('should set mobile sidebar open state directly', () => {
      setMobileSidebarOpen(true);
      expect(uiStore.preferences.mobileSidebarOpen).toBe(true);
      setMobileSidebarOpen(false);
      expect(uiStore.preferences.mobileSidebarOpen).toBe(false);
    });
  });

  describe('channel sections', () => {
    it('should toggle channel section expansion', () => {
      const initialState = uiStore.preferences.channelSectionsExpanded.channels;
      toggleChannelSection('channels');
      expect(uiStore.preferences.channelSectionsExpanded.channels).toBe(!initialState);
    });

    it('should set channel section expansion directly', () => {
      setChannelSectionExpanded('favorites', false);
      expect(uiStore.preferences.channelSectionsExpanded.favorites).toBe(false);
      setChannelSectionExpanded('favorites', true);
      expect(uiStore.preferences.channelSectionsExpanded.favorites).toBe(true);
    });
  });

  describe('width settings', () => {
    it('should set sidebar width within bounds', () => {
      setSidebarWidth(300);
      expect(uiStore.preferences.sidebarWidth).toBe(300);
    });

    it('should clamp sidebar width to minimum', () => {
      setSidebarWidth(100);
      expect(uiStore.preferences.sidebarWidth).toBe(220); // MIN_SIDEBAR_WIDTH
    });

    it('should clamp sidebar width to maximum', () => {
      setSidebarWidth(500);
      expect(uiStore.preferences.sidebarWidth).toBe(400); // MAX_SIDEBAR_WIDTH
    });

    it('should set right panel width within bounds', () => {
      setRightPanelWidth(300);
      expect(uiStore.preferences.rightPanelWidth).toBe(300);
    });

    it('should clamp right panel width to minimum', () => {
      setRightPanelWidth(100);
      expect(uiStore.preferences.rightPanelWidth).toBe(200);
    });

    it('should clamp right panel width to maximum', () => {
      setRightPanelWidth(500);
      expect(uiStore.preferences.rightPanelWidth).toBe(400);
    });
  });

  describe('computed values', () => {
    it('should return effective sidebar width when collapsed', () => {
      setSidebarCollapsed(true);
      expect(uiStore.effectiveSidebarWidth()).toBe(64);
    });

    it('should return effective sidebar width when expanded', () => {
      setSidebarCollapsed(false);
      setSidebarWidth(300);
      expect(uiStore.effectiveSidebarWidth()).toBe(300);
    });

    it('should return effective right panel width when closed', () => {
      setRightPanelOpen(false);
      expect(uiStore.effectiveRightPanelWidth()).toBe(0);
    });

    it('should return effective right panel width when open', () => {
      setRightPanelOpen(true);
      setRightPanelWidth(300);
      expect(uiStore.effectiveRightPanelWidth()).toBe(300);
    });
  });
});
