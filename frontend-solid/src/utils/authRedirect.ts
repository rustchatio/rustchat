const DEFAULT_AUTH_REDIRECT = '/';

const AUTH_BLOCKED_PATHS = new Set([
  '/login',
  '/register',
  '/login/callback',
  '/saml/callback',
  '/auth/saml/callback',
  '/channels/general',
]);

export function normalizeAuthRedirectPath(path: string | null | undefined): string {
  if (!path) return DEFAULT_AUTH_REDIRECT;
  if (!path.startsWith('/') || path.startsWith('//')) return DEFAULT_AUTH_REDIRECT;
  if (AUTH_BLOCKED_PATHS.has(path)) return DEFAULT_AUTH_REDIRECT;
  return path;
}

export function getDefaultAuthRedirectPath(): string {
  return DEFAULT_AUTH_REDIRECT;
}
