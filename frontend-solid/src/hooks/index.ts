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

export {
  Sounds,
  SoundSettings,
  playNewMessageSound,
  playMentionSound,
  playDirectMessageSound,
  playSendSound,
  playErrorSound,
  playConnectedSound,
  playDisconnectedSound,
} from '../utils/sounds';

export { SkipLink, SkipToMain, SkipToNav, SkipLinks } from '../components/SkipLink';

export {
  srOnlyClass,
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
} from '../utils/a11y';

export { useFocusTrap, createFocusTrap } from './useFocusTrap';

export {
  announce,
  announceSuccess,
  announceError,
  LiveRegion,
} from '../components/LiveRegion';

export {
  ConnectionIndicator,
  ConnectionBadge,
  ConnectionStatusPanel,
  ConnectionToastNotifier,
} from '../components/ConnectionStatus';
