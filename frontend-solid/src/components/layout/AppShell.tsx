// ============================================
// AppShell - Main Application Layout
// ============================================

import { JSX, Show, createEffect, onCleanup } from 'solid-js';
import { useIsMobile, useIsDesktop } from '@/hooks/useMediaQuery';
import { uiStore } from '@/stores/ui';
import Header from './Header';
import Sidebar from './Sidebar';
import MainContent from './MainContent';
import RightSidebar from './RightSidebar';
import MobileNav from './MobileNav';
import ActiveCallOverlay from '@/components/calls/ActiveCallOverlay';
import IncomingCallOverlay from '@/components/calls/IncomingCallOverlay';

// ============================================
// Props Interface
// ============================================

export interface AppShellProps {
  children: JSX.Element;
}

// ============================================
// AppShell Component
// ============================================

export function AppShell(props: AppShellProps) {
  const isMobile = useIsMobile();
  const isDesktop = useIsDesktop();

  // Close mobile sidebar on desktop
  createEffect(() => {
    if (!isMobile() && uiStore.preferences.mobileSidebarOpen) {
      uiStore.setMobileSidebarOpen(false);
    }
  });

  // Handle keyboard shortcuts
  createEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl/Cmd + B to toggle sidebar
      if ((e.ctrlKey || e.metaKey) && e.key === 'b') {
        e.preventDefault();
        if (isDesktop()) {
          uiStore.toggleSidebar();
        } else {
          uiStore.toggleMobileSidebar();
        }
      }

      // Escape to close mobile sidebar
      if (e.key === 'Escape' && uiStore.preferences.mobileSidebarOpen) {
        uiStore.setMobileSidebarOpen(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);

    onCleanup(() => {
      window.removeEventListener('keydown', handleKeyDown);
    });
  });

  return (
    <div class="h-screen flex flex-col bg-bg-app overflow-hidden">
      {/* Global Header */}
      <Header />

      {/* Main Layout Area */}
      <div class="flex-1 flex overflow-hidden">
        {/* Left Sidebar - Desktop always visible, Mobile overlay */}
        <Show when={!isMobile()}>
          <Sidebar />
        </Show>

        {/* Mobile Sidebar Overlay */}
        <Show when={isMobile() && uiStore.preferences.mobileSidebarOpen}>
          <>
            {/* Backdrop */}
            <div
              class="fixed inset-0 bg-black/50 z-40 animate-fade-in"
              onClick={() => uiStore.setMobileSidebarOpen(false)}
              aria-hidden="true"
            />
            {/* Mobile Sidebar */}
            <div class="fixed left-0 top-0 bottom-0 w-[280px] z-50 animate-slide-in-left">
              <Sidebar isMobile />
            </div>
          </>
        </Show>

        {/* Main Content Area */}
        <MainContent>{props.children}</MainContent>

        {/* Right Sidebar - Desktop only */}
        <Show when={isDesktop()}>
          <RightSidebar />
        </Show>
      </div>

      <ActiveCallOverlay />
      <IncomingCallOverlay />

      {/* Mobile Bottom Navigation */}
      <Show when={isMobile()}>
        <MobileNav />
      </Show>
    </div>
  );
}

// ============================================
// Animation Styles
// ============================================

const style = document.createElement('style');
style.textContent = `
  @keyframes slide-in-left {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(0);
    }
  }
  
  .animate-slide-in-left {
    animation: slide-in-left 0.2s ease-out forwards;
  }
  
  .animate-fade-in {
    animation: fade-in 0.2s ease-out forwards;
  }
  
  @keyframes fade-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
`;
document.head.appendChild(style);

export default AppShell;
