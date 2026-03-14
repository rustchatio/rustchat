// ============================================
// Auth Store with Session Management
// ============================================

import { createStore, produce } from 'solid-js/store';
import { createSignal, createEffect, batch, onCleanup } from 'solid-js';

// ============================================
// Types
// ============================================

export type LogoutReason = 'manual' | 'expired' | 'unauthorized' | 'session_conflict';

export interface AuthState {
  token: string;
  refreshToken: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
}

export interface LoginCredentials {
  username: string;
  password: string;
  remember?: boolean;
}

export interface LoginResponse {
  token: string;
  refresh_token?: string;
  expires_in?: number;
  user: User;
}

export interface User {
  id: string;
  username: string;
  email: string;
  first_name?: string;
  last_name?: string;
  display_name?: string;
  nickname?: string;
  position?: string;
  avatar_url?: string;
  role: string;
  presence: 'online' | 'away' | 'dnd' | 'offline';
  status_text?: string;
  status_emoji?: string;
  status_expires_at?: string;
  custom_status?: {
    text?: string;
    emoji?: string;
    expires_at?: string;
  };
  created_at: string;
}

export interface SessionWarning {
  show: boolean;
  expiresAt: number;
  remainingSeconds: number;
}

// ============================================
// Constants
// ============================================

const STORAGE_KEY = 'rustchat_auth_token';
const REFRESH_TOKEN_KEY = 'rustchat_refresh_token';
const TOKEN_EXPIRY_KEY = 'rustchat_token_expiry';
const SESSION_CHECK_INTERVAL = 30000; // 30 seconds
const REFRESH_BEFORE_EXPIRY = 60000; // 1 minute before expiry
const WARNING_BEFORE_EXPIRY = 300000; // 5 minutes warning

// ============================================
// JWT Helpers
// ============================================

function parseJwtExpiryMs(tokenValue: string): number | null {
  if (!tokenValue) return null;

  const parts = tokenValue.split('.');
  if (parts.length < 2) return null;

  const payloadPart = parts[1];
  if (!payloadPart) return null;

  const normalized = payloadPart.replace(/-/g, '+').replace(/_/g, '/');
  const paddingLength = (4 - (normalized.length % 4)) % 4;
  const padded = normalized + '='.repeat(paddingLength);

  try {
    const payload = JSON.parse(atob(padded)) as { exp?: unknown };
    const expSeconds = Number(payload.exp);
    if (!Number.isFinite(expSeconds) || expSeconds <= 0) return null;
    return expSeconds * 1000;
  } catch {
    return null;
  }
}

function parseJwtIssuedAt(tokenValue: string): number | null {
  if (!tokenValue) return null;

  const parts = tokenValue.split('.');
  if (parts.length < 2) return null;

  const payloadPart = parts[1];
  if (!payloadPart) return null;

  const normalized = payloadPart.replace(/-/g, '+').replace(/_/g, '/');
  const paddingLength = (4 - (normalized.length % 4)) % 4;
  const padded = normalized + '='.repeat(paddingLength);

  try {
    const payload = JSON.parse(atob(padded)) as { iat?: unknown };
    const iatSeconds = Number(payload.iat);
    if (!Number.isFinite(iatSeconds) || iatSeconds <= 0) return null;
    return iatSeconds * 1000;
  } catch {
    return null;
  }
}

function setAuthCookie(tokenValue: string) {
  document.cookie = `MMAUTHTOKEN=${tokenValue}; path=/; SameSite=Strict`;
}

function clearAuthCookie() {
  document.cookie = 'MMAUTHTOKEN=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT';
}

// ============================================
// Store Creation
// ============================================

const getInitialToken = (): string => {
  if (typeof window === 'undefined') return '';
  return localStorage.getItem(STORAGE_KEY) || '';
};

const getInitialRefreshToken = (): string | null => {
  if (typeof window === 'undefined') return null;
  return localStorage.getItem(REFRESH_TOKEN_KEY);
};

const [state, setState] = createStore<AuthState>({
  token: getInitialToken(),
  refreshToken: getInitialRefreshToken(),
  isAuthenticated: !!getInitialToken(),
  isLoading: false,
  error: null,
});

const [user, setUser] = createSignal<User | null>(null);
const [sessionWarning, setSessionWarning] = createSignal<SessionWarning>({
  show: false,
  expiresAt: 0,
  remainingSeconds: 0,
});

