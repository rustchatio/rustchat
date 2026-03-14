// ============================================
// Skip Link - Accessibility Navigation Aid
// Allows keyboard users to skip to main content
// ============================================

import { cn } from '../utils/cn';

interface SkipLinkProps {
  targetId: string;
  label?: string;
  class?: string;
}

/**
 * Skip Link Component
 * 
 * WCAG 2.1: 2.4.1 Bypass Blocks - A mechanism is available to bypass blocks
 * of content that are repeated on multiple Web pages.
 */
export function SkipLink(props: SkipLinkProps) {
  const handleClick = (e: MouseEvent) => {
    e.preventDefault();
    const target = document.getElementById(props.targetId);
    if (target) {
      target.focus();
      target.scrollIntoView({ behavior: 'smooth' });
    }
  };

  return (
    <a
      href={`#${props.targetId}`}
      onClick={handleClick}
      class={cn(
        // Hidden by default
        'sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4',
        // Visible when focused
        'focus:z-50 focus:px-4 focus:py-2',
        'focus:bg-brand focus:text-white',
        'focus:rounded-lg focus:shadow-lg',
        'focus:font-medium focus:text-sm',
        'transition-all duration-200',
        props.class
      )}
    >
      {props.label || `Skip to ${props.targetId}`}
    </a>
  );
}

/**
 * Skip to main content link (most common use case)
 */
export function SkipToMain(props: { label?: string }) {
  return <SkipLink targetId="main-content" label={props.label || 'Skip to main content'} />;
}

/**
 * Skip to navigation link
 */
export function SkipToNav(props: { label?: string }) {
  return <SkipLink targetId="main-nav" label={props.label || 'Skip to navigation'} />;
}

/**
 * Multiple skip links for complex layouts
 */
export function SkipLinks() {
  return (
    <div class="sr-only focus-within:not-sr-only focus-within:absolute focus-within:top-0 focus-within:left-0 focus-within:right-0 focus-within:z-50 focus-within:bg-bg-surface-1 focus-within:border-b focus-within:border-border-1 focus-within:p-4 focus-within:shadow-lg">
      <nav aria-label="Skip links">
        <ul class="flex flex-wrap gap-4">
          <li>
            <SkipLink targetId="main-content" label="Skip to main content" />
          </li>
          <li>
            <SkipLink targetId="channel-sidebar" label="Skip to channel list" />
          </li>
          <li>
            <SkipLink targetId="message-list" label="Skip to messages" />
          </li>
        </ul>
      </nav>
    </div>
  );
}

export default SkipLink;
