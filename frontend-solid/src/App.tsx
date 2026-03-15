// ============================================
// App Component with Session Management
// ============================================

import { Suspense, lazy, onMount, Show, createSignal } from 'solid-js';
import { Router, Route, useNavigate, type RouteSectionProps } from '@solidjs/router';
import { ThemeProvider } from './stores/theme';
import { authStore } from './stores/auth';
import { setupInterceptors } from './api/interceptors';
import { AppShell } from './components/layout';
import SessionTimeoutModal from './components/SessionTimeoutModal';
import ProtectedRoute from './components/ProtectedRoute';
import { resolveDefaultChannelPath } from './stores/channels';
import { ToastContainer } from './components/ToastContainer';
import { ConnectionToastNotifier } from './components/ConnectionStatus';
import { LiveRegion } from './components/LiveRegion';
import { SkipLinks } from './components/SkipLink';
import { ErrorBoundary, OfflineIndicator } from './components/ErrorBoundary';
import CommandPalette from './components/ui/CommandPalette';
import { isAdminRole } from './utils/roles';

// Eagerly load public routes
import Login from './routes/Login';
import Register from './routes/Register';
import LoginCallback from './routes/LoginCallback';
import ForgotPassword from './routes/ForgotPassword';
import ResetPassword from './routes/ResetPassword';
import Settings from './routes/Settings';

// Lazy load protected routes
const Channel = lazy(() => import('./routes/Channel'));
const Thread = lazy(() => import('./routes/Thread'));
const Admin = lazy(() => import('./routes/Admin'));
const NotFound = lazy(() => import('./routes/NotFound'));

// ============================================
// Loading Fallback Component
// ============================================

function LoadingFallback() {
  return (
    <div class="min-h-screen flex items-center justify-center bg-bg-app">
      <div class="flex flex-col items-center gap-4">
        <div class="w-12 h-12 border-4 border-brand/30 border-t-brand rounded-full animate-spin" />
        <p class="text-text-3">Loading...</p>
      </div>
    </div>
  );
}

function AuthenticatedRootRedirect() {
  const navigate = useNavigate();
  const [isResolving, setIsResolving] = createSignal(true);
  const [hasWorkspaceTarget, setHasWorkspaceTarget] = createSignal(true);
  const canAccessAdmin = () => isAdminRole(authStore.user()?.role);

  onMount(async () => {
    const targetPath = await resolveDefaultChannelPath();
    if (targetPath) {
      navigate(targetPath, { replace: true });
      return;
    }
    setHasWorkspaceTarget(false);
    setIsResolving(false);
  });

  return (
    <div class="min-h-screen flex items-center justify-center bg-bg-app px-4">
      <Show
        when={isResolving()}
        fallback={
          <Show when={!hasWorkspaceTarget()}>
            <div class="w-full max-w-xl rounded-xl border border-border-1 bg-bg-surface-1 p-6 space-y-4">
              <div>
                <h1 class="text-xl font-semibold text-text-1">No Channel Available Yet</h1>
                <p class="text-sm text-text-3 mt-1">
                  There is currently no channel in your visible workspace. You can open settings, or
                  use the admin console if you have access.
                </p>
              </div>
              <div class="flex flex-wrap gap-2">
                <button
                  type="button"
                  class="rounded-lg bg-brand px-4 py-2 text-sm font-medium text-white hover:bg-brand-hover"
                  onClick={() => navigate('/settings/profile')}
                >
                  Open Profile Settings
                </button>
                <Show when={canAccessAdmin()}>
                  <button
                    type="button"
                    class="rounded-lg border border-border-1 px-4 py-2 text-sm font-medium text-text-2 hover:bg-bg-surface-2"
                    onClick={() => navigate('/admin')}
                  >
                    Open Admin Console
                  </button>
                </Show>
              </div>
            </div>
          </Show>
        }
      >
        <div class="flex flex-col items-center gap-4">
          <div class="w-12 h-12 border-4 border-brand/30 border-t-brand rounded-full animate-spin" />
          <p class="text-text-3">Redirecting...</p>
        </div>
      </Show>
    </div>
  );
}

// ============================================
// Main App Routes Component
// ============================================

function AppRoutes() {
  return (
    <>
      {/* Public Routes - No AppShell */}
      <Route path="/login" component={Login} />
      <Route path="/register" component={Register} />
      <Route path="/login/callback" component={LoginCallback} />
      <Route path="/forgot-password" component={ForgotPassword} />
      <Route path="/reset-password" component={ResetPassword} />
      <Route path="/set-password" component={ResetPassword} />
      <Route path="/saml/callback" component={LoginCallback} />
      <Route path="/auth/saml/callback" component={LoginCallback} />

      {/* Protected Routes with AppShell */}
      <Route
        path="/"
        component={() => (
          <ProtectedRoute>
            <AuthenticatedRootRedirect />
          </ProtectedRoute>
        )}
      />
      <Route
        path="/channels/:channelId"
        component={() => (
          <ProtectedRoute>
            <AppShell>
              <Suspense fallback={<LoadingFallback />}>
                <Channel />
              </Suspense>
            </AppShell>
          </ProtectedRoute>
        )}
      />
      <Route
        path="/channels/:channelId/threads/:threadId"
        component={() => (
          <ProtectedRoute>
            <AppShell>
              <Suspense fallback={<LoadingFallback />}>
                <Thread />
              </Suspense>
            </AppShell>
          </ProtectedRoute>
        )}
      />
      <Route
        path="/settings/*"
        component={() => (
          <ProtectedRoute>
            <AppShell>
              <Suspense fallback={<LoadingFallback />}>
                <Settings />
              </Suspense>
            </AppShell>
          </ProtectedRoute>
        )}
      />
      <Route
        path="/admin/*"
        component={() => (
          <ProtectedRoute>
            <Suspense fallback={<LoadingFallback />}>
              <Admin />
            </Suspense>
          </ProtectedRoute>
        )}
      />

      {/* 404 Catch-all */}
      <Route path="*" component={NotFound} />
    </>
  );
}

function RouterLayout(props: RouteSectionProps) {
  onMount(() => {
    setupInterceptors();
    if (authStore.isAuthenticated && !authStore.user()) {
      void authStore.fetchMe();
    }
  });

  return (
    <ErrorBoundary>
      <div class="min-h-screen bg-bg-app text-text-1">
        <SkipLinks />
        <OfflineIndicator />
        {props.children}
        <SessionTimeoutModal />
        <CommandPalette />
        <ToastContainer />
        <ConnectionToastNotifier />
        <LiveRegion />
      </div>
    </ErrorBoundary>
  );
}

// ============================================
// Main App Component
// ============================================

function App() {
  return (
    <ThemeProvider>
      <Router root={RouterLayout}>
        <AppRoutes />
      </Router>
    </ThemeProvider>
  );
}

export default App;