// Internal state
let tokenExpiryTimer: ReturnType<typeof setTimeout> | null = null;
let refreshTimer: ReturnType<typeof setTimeout> | null = null;
let warningTimer: ReturnType<typeof setTimeout> | null = null;
let sessionCheckInterval: ReturnType<typeof setInterval> | null = null;
let isLoggingOut = false;
let isRefreshing = false;
let refreshPromise: Promise<boolean> | null = null;
let concurrentSessionId: string | null = null;

// ============================================
// Token Expiry & Refresh Management
// ============================================

function clearAllTimers() {
  if (tokenExpiryTimer) {
    clearTimeout(tokenExpiryTimer);
    tokenExpiryTimer = null;
  }
  if (refreshTimer) {
    clearTimeout(refreshTimer);
    refreshTimer = null;
  }
  if (warningTimer) {
    clearTimeout(warningTimer);
    warningTimer = null;
  }
  if (sessionCheckInterval) {
    clearInterval(sessionCheckInterval);
    sessionCheckInterval = null;
  }
}

function scheduleTokenExpiryLogout(expiryMs: number) {
  clearAllTimers();

  const remainingMs = expiryMs - Date.now();
  if (remainingMs <= 0) {
    logout('expired');
    return;
  }

  // Schedule warning
  if (remainingMs > WARNING_BEFORE_EXPIRY) {
    warningTimer = setTimeout(() => {
      setSessionWarning({
        show: true,
        expiresAt: expiryMs,
        remainingSeconds: Math.floor(WARNING_BEFORE_EXPIRY / 1000),
      });
    }, remainingMs - WARNING_BEFORE_EXPIRY);
  }

  // Schedule token refresh
  if (remainingMs > REFRESH_BEFORE_EXPIRY && state.refreshToken) {
    refreshTimer = setTimeout(() => {
      refreshToken();
    }, remainingMs - REFRESH_BEFORE_EXPIRY);
  }

  // Schedule expiry logout
  tokenExpiryTimer = setTimeout(() => {
    logout('expired');
  }, remainingMs);

  // Start periodic session check
  sessionCheckInterval = setInterval(() => {
    checkSessionHealth();
  }, SESSION_CHECK_INTERVAL);
}

function checkSessionHealth() {
  // Check if token is still valid
  const expiryMs = parseJwtExpiryMs(state.token);
  if (!expiryMs) return;

  const remainingMs = expiryMs - Date.now();

  // Update warning countdown
  if (sessionWarning().show && remainingMs > 0) {
    setSessionWarning((prev) => ({
      ...prev,
      remainingSeconds: Math.max(0, Math.floor(remainingMs / 1000)),
    }));
  }

  // Check for concurrent session conflicts
  checkConcurrentSession();
}

// ============================================
// Concurrent Session Detection
// ============================================

function checkConcurrentSession() {
  const tokenIat = parseJwtIssuedAt(state.token);
  if (!tokenIat) return;

  // Store the issue time of this session
  const storedIat = localStorage.getItem('rustchat_session_iat');
  const currentSessionId = localStorage.getItem('rustchat_session_id');

  if (!currentSessionId) {
    // Initialize session
    const newSessionId = crypto.randomUUID();
    localStorage.setItem('rustchat_session_id', newSessionId);
    localStorage.setItem('rustchat_session_iat', tokenIat.toString());
    concurrentSessionId = newSessionId;
    return;
  }

  concurrentSessionId = currentSessionId;

  // Check for newer session
  if (storedIat && parseInt(storedIat, 10) > tokenIat) {
    // Another session was started after this one
    logout('session_conflict');
  }
}

function initializeSessionTracking() {
  if (!state.token) return;

  const tokenIat = parseJwtIssuedAt(state.token);
  if (!tokenIat) return;

  const currentSessionId = localStorage.getItem('rustchat_session_id');
  if (!currentSessionId) {
    const newSessionId = crypto.randomUUID();
    localStorage.setItem('rustchat_session_id', newSessionId);
    localStorage.setItem('rustchat_session_iat', tokenIat.toString());
    concurrentSessionId = newSessionId;
  } else {
    concurrentSessionId = currentSessionId;
  }

  // Listen for storage changes (other tabs)
  const handleStorageChange = (e: StorageEvent) => {
    if (e.key === 'rustchat_session_id' && e.newValue !== concurrentSessionId) {
      // Another tab started a new session
      logout('session_conflict');
    }
  };

  window.addEventListener('storage', handleStorageChange);

  // Cleanup on logout
  onCleanup(() => {
    window.removeEventListener('storage', handleStorageChange);
  });
}

// ============================================
// Persistence Effects
// ============================================

