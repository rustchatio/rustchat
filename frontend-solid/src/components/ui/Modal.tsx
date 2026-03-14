import { Dialog } from '@kobalte/core';
import { Show, splitProps, JSX, createEffect, on } from 'solid-js';
import { cn } from '@/utils/cn';
import type { BaseProps, TestProps } from '@/types';

// ============================================
// Modal Props Interface
// ============================================

export interface ModalProps extends BaseProps, TestProps {
  /** Whether the modal is open */
  isOpen: boolean;
  /** Callback when modal is closed */
  onClose: () => void;
  /** Modal title */
  title: string;
  /** Modal description (for accessibility) */
  description?: string;
  /** Modal content */
  children: JSX.Element;
  /** Whether to close on overlay click */
  closeOnOverlayClick?: boolean;
  /** Whether to close on Escape key */
  closeOnEsc?: boolean;
  /** Whether to prevent body scroll when open */
  preventScroll?: boolean;
  /** Initial focus element ref */
  initialFocus?: () => HTMLElement | undefined;
  /** Modal size */
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
}

// ============================================
// Default Props
// ============================================

const defaultProps: Partial<ModalProps> = {
  closeOnOverlayClick: true,
  closeOnEsc: true,
  preventScroll: true,
  size: 'md',
};

// ============================================
// Size Styles
// ============================================

const sizeStyles = {
  sm: 'max-w-sm',
  md: 'max-w-lg',
  lg: 'max-w-2xl',
  xl: 'max-w-4xl',
  full: 'max-w-full mx-4',
};

// ============================================
// Modal Component
// ============================================

export function Modal(props: ModalProps): JSX.Element {
  const merged = mergeProps(defaultProps, props);
  const [local] = splitProps(merged, [
    'isOpen',
    'onClose',
    'title',
    'description',
    'children',
    'closeOnOverlayClick',
    'closeOnEsc',
    'size',
    'class',
    'data-testid',
  ]);

  // Handle body scroll lock
  createEffect(
    on(
      () => local.isOpen,
      isOpen => {
        if (typeof document === 'undefined') return;

        if (isOpen) {
          document.body.style.overflow = 'hidden';
        } else {
          document.body.style.overflow = '';
        }
      }
    )
  );

  return (
    <Dialog.Root
      open={local.isOpen}
      onOpenChange={open => {
        if (!open) local.onClose();
      }}
      modal
    >
      <Dialog.Portal>
        {/* Overlay */}
        <Dialog.Overlay
          class={cn('fixed inset-0 z-50', 'bg-black/50 backdrop-blur-sm', 'animate-fade-in')}
        />

        {/* Modal Container */}
        <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
          <Dialog.Content
            class={cn(
              'relative w-full',
              sizeStyles[local.size as keyof typeof sizeStyles],
              'bg-bg-surface-1 rounded-lg border border-border-1',
              'shadow-2xl animate-slide-in-up focus:outline-none',
              local.class
            )}
            onPointerDownOutside={e => {
              if (!local.closeOnOverlayClick) {
                e.preventDefault();
              }
            }}
            onEscapeKeyDown={e => {
              if (!local.closeOnEsc) {
                e.preventDefault();
              }
            }}
            data-testid={local['data-testid']}
          >
            {/* Header */}
            <div class="flex items-center justify-between px-6 py-4 border-b border-border-1">
              <div>
                <Dialog.Title class="text-lg font-semibold text-text-1">{local.title}</Dialog.Title>
                <Show when={local.description}>
                  <Dialog.Description class="mt-1 text-sm text-text-3">
                    {local.description}
                  </Dialog.Description>
                </Show>
              </div>

              {/* Close Button */}
              <Dialog.CloseButton
                class={cn(
                  'p-2 rounded-lg text-text-3 hover:text-text-1 hover:bg-bg-surface-2',
                  'transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-brand/50'
                )}
                aria-label="Close modal"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  aria-hidden="true"
                >
                  <path d="M18 6 6 18" />
                  <path d="m6 6 12 12" />
                </svg>
              </Dialog.CloseButton>
            </div>

            {/* Content */}
            <div class="px-6 py-4">{local.children}</div>
          </Dialog.Content>
        </div>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

// ============================================
// Helper for mergeProps with default values
// ============================================

function mergeProps<T>(defaults: Partial<T>, props: T): T {
  return { ...defaults, ...props };
}

export default Modal;
