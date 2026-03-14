// ============================================
// Route Definitions for RustChat
// ============================================

import { lazy } from 'solid-js';
import { Route, Navigate } from '@solidjs/router';
import ProtectedRoute from '../components/ProtectedRoute';

// Lazy load route components for code splitting
const Login = lazy(() => import('./Login'));
const LoginCallback = lazy(() => import('./LoginCallback'));
const ForgotPassword = lazy(() => import('./ForgotPassword'));
const ResetPassword = lazy(() => import('./ResetPassword'));
const Channel = lazy(() => import('./Channel'));
const Thread = lazy(() => import('./Thread'));
const Settings = lazy(() => import('./Settings'));
const NotFound = lazy(() => import('./NotFound'));

// ============================================
// Route Components
// ============================================

export function AppRoutes() {
  return (
    <>
      {/* Public Routes */}
      <Route path="/login" component={Login} />
      <Route path="/login/callback" component={LoginCallback} />
      <Route path="/forgot-password" component={ForgotPassword} />
      <Route path="/reset-password" component={ResetPassword} />

      {/* SAML callback can be at different paths */}
      <Route path="/saml/callback" component={LoginCallback} />
      <Route path="/auth/saml/callback" component={LoginCallback} />

      {/* Protected Routes */}
      <Route
        path="/"
        component={() => (
          <ProtectedRoute>
            <Navigate href="/settings/profile" />
          </ProtectedRoute>
        )}
      />
      <Route
        path="/channels/:channelId"
        component={() => (
          <ProtectedRoute>
            <Channel />
          </ProtectedRoute>
        )}
      />
      <Route
        path="/channels/:channelId/threads/:threadId"
        component={() => (
          <ProtectedRoute>
            <Thread />
          </ProtectedRoute>
        )}
      />
      <Route
        path="/settings/*"
        component={() => (
          <ProtectedRoute>
            <Settings />
          </ProtectedRoute>
        )}
      />

      {/* 404 Catch-all */}
      <Route path="*" component={NotFound} />
    </>
  );
}

export default AppRoutes;
