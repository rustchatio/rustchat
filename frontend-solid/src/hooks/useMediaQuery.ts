// ============================================
// useMediaQuery Hook - Responsive Breakpoints
// ============================================

import { createSignal, createEffect, onCleanup, Accessor } from 'solid-js';

// ============================================
// Breakpoint Definitions
// ============================================

export const breakpoints = {
  sm: 640,   // Small devices (phones)
  md: 768,   // Medium devices (tablets)
  lg: 1024,  // Large devices (desktops)
  xl: 1280,  // Extra large devices
  '2xl': 1536, // Extra extra large
} as const;

export type Breakpoint = keyof typeof breakpoints;

// ============================================
// Hook: useMediaQuery
// ============================================

/**
 * Hook to track media query matches
 * @param query - CSS media query string
 * @returns Accessor<boolean> - whether the query matches
 */
export function useMediaQuery(query: string): Accessor<boolean> {
  const [matches, setMatches] = createSignal(false);

  createEffect(() => {
    if (typeof window === 'undefined') {
      setMatches(false);
      return;
    }

    const media = window.matchMedia(query);
    setMatches(media.matches);

    const handler = (e: MediaQueryListEvent) => {
      setMatches(e.matches);
    };

    media.addEventListener('change', handler);

    onCleanup(() => {
      media.removeEventListener('change', handler);
    });
  });

  return matches;
}

// ============================================
// Convenience Hooks
// ============================================

/**
 * Hook to check if viewport is mobile (< md breakpoint)
 */
export function useIsMobile(): Accessor<boolean> {
  return useMediaQuery(`(max-width: ${breakpoints.md - 1}px)`);
}

/**
 * Hook to check if viewport is tablet (md to lg)
 */
export function useIsTablet(): Accessor<boolean> {
  return useMediaQuery(`(min-width: ${breakpoints.md}px) and (max-width: ${breakpoints.lg - 1}px)`);
}

/**
 * Hook to check if viewport is desktop (>= lg)
 */
export function useIsDesktop(): Accessor<boolean> {
  return useMediaQuery(`(min-width: ${breakpoints.lg}px)`);
}

/**
 * Hook to check if viewport is large desktop (>= xl)
 */
export function useIsLargeDesktop(): Accessor<boolean> {
  return useMediaQuery(`(min-width: ${breakpoints.xl}px)`);
}

/**
 * Hook to check if viewport is touch device
 */
export function useIsTouchDevice(): Accessor<boolean> {
  return useMediaQuery('(pointer: coarse)');
}

/**
 * Hook to check if reduced motion is preferred
 */
export function usePrefersReducedMotion(): Accessor<boolean> {
  return useMediaQuery('(prefers-reduced-motion: reduce)');
}

/**
 * Hook to check dark mode preference
 */
export function usePrefersDarkMode(): Accessor<boolean> {
  return useMediaQuery('(prefers-color-scheme: dark)');
}

// ============================================
// Breakpoint Hook (returns current breakpoint)
// ============================================

export type BreakpointValue = Breakpoint | 'xs';

/**
 * Hook to get the current breakpoint name
 */
export function useBreakpoint(): Accessor<BreakpointValue> {
  const [breakpoint, setBreakpoint] = createSignal<BreakpointValue>('xs');

  createEffect(() => {
    if (typeof window === 'undefined') {
      setBreakpoint('xs');
      return;
    }

    const updateBreakpoint = () => {
      const width = window.innerWidth;
      if (width >= breakpoints['2xl']) setBreakpoint('2xl');
      else if (width >= breakpoints.xl) setBreakpoint('xl');
      else if (width >= breakpoints.lg) setBreakpoint('lg');
      else if (width >= breakpoints.md) setBreakpoint('md');
      else if (width >= breakpoints.sm) setBreakpoint('sm');
      else setBreakpoint('xs');
    };

    updateBreakpoint();

    window.addEventListener('resize', updateBreakpoint);

    onCleanup(() => {
      window.removeEventListener('resize', updateBreakpoint);
    });
  });

  return breakpoint;
}

// ============================================
// Combined Responsive Hook
// ============================================

export interface ResponsiveState {
  isMobile: boolean;
  isTablet: boolean;
  isDesktop: boolean;
  isLargeDesktop: boolean;
  isTouch: boolean;
  breakpoint: BreakpointValue;
}

/**
 * Hook to get all responsive state in one object
 */
export function useResponsive(): Accessor<ResponsiveState> {
  const isMobile = useIsMobile();
  const isTablet = useIsTablet();
  const isDesktop = useIsDesktop();
  const isLargeDesktop = useIsLargeDesktop();
  const isTouch = useIsTouchDevice();
  const breakpoint = useBreakpoint();

  return () => ({
    isMobile: isMobile(),
    isTablet: isTablet(),
    isDesktop: isDesktop(),
    isLargeDesktop: isLargeDesktop(),
    isTouch: isTouch(),
    breakpoint: breakpoint(),
  });
}

// ============================================
// Exports
// ============================================

export default useMediaQuery;