// Persist token to localStorage
createEffect(() => {
  const token = state.token;
  const refreshToken = state.refreshToken;

  if (token) {
    localStorage.setItem(STORAGE_KEY, token);
    setAuthCookie(token);

    const expiryMs = parseJwtExpiryMs(token);
    if (expiryMs) {
      scheduleTokenExpiryLogout(expiryMs);
      localStorage.setItem(TOKEN_EXPIRY_KEY, expiryMs.toString());
    }

    initializeSessionTracking();
  } else {
    localStorage.removeItem(STORAGE_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);
    localStorage.removeItem(TOKEN_EXPIRY_KEY);
    localStorage.removeItem('rustchat_session_id');
    localStorage.removeItem('rustchat_session_iat');
    clearAuthCookie();
    clearAllTimers();
  }

  if (refreshToken) {
    localStorage.setItem(REFRESH_TOKEN_KEY, refreshToken);
  }
});

// ============================================
// Auth Actions
// ============================================

export async function login(credentials: LoginCredentials): Promise<void> {
  batch(() => {
    setState('isLoading', true);
    setState('error', null);
  });

  try {
    const response = await fetch('/api/v1/auth/login', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(credentials),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(error.message || 'Login failed');
    }

    const data: LoginResponse = await response.json();

    batch(() => {
      setState('token', data.token);
      setState('refreshToken', data.refresh_token || null);
      setState('isAuthenticated', true);
      setUser(data.user);
    });

    setAuthCookie(data.token);

    // Initialize session tracking
    const tokenIat = parseJwtIssuedAt(data.token);
    if (tokenIat) {
      const sessionId = crypto.randomUUID();
      localStorage.setItem('rustchat_session_id', sessionId);
      localStorage.setItem('rustchat_session_iat', tokenIat.toString());
      concurrentSessionId = sessionId;
    }

    // Fetch full profile
    await fetchMe();
  } catch (err) {
    batch(() => {
      setState('error', err instanceof Error ? err.message : 'Login failed');
      setState('isAuthenticated', false);
    });
    throw err;
  } finally {
    setState('isLoading', false);
  }
}

export async function loginWithToken(token: string, refreshToken?: string): Promise<void> {
  batch(() => {
    setState('token', token);
    setState('refreshToken', refreshToken || null);
    setState('isAuthenticated', true);
    setState('error', null);
  });

  setAuthCookie(token);

  // Initialize session tracking
  const tokenIat = parseJwtIssuedAt(token);
  if (tokenIat) {
    const sessionId = crypto.randomUUID();
    localStorage.setItem('rustchat_session_id', sessionId);
    localStorage.setItem('rustchat_session_iat', tokenIat.toString());
    concurrentSessionId = sessionId;
  }

  await fetchMe();
}

export async function fetchMe(): Promise<void> {
  if (!state.token) return;

  // Sync cookie on page reload
  setAuthCookie(state.token);

  try {
    const response = await fetch('/api/v1/auth/me', {
      headers: {
        Authorization: `Bearer ${state.token}`,
      },
    });

    if (!response.ok) {
      throw new Error('Failed to fetch user');
    }

    const data: User = await response.json();

    // Map custom_status fields for easier access
    if (data.custom_status) {
      data.status_text = data.custom_status.text;
      data.status_emoji = data.custom_status.emoji;
      data.status_expires_at = data.custom_status.expires_at;
    }

    setUser(data);
  } catch (e) {
    await logout('unauthorized');
  }
}

export async function logout(reason: LogoutReason = 'manual'): Promise<void> {
  if (isLoggingOut) return;
  isLoggingOut = true;

  clearAllTimers();

  // Clear session warning
  setSessionWarning({ show: false, expiresAt: 0, remainingSeconds: 0 });

  try {
    // Call logout endpoint if authenticated
    if (state.token) {
      try {
        await fetch('/api/v1/auth/logout', {
          method: 'POST',
          headers: {
            Authorization: `Bearer ${state.token}`,
          },
        });
      } catch {
        // Ignore logout errors
      }
    }

    batch(() => {
      setState('token', '');
      setState('refreshToken', null);
      setState('isAuthenticated', false);
      setState('error', null);
      setUser(null);
    });

    localStorage.removeItem(STORAGE_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);
    localStorage.removeItem(TOKEN_EXPIRY_KEY);
    localStorage.removeItem('rustchat_session_id');
    localStorage.removeItem('rustchat_session_iat');
    clearAuthCookie();

    // Clear other stores' session state
    const event = new CustomEvent('rustchat:logout', { detail: { reason } });
    window.dispatchEvent(event);

    if (window.location.pathname !== '/login') {
      window.location.replace('/login');
    }
  } finally {
    isLoggingOut = false;
  }
}

