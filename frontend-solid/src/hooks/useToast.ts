// ============================================
// Toast Notification System
// ============================================

import { createSignal, batch } from 'solid-js';

// ============================================
// Types
// ============================================

export type ToastType = 'info' | 'success' | 'warning' | 'error';

export interface Toast {
  id: string;
  type: ToastType;
  title: string;
  message?: string;
  duration?: number;
  dismissible?: boolean;
  action?: {
    label: string;
    onClick: () => void;
  };
}

export interface ToastOptions {
  type?: ToastType;
  title: string;
  message?: string;
  duration?: number;
  dismissible?: boolean;
  action?: {
    label: string;
    onClick: () => void;
  };
}

// ============================================
// Global State (created once)
// ============================================

const [toasts, setToasts] = createSignal<Toast[]>([]);

// Auto-incrementing ID
let toastIdCounter = 0;

// ============================================
// Actions
// ============================================

function generateId(): string {
  return `toast-${++toastIdCounter}-${Date.now()}`;
}

export function addToast(options: ToastOptions): string {
  const id = generateId();
  const duration = options.duration ?? getDefaultDuration(options.type);

  const toast: Toast = {
    id,
    type: options.type || 'info',
    title: options.title,
    message: options.message,
    duration,
    dismissible: options.dismissible ?? true,
    action: options.action,
  };

  batch(() => {
    setToasts((prev) => [...prev, toast]);
  });

  // Auto-dismiss
  if (duration > 0) {
    setTimeout(() => {
      removeToast(id);
    }, duration);
  }

  return id;
}

export function removeToast(id: string): void {
  batch(() => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  });
}

export function clearAllToasts(): void {
  batch(() => {
    setToasts([]);
  });
}

function getDefaultDuration(type?: ToastType): number {
  switch (type) {
    case 'error':
      return 8000; // Errors stay longer
    case 'success':
      return 3000;
    case 'warning':
      return 5000;
    default:
      return 4000;
  }
}

// ============================================
// Convenience Methods
// ============================================

export const toast = {
  info: (title: string, message?: string, options?: Omit<ToastOptions, 'type' | 'title' | 'message'>) =>
    addToast({ type: 'info', title, message, ...options }),

  success: (title: string, message?: string, options?: Omit<ToastOptions, 'type' | 'title' | 'message'>) =>
    addToast({ type: 'success', title, message, ...options }),

  warning: (title: string, message?: string, options?: Omit<ToastOptions, 'type' | 'title' | 'message'>) =>
    addToast({ type: 'warning', title, message, ...options }),

  error: (title: string, message?: string, options?: Omit<ToastOptions, 'type' | 'title' | 'message'>) =>
    addToast({ type: 'error', title, message, ...options }),

  promise: async <T>(
    promise: Promise<T>,
    messages: {
      loading: string;
      success: string;
      error: string;
    },
    options?: Omit<ToastOptions, 'type' | 'title' | 'message' | 'duration'>
  ): Promise<T> => {
    const id = addToast({
      type: 'info',
      title: messages.loading,
      duration: 0, // Never auto-dismiss
      dismissible: false,
      ...options,
    });

    try {
      const result = await promise;
      removeToast(id);
      addToast({
        type: 'success',
        title: messages.success,
        duration: 3000,
      });
      return result;
    } catch (err) {
      removeToast(id);
      addToast({
        type: 'error',
        title: messages.error,
        message: err instanceof Error ? err.message : undefined,
        duration: 5000,
      });
      throw err;
    }
  },

  remove: removeToast,
  clear: clearAllToasts,
};

// ============================================
// Hook
// ============================================

export function useToast() {
  return {
    toasts,
    add: addToast,
    remove: removeToast,
    clear: clearAllToasts,
    toast,
  };
}

export default useToast;
