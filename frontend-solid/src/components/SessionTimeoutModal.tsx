// ============================================
// Session Timeout Warning Modal
// ============================================

import { createEffect } from 'solid-js';
import { authStore, extendSession, dismissSessionWarning } from '../stores/auth';
import Modal from './ui/Modal';
import Button from './ui/Button';

export default function SessionTimeoutModal() {
  const warning = () => authStore.sessionWarning();
  const remainingMinutes = () => Math.ceil(warning().remainingSeconds / 60);

  createEffect(() => {
    if (warning().show) {
      // Auto-logout when timer reaches 0
      if (warning().remainingSeconds <= 0) {
        authStore.logout('expired');
      }
    }
  });

  const handleExtend = async () => {
    const success = await extendSession();
    if (success) {
      dismissSessionWarning();
    }
  };

  const handleLogout = () => {
    dismissSessionWarning();
    authStore.logout('manual');
  };

  return (
    <Modal
      isOpen={warning().show}
      onClose={() => {}} // Prevent closing by clicking outside
      title="Session Expiring Soon"
      size="sm"
      closeOnOverlayClick={false}
    >
      <div class="space-y-4">
        <div class="flex items-start gap-3">
          <div class="flex-shrink-0">
            <svg
              class="h-6 w-6 text-warning"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width={2}
                d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          </div>
          <div>
            <p class="text-text-1">
              Your session will expire in{' '}
              <strong class="text-warning">{remainingMinutes()} minutes</strong>.
            </p>
            <p class="text-sm text-text-2 mt-1">
              Would you like to extend your session or sign out now?
            </p>
          </div>
        </div>

        <div class="flex gap-3 justify-end">
          <Button variant="ghost" onClick={handleLogout}>
            Sign Out
          </Button>
          <Button variant="primary" onClick={handleExtend}>
            Extend Session
          </Button>
        </div>
      </div>
    </Modal>
  );
}
