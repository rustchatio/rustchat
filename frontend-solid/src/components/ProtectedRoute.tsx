import { JSX, createEffect } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { authStore } from '../stores/auth';

interface ProtectedRouteProps {
  children: JSX.Element;
  fallback?: JSX.Element;
}

function isPublicPath(pathname: string): boolean {
  return pathname === '/login'
    || pathname === '/register'
    || pathname === '/forgot-password'
    || pathname === '/reset-password'
    || pathname === '/login/callback'
    || pathname === '/saml/callback'
    || pathname === '/auth/saml/callback';
}

export function ProtectedRoute(props: ProtectedRouteProps) {
  const navigate = useNavigate();

  createEffect(() => {
    if (!authStore.isAuthenticated) {
      if (isPublicPath(window.location.pathname)) {
        return;
      }

      const requestedPath = `${window.location.pathname}${window.location.search}${window.location.hash}`;
      const redirect = encodeURIComponent(requestedPath);
      navigate(`/login?redirect=${redirect}`, { replace: true });
    }
  });

  // Use a function to check auth state reactively
  // In Solid, we need to access the store property to track reactivity
  const isAuth = () => authStore.isAuthenticated;

  return (
    <>
      {isAuth() ? props.children : props.fallback || null}
    </>
  );
}

export default ProtectedRoute;
