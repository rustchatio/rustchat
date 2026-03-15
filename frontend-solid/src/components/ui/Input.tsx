import { splitProps, mergeProps, Show, JSX } from 'solid-js';
import { cn } from '@/utils/cn';
import type { InputSize, BaseProps, TestProps } from '@/types';

// ============================================
// Input Props Interface
// ============================================

export interface InputProps extends BaseProps, TestProps {
  /** Input type */
  type?: 'text' | 'password' | 'email' | 'number' | 'search' | 'tel' | 'url';
  /** Input size */
  size?: InputSize;
  /** Input value */
  value?: string | number;
  /** Default value for uncontrolled input */
  defaultValue?: string | number;
  /** Placeholder text */
  placeholder?: string;
  /** Label text */
  label?: string;
  /** Helper text */
  helperText?: string;
  /** Error message */
  error?: string;
  /** Disabled state */
  disabled?: boolean;
  /** Read-only state */
  readOnly?: boolean;
  /** Required field */
  required?: boolean;
  /** Auto-focus on mount */
  autofocus?: boolean;
  /** Input name */
  name?: string;
  /** Input id (auto-generated if not provided) */
  id?: string;
  /** Browser autocomplete hint */
  autocomplete?: string;
  /** React-style alias for browser autocomplete hint */
  autoComplete?: string;
  /** Change handler */
  onInput?: (event: InputEvent & { currentTarget: HTMLInputElement }) => void;
  /** Change handler (on blur) */
  onChange?: (event: Event & { currentTarget: HTMLInputElement }) => void;
  /** Key down handler */
  onKeyDown?: (event: KeyboardEvent & { currentTarget: HTMLInputElement }) => void;
  /** Focus handler */
  onFocus?: (event: FocusEvent & { currentTarget: HTMLInputElement }) => void;
  /** Blur handler */
  onBlur?: (event: FocusEvent & { currentTarget: HTMLInputElement }) => void;
  /** ARIA label */
  'aria-label'?: string;
  /** ARIA labelledby */
  'aria-labelledby'?: string;
  /** ARIA describedby */
  'aria-describedby'?: string;
}

// ============================================
// Default Props
// ============================================

const defaultProps: Partial<InputProps> = {
  type: 'text',
  size: 'md',
  autoComplete: 'on',
};

// ============================================
// Size Styles
// ============================================

const sizeStyles: Record<InputSize, string> = {
  sm: 'px-3 py-1.5 text-sm',
  md: 'px-3 py-2 text-sm',
  lg: 'px-4 py-3 text-base',
};

const baseStyles = `w-full rounded-lg border border-border-1 bg-bg-surface-1 text-text-1
  placeholder:text-text-3 transition-all duration-200 focus:outline-none focus:ring-2
  focus:ring-brand/50 focus:border-brand disabled:opacity-50 disabled:cursor-not-allowed
  disabled:bg-bg-surface-2 read-only:bg-bg-surface-2 read-only:cursor-default`;

// ============================================
// Helper function for unique IDs
// ============================================

let inputIdCounter = 0;
function generateInputId(): string {
  return `input-${++inputIdCounter}`;
}

// ============================================
// Input Component
// ============================================

export function Input(props: InputProps): JSX.Element {
  const merged = mergeProps(defaultProps, props);
  const [local, rest] = splitProps(merged, ['size', 'label', 'helperText', 'error', 'class', 'id', 'autocomplete', 'autoComplete']);

  const inputId = () => local.id ?? generateInputId();
  const helperId = () => `${inputId()}-helper`;
  const errorId = () => `${inputId()}-error`;
  const hasError = () => !!local.error;
  const resolvedAutoComplete = () => local.autoComplete ?? local.autocomplete ?? 'on';

  const ariaDescribedBy = () => {
    const ids: string[] = [];
    if (local.helperText) ids.push(helperId());
    if (hasError()) ids.push(errorId());
    return ids.length > 0 ? ids.join(' ') : undefined;
  };

  return (
    <div class={cn('w-full', local.class)}>
      <Show when={local.label}>
        <label for={inputId()} class="block text-sm font-medium text-text-1 mb-1">
          {local.label}
          <Show when={rest.required}>
            <span class="text-danger ml-1" aria-label="required">
              *
            </span>
          </Show>
        </label>
      </Show>

      <input
        id={inputId()}
        autocomplete={resolvedAutoComplete()}
        class={cn(
          baseStyles,
          sizeStyles[local.size as InputSize],
          hasError() && 'border-danger focus:ring-danger/50 focus:border-danger'
        )}
        aria-invalid={hasError()}
        aria-describedby={ariaDescribedBy()}
        {...rest}
      />

      <Show when={local.helperText && !hasError()}>
        <p id={helperId()} class="mt-1 text-sm text-text-3">
          {local.helperText}
        </p>
      </Show>

      <Show when={hasError()}>
        <p id={errorId()} class="mt-1 text-sm text-danger" role="alert">
          {local.error}
        </p>
      </Show>
    </div>
  );
}

export default Input;