export async function updateStatus(status: {
  status?: string;
  presence?: string;
  text?: string;
  emoji?: string;
  duration?: string;
  duration_minutes?: number;
  dnd_end_time?: number;
}): Promise<void> {
  if (!state.token) return;

  try {
    const payload = { ...status };
    if (payload.presence && !payload.status) {
      payload.status = payload.presence;
    }
    delete payload.presence;

    const response = await fetch('/api/v1/users/me/status', {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${state.token}`,
      },
      body: JSON.stringify(payload),
    });

    if (!response.ok) throw new Error('Failed to update status');

    const data = await response.json();

    setUser(
      produce((draft) => {
        if (!draft) return;
        if (data.status) draft.presence = data.status;
        draft.status_text = data.text;
        draft.status_emoji = data.emoji;
        draft.status_expires_at = data.expires_at;

        // Also update the nested object
        if (data.text !== undefined || data.emoji !== undefined) {
          draft.custom_status = {
            text: data.text,
            emoji: data.emoji,
            expires_at: data.expires_at,
          };
        }
      })
    );
  } catch (e) {
    console.error('Failed to update status', e);
  }
}

// ============================================
// Token Refresh
// ============================================

export async function refreshToken(): Promise<boolean> {
  // If already refreshing, return existing promise
  if (isRefreshing && refreshPromise) {
    return refreshPromise;
  }

  if (!state.refreshToken) {
    await logout('expired');
    return false;
  }

  isRefreshing = true;

  refreshPromise = (async () => {
    try {
      const response = await fetch('/api/v1/auth/refresh', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${state.refreshToken}`,
        },
      });

      if (!response.ok) {
        throw new Error('Token refresh failed');
      }

      const data: LoginResponse = await response.json();

      batch(() => {
        setState('token', data.token);
        if (data.refresh_token) {
          setState('refreshToken', data.refresh_token);
        }
        setState('isAuthenticated', true);
        if (data.user) {
          setUser(data.user);
        }
      });

      setAuthCookie(data.token);

      // Reset timers
      const expiryMs = parseJwtExpiryMs(data.token);
      if (expiryMs) {
        scheduleTokenExpiryLogout(expiryMs);
      }

      // Hide session warning if showing
      setSessionWarning({ show: false, expiresAt: 0, remainingSeconds: 0 });

      return true;
    } catch (err) {
      console.error('Token refresh failed:', err);
      await logout('expired');
      return false;
    } finally {
      isRefreshing = false;
      refreshPromise = null;
    }
  })();

  return refreshPromise;
}

// ============================================
// Session Warning
// ============================================

export function dismissSessionWarning(): void {
  setSessionWarning({ show: false, expiresAt: 0, remainingSeconds: 0 });
}

export function extendSession(): Promise<boolean> {
  return refreshToken();
}

// ============================================
// Auth Policy
// ============================================

const [authPolicy, setAuthPolicy] = createSignal<{
  enable_email_password: boolean;
  enable_sso: boolean;
  require_sso: boolean;
  allow_registration: boolean;
  password_min_length: number;
  password_require_uppercase: boolean;
  password_require_number: boolean;
  password_require_symbol: boolean;
  session_length_hours: number;
  saml_enabled?: boolean;
  oidc_enabled?: boolean;
  oidc_provider_name?: string;
} | null>(null);

export async function getAuthPolicy() {
  try {
    const response = await fetch('/api/v1/auth/policy');
    if (!response.ok) throw new Error('Failed to fetch auth policy');
    const data = await response.json();
    setAuthPolicy(data);
    return data;
  } catch (e) {
    console.error('Failed to fetch auth policy', e);
    return null;
  }
}

// ============================================
// Exports
// ============================================

// ============================================
// Auth Store Object
// ============================================

const authStoreObj = {
  // State getters
  get token() { return state.token; },
  get refreshToken() { return state.refreshToken; },
  get isAuthenticated() { return state.isAuthenticated; },
  get isLoading() { return state.isLoading; },
  get error() { return state.error; },
};

export const authStore = {
  // State
  ...authStoreObj,
  user,
  authPolicy,
  sessionWarning,

  // Actions
  login,
  loginWithToken,
  logout,
  fetchMe,
  updateStatus,
  getAuthPolicy,
  refreshToken,
  dismissSessionWarning,
  extendSession,
};

export default authStore;
