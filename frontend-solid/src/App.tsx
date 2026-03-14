// ============================================
// App Component with Session Management
// ============================================

import { Suspense, lazy, onMount } from 'solid-js';
import { Router, Route, useNavigate, type RouteSectionProps } from '@solidjs/router';
import { ThemeProvider } from './stores/theme';
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

// Eagerly load public routes
import Login from './routes/Login';
import Register from './routes/Register';
import LoginCallback from './routes/LoginCallback';
import ForgotPassword from './routes/ForgotPassword';
import ResetPassword from './routes/ResetPassword';

// Lazy load protected routes
const Channel = lazy(() => import('./routes/Channel'));
const Thread = lazy(() => import('./routes/Thread'));
const Settings = lazy(() => import('./routes/Settings'));
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

  onMount(async () => {
    const targetPath = await resolveDefaultChannelPath();
    navigate(targetPath || '/settings/profile', { replace: true });
  });

  return (
    <div class="min-h-screen flex items-center justify-center bg-bg-app">
      <div class="flex flex-col items-center gap-4">
        <div class="w-12 h-12 border-4 border-brand/30 border-t-brand rounded-full animate-spin" />
        <p class="text-text-3">Redirecting...</p>
      </div>
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

      {/* 404 Catch-all */}
      <Route path="*" component={NotFound} />
    </>
  );
}

function RouterLayout(props: RouteSectionProps) {
  onMount(() => {
    setupInterceptors();
  });

  return (
    <ErrorBoundary>
      <div class="min-h-screen bg-bg-app text-text-1">
        <SkipLinks />
        <OfflineIndicator />
        {props.children}
        <SessionTimeoutModal />
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
