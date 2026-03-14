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

export { useDesktopNotifications, notifyNewMessage } from './useDesktopNotifications';
export type {
  NotificationPermissionState,
  NotificationOptions,
} from './useDesktopNotifications';

export { useToast, toast, addToast, removeToast, clearAllToasts } from './useToast';
export type { Toast, ToastType, ToastOptions } from './useToast';
