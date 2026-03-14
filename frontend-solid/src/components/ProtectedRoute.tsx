import { JSX, createEffect } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { authStore } from '../stores/auth';

interface ProtectedRouteProps {
  children: JSX.Element;
  fallback?: JSX.Element;
}

export function ProtectedRoute(props: ProtectedRouteProps) {
  const navigate = useNavigate();

  createEffect(() => {
    if (!authStore.isAuthenticated) {
      navigate('/login', { replace: true });
    }
  });

  // Show fallback or nothing while checking auth
  if (!authStore.isAuthenticated) {
    return <>{props.fallback || null}</>;
  }

  return <>{props.children}</>;
}

export default ProtectedRoute;
