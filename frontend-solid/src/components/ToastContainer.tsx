// ============================================
// Toast Container Component
// ============================================

import { For, Show } from 'solid-js';
import { useToast, type Toast, type ToastType } from '../hooks/useToast';

// ============================================
// Toast Item Component
// ============================================

interface ToastItemProps {
  toast: Toast;
  onDismiss: (id: string) => void;
}

const typeIcons: Record<ToastType, string> = {
  info: 'ℹ️',
  success: '✅',
  warning: '⚠️',
  error: '❌',
};

const typeClasses: Record<ToastType, string> = {
  info: 'border-l-info bg-info/10',
  success: 'border-l-success bg-success/10',
  warning: 'border-l-warning bg-warning/10',
  error: 'border-l-error bg-error/10',
};

function ToastItem(props: ToastItemProps) {
  return (
    <div
      role="alert"
      aria-live="polite"
      class={`relative flex gap-3 p-4 rounded-lg shadow-lg border-l-4 ${typeClasses[props.toast.type]} 
        bg-base-100 animate-slide-in-right min-w-[320px] max-w-md`}
    >
      {/* Icon */}
      <span class="text-xl flex-shrink-0" aria-hidden="true">
        {typeIcons[props.toast.type]}
      </span>

      {/* Content */}
      <div class="flex-1 min-w-0">
        <h4 class="font-medium text-sm">{props.toast.title}</h4>
        <Show when={props.toast.message}>
          <p class="text-sm text-base-content/70 mt-0.5">{props.toast.message}</p>
        </Show>

        {/* Action button */}
        <Show when={props.toast.action}>
          <button
            type="button"
            onClick={() => {
              props.toast.action?.onClick();
              props.onDismiss(props.toast.id);
            }}
            class="mt-2 text-sm font-medium text-primary hover:text-primary/80 transition-colors"
          >
            {props.toast.action?.label}
          </button>
        </Show>
      </div>

      {/* Dismiss button */}
      <Show when={props.toast.dismissible}>
        <button
          type="button"
          onClick={() => props.onDismiss(props.toast.id)}
          class="flex-shrink-0 p-1 rounded hover:bg-base-200/50 transition-colors text-base-content/50 hover:text-base-content"
          aria-label="Dismiss notification"
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </Show>
    </div>
  );
}

// ============================================
// Toast Container Component
// ============================================

interface ToastContainerProps {
  position?: 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left' | 'top-center' | 'bottom-center';
  class?: string;
}

const positionClasses: Record<string, string> = {
  'top-right': 'top-4 right-4',
  'top-left': 'top-4 left-4',
  'bottom-right': 'bottom-4 right-4',
  'bottom-left': 'bottom-4 left-4',
  'top-center': 'top-4 left-1/2 -translate-x-1/2',
  'bottom-center': 'bottom-4 left-1/2 -translate-x-1/2',
};

export function ToastContainer(props: ToastContainerProps) {
  const { toasts, remove } = useToast();
  const position = props.position || 'top-right';

  return (
    <div
      class={`fixed z-50 flex flex-col gap-2 pointer-events-none ${positionClasses[position]} ${props.class || ''}`}
      aria-live="polite"
      aria-atomic="false"
    >
      <For each={toasts()}>
        {(toast) => (
          <div class="pointer-events-auto">
            <ToastItem toast={toast} onDismiss={remove} />
          </div>
        )}
      </For>
    </div>
  );
}

export default ToastContainer;
