// ============================================
// Error Boundary - Global Error Handling
// ============================================

import { ErrorBoundary as SolidErrorBoundary, JSX, Show } from 'solid-js';
import { createSignal, onMount, onCleanup } from 'solid-js';
import { Button } from './ui';

// ============================================
// Types
// ============================================

interface ErrorBoundaryProps {
  children: JSX.Element;
  fallback?: (props: { error: Error; reset: () => void }) => JSX.Element;
}

// ============================================
// Default Error Fallback
// ============================================

interface FallbackProps {
  error: Error;
  reset: () => void;
}

function DefaultErrorFallback(props: FallbackProps) {
  return (
    <div class="min-h-screen flex items-center justify-center bg-bg-app p-4">
      <div class="max-w-md w-full bg-bg-surface-1 border border-border-1 rounded-xl p-6 text-center">
        <div class="w-16 h-16 bg-danger/10 rounded-full flex items-center justify-center mx-auto mb-4">
          <span class="text-3xl">⚠️</span>
        </div>
        
        <h1 class="text-xl font-bold text-text-1 mb-2">Something went wrong</h1>
        <p class="text-text-3 text-sm mb-6">
          An unexpected error occurred. Please try refreshing the page.
        </p>

        <div class="bg-bg-app border border-border-1 rounded-lg p-3 mb-6 text-left overflow-auto">
          <code class="text-xs text-danger font-mono">
            {props.error.message}
          </code>
        </div>

        <div class="flex gap-3 justify-center">
          <Button variant="primary" onClick={props.reset}>
            Try Again
          </Button>
          <Button variant="secondary" onClick={() => window.location.reload()}>
            Refresh Page
          </Button>
        </div>
      </div>
    </div>
  );
}

// ============================================
// Error Boundary Component
// ============================================

export function ErrorBoundary(props: ErrorBoundaryProps) {
  return (
    <SolidErrorBoundary
      fallback={(err, reset) => {
        const error = err instanceof Error ? err : new Error(String(err));
        const Fallback = props.fallback || DefaultErrorFallback;
        return <Fallback error={error} reset={reset} />;
      }}
    >
      {props.children}
    </SolidErrorBoundary>
  );
}

// ============================================
// Error Reporting Utility
// ============================================

export const ErrorReporting = {
  report(error: Error, context?: Record<string, unknown>) {
    console.error('Error reported:', error, context);

    if (typeof window !== 'undefined' && (window as any).Sentry) {
      (window as any).Sentry.withScope((scope: any) => {
        if (context) {
          Object.entries(context).forEach(([key, value]) => {
            scope.setExtra(key, value);
          });
        }
        (window as any).Sentry.captureException(error);
      });
    }
  },

  log(message: string, level: 'info' | 'warn' | 'error' = 'info') {
    const consoleMethod = level === 'warn' ? 'warn' : level === 'error' ? 'error' : 'log';
    console[consoleMethod](message);

    if (typeof window !== 'undefined' && (window as any).Sentry) {
      (window as any).Sentry.captureMessage(message, level);
    }
  },
};

// ============================================
// Offline Detection Hook
// ============================================

export function useOfflineDetection() {
  const [isOnline, setIsOnline] = createSignal(true);
  const [wasOffline, setWasOffline] = createSignal(false);

  onMount(() => {
    setIsOnline(navigator.onLine);

    const handleOnline = () => {
      setIsOnline(true);
      setWasOffline(true);
      setTimeout(() => setWasOffline(false), 3000);
    };

    const handleOffline = () => {
      setIsOnline(false);
    };

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    onCleanup(() => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    });
  });

  return {
    isOnline,
    wasOffline,
    isOffline: () => !isOnline(),
  };
}

// ============================================
// Offline Indicator Component
// ============================================

export function OfflineIndicator() {
  const { isOnline, wasOffline } = useOfflineDetection();

  return (
    <>
      <Show when={!isOnline()}>
        <div class="fixed top-0 left-0 right-0 z-50 bg-warning text-warning-foreground px-4 py-2 text-center text-sm font-medium">
          <span class="inline-flex items-center gap-2">
            <span>📡</span>
            You're offline. Some features may be unavailable.
          </span>
        </div>
      </Show>

      <Show when={wasOffline() && isOnline()}>
        <div class="fixed top-0 left-0 right-0 z-50 bg-success text-success-foreground px-4 py-2 text-center text-sm font-medium">
          <span class="inline-flex items-center gap-2">
            <span>✅</span>
            Back online!
          </span>
        </div>
      </Show>
    </>
  );
}

export default ErrorBoundary;
