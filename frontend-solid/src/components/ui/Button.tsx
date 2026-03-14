import { Button as KobalteButton } from '@kobalte/core';
import { splitProps, mergeProps, JSX, Show } from 'solid-js';
import { cn } from '@/utils/cn';
import type { ButtonVariant, ButtonSize, BaseProps, TestProps } from '@/types';

// ============================================
// Button Props Interface
// ============================================

export interface ButtonProps extends BaseProps, TestProps {
  /** Button visual variant */
  variant?: ButtonVariant;
  /** Button size */
  size?: ButtonSize;
  /** Full width button */
  fullWidth?: boolean;
  /** Loading state */
  loading?: boolean;
  /** Disabled state */
  disabled?: boolean;
  /** Button type */
  type?: 'button' | 'submit' | 'reset';
  /** Click handler */
  onClick?: (event: MouseEvent & { currentTarget: HTMLButtonElement }) => void;
  /** Children content */
  children: JSX.Element;
  /** ARIA label */
  'aria-label'?: string;
  /** ARIA pressed state */
  'aria-pressed'?: boolean | 'mixed';
  /** ARIA expanded state */
  'aria-expanded'?: boolean;
  /** ARIA haspopup */
  'aria-haspopup'?: boolean | 'menu' | 'listbox' | 'tree' | 'grid' | 'dialog';
  /** ARIA controls */
  'aria-controls'?: string;
}

// ============================================
// Default Props
// ============================================

const defaultProps: Partial<ButtonProps> = {
  variant: 'primary',
  size: 'md',
  type: 'button',
};

// ============================================
// Variant Styles
// ============================================

const variantStyles: Record<ButtonVariant, string> = {
  primary: 'bg-brand text-white hover:bg-brand-hover focus:ring-brand/50',
  secondary:
    'bg-bg-surface-2 text-text-1 hover:bg-border-1 focus:ring-brand/50 border border-border-1',
  ghost: 'bg-transparent text-text-2 hover:bg-bg-surface-2 hover:text-text-1 focus:ring-brand/50',
  danger: 'bg-danger text-white hover:bg-red-700 focus:ring-danger/50',
};

const sizeStyles: Record<ButtonSize, string> = {
  sm: 'px-3 py-1.5 text-sm',
  md: 'px-4 py-2 text-sm',
  lg: 'px-6 py-3 text-base',
};

const baseStyles = `inline-flex items-center justify-center gap-2 rounded-lg font-medium
  transition-all duration-200 ease-out focus:outline-none focus:ring-2 focus:ring-offset-2
  disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none cursor-pointer`;

// ============================================
// Button Component
// ============================================

export function Button(props: ButtonProps): JSX.Element {
  const merged = mergeProps(defaultProps, props);
  const [local, rest] = splitProps(merged, [
    'variant',
    'size',
    'fullWidth',
    'loading',
    'disabled',
    'class',
    'children',
    'onClick',
  ]);

  const isDisabled = () => local.disabled || local.loading;

  const handleClick = (event: MouseEvent & { currentTarget: HTMLButtonElement }) => {
    if (isDisabled()) return;
    local.onClick?.(event);
  };

  return (
    <KobalteButton.Root
      class={cn(
        baseStyles,
        variantStyles[local.variant as ButtonVariant],
        sizeStyles[local.size as ButtonSize],
        local.fullWidth && 'w-full',
        local.loading && 'cursor-wait',
        local.class
      )}
      disabled={isDisabled()}
      onClick={handleClick}
      {...rest}
    >
      <Show when={local.loading}>
        <span
          class="inline-block w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin"
          aria-hidden="true"
        />
        <span class="sr-only">Loading</span>
      </Show>
      {local.children}
    </KobalteButton.Root>
  );
}

export default Button;
