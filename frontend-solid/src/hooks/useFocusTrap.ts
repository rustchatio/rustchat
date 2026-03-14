// ============================================
// useFocusTrap Hook
// Traps focus within a modal/dialog for accessibility
// WCAG 2.1: 2.4.3 Focus Order
// ============================================

import { onCleanup, createEffect } from 'solid-js';
import type { Accessor } from 'solid-js';

interface FocusTrapOptions {
  /** Whether the trap is active */
  enabled?: Accessor<boolean>;
  /** Element to focus when trap is activated */
  initialFocus?: Accessor<HTMLElement | null>;
  /** Element to focus when trap is deactivated */
  returnFocus?: Accessor<HTMLElement | null>;
  /** Callback when user attempts to escape (Escape key) */
  onEscape?: () => void;
}

/**
 * List of selectors for focusable elements
 */
const FOCUSABLE_SELECTORS = [
  'button:not([disabled])',
  '[href]',
  'input:not([disabled])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  '[tabindex]:not([tabindex="-1"])',
  '[contenteditable]',
  'audio[controls]',
  'video[controls]',
  'details summary',
].join(', ');

/**
 * Custom hook to trap focus within a container element
 * Essential for modal dialogs, dropdowns, and other overlay components
 */
export function useFocusTrap(
  containerRef: Accessor<HTMLElement | null>,
  options: FocusTrapOptions = {}
) {
  let previousActiveElement: Element | null = null;
  let focusableElements: HTMLElement[] = [];

  /**
   * Get all focusable elements within the container
   */
  const getFocusableElements = (): HTMLElement[] => {
    const container = containerRef();
    if (!container) return [];

    return Array.from(
      container.querySelectorAll(FOCUSABLE_SELECTORS)
    ).filter((el): el is HTMLElement => {
      // Filter out hidden elements
      const style = window.getComputedStyle(el);
      return style.display !== 'none' && style.visibility !== 'hidden';
    });
  };

  /**
   * Handle Tab key navigation
   */
  const handleTab = (e: KeyboardEvent) => {
    if (e.key !== 'Tab') return;

    const container = containerRef();
    if (!container) return;

    focusableElements = getFocusableElements();
    if (focusableElements.length === 0) {
      e.preventDefault();
      return;
    }

    const firstElement = focusableElements[0];
    const lastElement = focusableElements[focusableElements.length - 1];

    // Shift + Tab
    if (e.shiftKey) {
      if (document.activeElement === firstElement) {
        e.preventDefault();
        lastElement.focus();
      }
    } else {
      // Tab
      if (document.activeElement === lastElement) {
        e.preventDefault();
        firstElement.focus();
      }
    }
  };

  /**
   * Handle Escape key
   */
  const handleEscape = (e: KeyboardEvent) => {
    if (e.key === 'Escape' && options.onEscape) {
      options.onEscape();
    }
  };

  /**
   * Activate focus trap
   */
  const activate = () => {
    const container = containerRef();
    if (!container) return;

    // Store previously focused element
    previousActiveElement = document.activeElement;

    // Set focus to initial element or first focusable
    const initialElement = options.initialFocus?.() || getFocusableElements()[0];
    if (initialElement) {
      initialElement.focus();
    }

    // Add event listeners
    container.addEventListener('keydown', handleTab);
    container.addEventListener('keydown', handleEscape);
  };

  /**
   * Deactivate focus trap
   */
  const deactivate = () => {
    const container = containerRef();
    if (!container) return;

    // Remove event listeners
    container.removeEventListener('keydown', handleTab);
    container.removeEventListener('keydown', handleEscape);

    // Return focus to previous element or specified element
    const returnEl = options.returnFocus?.() || previousActiveElement;
    if (returnEl && 'focus' in returnEl) {
      (returnEl as HTMLElement).focus();
    }
  };

  /**
   * Re-calculate focusable elements (useful when content changes)
   */
  const refresh = () => {
    focusableElements = getFocusableElements();
  };

  createEffect(() => {
    const isEnabled = options.enabled?.() ?? true;
    const container = containerRef();

    if (isEnabled && container) {
      activate();
    } else {
      deactivate();
    }

    onCleanup(() => {
      deactivate();
    });
  });

  return {
    activate,
    deactivate,
    refresh,
    getFocusableElements,
  };
}

/**
 * Simple focus trap for basic use cases
 */
export function createFocusTrap(container: HTMLElement, options: FocusTrapOptions = {}) {
  let previousActiveElement: Element | null = null;

  const getFocusableElements = (): HTMLElement[] => {
    return Array.from(container.querySelectorAll(FOCUSABLE_SELECTORS))
      .filter((el): el is HTMLElement => {
        const style = window.getComputedStyle(el);
        return style.display !== 'none' && style.visibility !== 'hidden';
      });
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key !== 'Tab') {
      if (e.key === 'Escape' && options.onEscape) {
        options.onEscape();
      }
      return;
    }

    const focusableElements = getFocusableElements();
    if (focusableElements.length === 0) {
      e.preventDefault();
      return;
    }

    const firstElement = focusableElements[0];
    const lastElement = focusableElements[focusableElements.length - 1];

    if (e.shiftKey) {
      if (document.activeElement === firstElement) {
        e.preventDefault();
        lastElement.focus();
      }
    } else {
      if (document.activeElement === lastElement) {
        e.preventDefault();
        firstElement.focus();
      }
    }
  };

  const activate = () => {
    previousActiveElement = document.activeElement;
    container.addEventListener('keydown', handleKeyDown);
    
    const initialElement = options.initialFocus?.() || getFocusableElements()[0];
    if (initialElement) {
      initialElement.focus();
    }
  };

  const deactivate = () => {
    container.removeEventListener('keydown', handleKeyDown);
    
    const returnEl = options.returnFocus?.() || previousActiveElement;
    if (returnEl && 'focus' in returnEl) {
      (returnEl as HTMLElement).focus();
    }
  };

  return { activate, deactivate };
}

export default useFocusTrap;
