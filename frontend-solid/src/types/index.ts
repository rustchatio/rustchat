import type { JSX } from 'solid-js';

/**
 * Common type definitions for RustChat Solid.js frontend
 */

// ============================================
// UI Component Types
// ============================================

export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger';
export type ButtonSize = 'sm' | 'md' | 'lg';

export type InputSize = 'sm' | 'md' | 'lg';

export type Theme =
  | 'light'
  | 'dark'
  | 'modern'
  | 'metallic'
  | 'futuristic'
  | 'high-contrast'
  | 'simple'
  | 'dynamic';

// ============================================
// Accessibility Types
// ============================================

export interface AriaProps {
  'aria-label'?: string;
  'aria-labelledby'?: string;
  'aria-describedby'?: string;
  'aria-expanded'?: boolean;
  'aria-haspopup'?: boolean | 'menu' | 'listbox' | 'tree' | 'grid' | 'dialog';
  'aria-controls'?: string;
  'aria-pressed'?: boolean | 'mixed';
  'aria-selected'?: boolean;
  'aria-hidden'?: boolean;
  'aria-live'?: 'off' | 'assertive' | 'polite';
  'aria-atomic'?: boolean;
  'aria-busy'?: boolean;
  'aria-disabled'?: boolean;
  role?:
    | 'alert'
    | 'alertdialog'
    | 'button'
    | 'checkbox'
    | 'dialog'
    | 'grid'
    | 'gridcell'
    | 'link'
    | 'listbox'
    | 'menu'
    | 'menubar'
    | 'menuitem'
    | 'option'
    | 'progressbar'
    | 'radio'
    | 'radiogroup'
    | 'separator'
    | 'slider'
    | 'switch'
    | 'tab'
    | 'tablist'
    | 'tabpanel'
    | 'textbox'
    | 'toolbar'
    | 'tooltip'
    | 'tree'
    | 'treeitem'
    | string;
}

// ============================================
// Common Props
// ============================================

export interface BaseProps {
  id?: string;
  class?: string;
  style?: string | JSX.CSSProperties;
}

export interface TestProps {
  'data-testid'?: string;
}

// ============================================
// Component Event Types
// ============================================

export type ButtonClickEvent = MouseEvent & {
  currentTarget: HTMLButtonElement;
  target: Element;
};

export type InputChangeEvent = Event & {
  currentTarget: HTMLInputElement;
  target: HTMLInputElement;
};

export type InputKeyboardEvent = KeyboardEvent & {
  currentTarget: HTMLInputElement;
};

export type FormSubmitEvent = Event & {
  currentTarget: HTMLFormElement;
  target: HTMLFormElement;
};
