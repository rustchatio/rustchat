// ============================================
// Accessibility Utilities
// WCAG 2.1 AA Compliance Helpers
// ============================================

// ============================================
// Screen Reader Only Text
// ============================================

/**
 * CSS class for screen-reader-only content
 * Visually hidden but accessible to assistive technologies
 */
export const srOnlyClass = `
  absolute w-px h-px p-0 -m-px overflow-hidden whitespace-nowrap border-0
  [clip:rect(0,0,0,0)]
`;

/**
 * Props for screen reader only elements
 */
export interface SROnlyProps {
  children: string;
  as?: 'span' | 'div' | 'p';
}

// ============================================
// ARIA Helpers
// ============================================

/**
 * Generate unique ID for ARIA relationships
 */
export function generateAriaId(prefix: string): string {
  return `${prefix}-${Math.random().toString(36).slice(2, 11)}`;
}

/**
 * Combine refs for ARIA describedby/labelledby
 */
export function combineAriaIds(...ids: (string | undefined)[]): string {
  return ids.filter(Boolean).join(' ');
}

// ============================================
// Focus Management
// ============================================

/**
 * Check if element is focusable
 */
export function isFocusable(element: Element): boolean {
  const focusableSelectors = [
    'button:not([disabled])',
    '[href]',
    'input:not([disabled])',
    'select:not([disabled])',
    'textarea:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
    '[contenteditable]',
  ];

  return focusableSelectors.some((selector) => element.matches(selector));
}

/**
 * Get all focusable elements within a container
 */
export function getFocusableElements(container: HTMLElement): HTMLElement[] {
  const selectors = [
    'button:not([disabled])',
    '[href]',
    'input:not([disabled])',
    'select:not([disabled])',
    'textarea:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
    '[contenteditable]',
  ].join(', ');

  return Array.from(container.querySelectorAll(selectors)).filter(
    (el): el is HTMLElement => {
      const style = window.getComputedStyle(el);
      return style.display !== 'none' && style.visibility !== 'hidden';
    }
  );
}

/**
 * Move focus to element safely
 */
export function focusElement(element: HTMLElement | null | undefined): void {
  if (element && 'focus' in element) {
    element.focus();
  }
}

/**
 * Move focus to first focusable element
 */
export function focusFirst(container: HTMLElement): void {
  const elements = getFocusableElements(container);
  if (elements.length > 0) {
    elements[0].focus();
  }
}

// ============================================
// Keyboard Navigation
// ============================================

/**
 * Key codes for common keys
 */
export const Keys = {
  Escape: 'Escape',
  Enter: 'Enter',
  Space: ' ',
  Tab: 'Tab',
  ArrowUp: 'ArrowUp',
  ArrowDown: 'ArrowDown',
  ArrowLeft: 'ArrowLeft',
  ArrowRight: 'ArrowRight',
  Home: 'Home',
  End: 'End',
  PageUp: 'PageUp',
  PageDown: 'PageDown',
} as const;

/**
 * Check if key is an arrow key
 */
export function isArrowKey(key: string): boolean {
  return [
    Keys.ArrowUp,
    Keys.ArrowDown,
    Keys.ArrowLeft,
    Keys.ArrowRight,
  ].includes(key as typeof Keys.ArrowUp);
}

/**
 * Handle list navigation with arrow keys
 */
export function handleListNavigation(
  event: KeyboardEvent,
  currentIndex: number,
  totalItems: number,
  onSelect: (index: number) => void
): void {
  if (totalItems === 0) return;

  let newIndex = currentIndex;

  switch (event.key) {
    case Keys.ArrowDown:
      event.preventDefault();
      newIndex = (currentIndex + 1) % totalItems;
      break;
    case Keys.ArrowUp:
      event.preventDefault();
      newIndex = currentIndex <= 0 ? totalItems - 1 : currentIndex - 1;
      break;
    case Keys.Home:
      event.preventDefault();
      newIndex = 0;
      break;
    case Keys.End:
      event.preventDefault();
      newIndex = totalItems - 1;
      break;
    default:
      return;
  }

  if (newIndex !== currentIndex) {
    onSelect(newIndex);
  }
}

// ============================================
// ARIA Role Helpers
// ============================================

/**
 * Get ARIA role props for common patterns
 */
