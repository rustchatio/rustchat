// ============================================
// Hooks Export
// ============================================

export {
  useMediaQuery,
  useIsMobile,
  useIsTablet,
  useIsDesktop,
  useIsLargeDesktop,
  useIsTouchDevice,
  usePrefersReducedMotion,
  usePrefersDarkMode,
  useBreakpoint,
  useResponsive,
  breakpoints,
} from './useMediaQuery';
export type {
  Breakpoint,
  BreakpointValue,
  ResponsiveState,
} from './useMediaQuery';

export { useWebSocket } from './useWebSocket';
export type { UseWebSocketReturn } from './useWebSocket';
