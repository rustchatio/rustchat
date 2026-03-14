// ============================================
// App Component with Session Management
// ============================================

import { Suspense, lazy, onMount } from 'solid-js';
import { Router, Route } from '@solidjs/router';
import { ThemeProvider } from './stores/theme';
import { setupInterceptors } from './api/interceptors';
import { AppShell } from './components/layout';
import SessionTimeoutModal from './components/SessionTimeoutModal';
import ProtectedRoute from './components/ProtectedRoute';

// Eagerly load public routes
import Login from './routes/Login';
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

// ============================================
// Main App Routes Component
// ============================================

function AppRoutes() {
  onMount(() => {
    setupInterceptors();
  });

  return (
    <>
      {/* Public Routes - No AppShell */}
      <Route path="/login" component={Login} />
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
            <AppShell>
              <div class="flex items-center justify-center h-full text-text-3">
                <div class="text-center">
                  <div class="w-12 h-12 border-4 border-brand/30 border-t-brand rounded-full animate-spin mx-auto mb-4" />
                  <p>Redirecting...</p>
                </div>
              </div>
            </AppShell>
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

// ============================================
// App Content Component
// ============================================

function AppContent() {
  return (
    <div class="min-h-screen bg-bg-app text-text-1">
      <AppRoutes />
      <SessionTimeoutModal />
    </div>
  );
}

// ============================================
// Main App Component
// ============================================

function App() {
  return (
    <ThemeProvider>
      <Router>
        <AppContent />
      </Router>
    </ThemeProvider>
  );
}

export default App;