export const ariaRoles = {
  /** Button that triggers a menu */
  menuButton: (expanded: boolean, controls: string) => ({
    role: 'button' as const,
    'aria-haspopup': 'menu' as const,
    'aria-expanded': expanded,
    'aria-controls': controls,
  }),

  /** Menu container */
  menu: (labelledBy?: string) => ({
    role: 'menu' as const,
    'aria-labelledby': labelledBy,
  }),

  /** Menu item */
  menuItem: (disabled?: boolean) => ({
    role: 'menuitem' as const,
    tabIndex: -1,
    'aria-disabled': disabled ?? false,
  }),

  /** Dialog/Modal */
  dialog: (labelledBy?: string, describedBy?: string) => ({
    role: 'dialog' as const,
    'aria-modal': true,
    'aria-labelledby': labelledBy,
    'aria-describedby': describedBy,
  }),

  /** Alert dialog */
  alertDialog: (labelledBy?: string, describedBy?: string) => ({
    role: 'alertdialog' as const,
    'aria-modal': true,
    'aria-labelledby': labelledBy,
    'aria-describedby': describedBy,
  }),

  /** Tab list */
  tabList: (label?: string) => ({
    role: 'tablist' as const,
    'aria-label': label,
  }),

  /** Tab */
  tab: (selected: boolean, controls: string) => ({
    role: 'tab' as const,
    'aria-selected': selected,
    'aria-controls': controls,
    tabIndex: selected ? 0 : -1,
  }),

  /** Tab panel */
  tabPanel: (labelledBy: string) => ({
    role: 'tabpanel' as const,
    'aria-labelledby': labelledBy,
    tabIndex: 0,
  }),

  /** Checkbox */
  checkbox: (checked: boolean, mixed?: boolean) => ({
    role: 'checkbox' as const,
    'aria-checked': mixed ? ('mixed' as const) : checked,
  }),

  /** Switch/Toggle */
  switch: (checked: boolean) => ({
    role: 'switch' as const,
    'aria-checked': checked,
  }),

  /** Tooltip */
  tooltip: (id: string) => ({
    role: 'tooltip' as const,
    id,
  }),

  /** Status/Alert */
  status: () => ({
    role: 'status' as const,
    'aria-live': 'polite' as const,
    'aria-atomic': true,
  }),

  /** Alert (important) */
  alert: () => ({
    role: 'alert' as const,
    'aria-live': 'assertive' as const,
    'aria-atomic': true,
  }),
};

// ============================================
// Accessibility Labels
// ============================================

/**
 * Create accessible button label
 */
export function buttonLabel(
  label: string,
  shortcut?: string
): { 'aria-label': string; title?: string } {
  if (shortcut) {
    return {
      'aria-label': `${label} (${shortcut})`,
      title: `${label} (${shortcut})`,
    };
  }
  return { 'aria-label': label };
}

/**
 * Create accessible count label
 */
export function countLabel(count: number, singular: string, plural: string): string {
  return `${count} ${count === 1 ? singular : plural}`;
}

/**
 * Create accessible loading label
 */
export function loadingLabel(what?: string): string {
  return what ? `Loading ${what}` : 'Loading';
}

// ============================================
// Color Contrast Helpers
// ============================================

/**
 * Calculate relative luminance (WCAG formula)
 */
function getLuminance(r: number, g: number, b: number): number {
  const [rs, gs, bs] = [r, g, b].map((c) => {
    c = c / 255;
    return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
  });
  return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
}

/**
 * Calculate contrast ratio between two colors
 * WCAG AA requires 4.5:1 for normal text, 3:1 for large text
 */
export function getContrastRatio(
  color1: { r: number; g: number; b: number },
  color2: { r: number; g: number; b: number }
): number {
  const l1 = getLuminance(color1.r, color1.g, color1.b);
  const l2 = getLuminance(color2.r, color2.g, color2.b);
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  return (lighter + 0.05) / (darker + 0.05);
}

// ============================================
// Reduced Motion
// ============================================

/**
 * Check if user prefers reduced motion
 */
export function prefersReducedMotion(): boolean {
  if (typeof window === 'undefined') return false;
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

/**
 * Get animation duration respecting reduced motion
 */
export function getAnimationDuration(defaultDuration: number): number {
  return prefersReducedMotion() ? 0 : defaultDuration;
}

// ============================================
// High Contrast Mode
// ============================================

/**
 * Check if user prefers high contrast
 */
export function prefersHighContrast(): boolean {
  if (typeof window === 'undefined') return false;
  return window.matchMedia('(prefers-contrast: high)').matches;
}

// ============================================
// Export All
// ============================================

export default {
  generateAriaId,
  combineAriaIds,
  isFocusable,
  getFocusableElements,
  focusElement,
  focusFirst,
  Keys,
  isArrowKey,
  handleListNavigation,
  ariaRoles,
  buttonLabel,
  countLabel,
  loadingLabel,
  getContrastRatio,
  prefersReducedMotion,
  getAnimationDuration,
  prefersHighContrast,
};
