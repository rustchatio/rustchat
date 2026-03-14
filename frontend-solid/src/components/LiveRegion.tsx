// ============================================
// LiveRegion Component
// ARIA Live Regions for screen reader announcements
// WCAG 2.1: 4.1.3 Status Messages
// ============================================

import { createSignal, batch } from 'solid-js';

// ============================================
// Global Announcer State
// ============================================

type AnnouncerPriority = 'polite' | 'assertive';

interface Announcement {
  message: string;
  priority: AnnouncerPriority;
  id: string;
}

const [, setAnnouncements] = createSignal<Announcement[]>([]);
const [politeMessage, setPoliteMessage] = createSignal('');
const [assertiveMessage, setAssertiveMessage] = createSignal('');

// ============================================
// Announcer Functions
// ============================================

export function announce(
  message: string,
  priority: AnnouncerPriority = 'polite',
  clearAfter = 1000
) {
  const id = Math.random().toString(36).slice(2);

  batch(() => {
    setAnnouncements((prev) => [...prev, { message, priority, id }]);

    if (priority === 'polite') {
      setPoliteMessage(message);
    } else {
      setAssertiveMessage(message);
    }
  });

  setTimeout(() => {
    batch(() => {
      setAnnouncements((prev) => prev.filter((a) => a.id !== id));

      if (priority === 'polite') {
        setPoliteMessage('');
      } else {
        setAssertiveMessage('');
      }
    });
  }, clearAfter);
}

export function announceSuccess(message: string) {
  announce(message, 'polite', 1500);
}

export function announceError(message: string) {
  announce(message, 'assertive', 3000);
}

// ============================================
// Live Region Component
// ============================================

export function LiveRegion() {
  return (
    <div style={{ position: 'absolute', width: '1px', height: '1px', overflow: 'hidden', 'clip-path': 'inset(50%)' }}>
      <div role="status" aria-live="polite" aria-atomic="true">
        {politeMessage()}
      </div>
      <div role="alert" aria-live="assertive" aria-atomic="true">
        {assertiveMessage()}
      </div>
    </div>
  );
}

export default LiveRegion;
