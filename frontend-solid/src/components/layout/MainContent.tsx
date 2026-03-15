// ============================================
// MainContent - Flexible Content Area
// ============================================

import { JSX, createEffect } from 'solid-js';
import { channelStore } from '@/stores/channels';

// ============================================
// Props Interface
// ============================================

export interface MainContentProps {
  children: JSX.Element;
}

// ============================================
// MainContent Component
// ============================================

export function MainContent(props: MainContentProps) {
  // Track current channel for title updates
  createEffect(() => {
    const channel = channelStore.currentChannel();
    if (channel && typeof document !== 'undefined') {
      document.title = `${channel.display_name} - RustChat`;
    } else if (typeof document !== 'undefined') {
      document.title = 'RustChat';
    }
  });

  return (
    <main id="main-content" class="flex-1 min-w-0 bg-bg-app" tabIndex={-1}>
      {props.children}
    </main>
  );
}

export default MainContent;
