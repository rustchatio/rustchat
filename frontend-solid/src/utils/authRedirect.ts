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

  try {
    const parsed = new URL(path, window.location.origin);
    if (AUTH_BLOCKED_PATHS.has(parsed.pathname)) return DEFAULT_AUTH_REDIRECT;
    const query = parsed.searchParams.toString();
    return `${parsed.pathname}${query ? `?${query}` : ''}${parsed.hash}`;
  } catch {
    return DEFAULT_AUTH_REDIRECT;
  }
}

export function getDefaultAuthRedirectPath(): string {
  return DEFAULT_AUTH_REDIRECT;
}
