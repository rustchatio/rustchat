// ============================================
// Login Callback Page (OIDC/SAML)
// ============================================

import { createSignal, Show, onMount } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { loginWithToken } from '../stores/auth';
import { handleOIDCCallback, getOIDCRedirectUrl, clearAllOIDCStorage } from '../auth/oidc';
import { handleSAMLCallback, clearAllSAMLStorage } from '../auth/saml';
import { normalizeAuthRedirectPath, getDefaultAuthRedirectPath } from '../utils/authRedirect';

type CallbackStatus = 'processing' | 'success' | 'error';
const AUTH_REDIRECT_STORAGE_KEY = 'rustchat_auth_redirect_to';

export default function LoginCallback() {
  const navigate = useNavigate();

  const [status, setStatus] = createSignal<CallbackStatus>('processing');
  const [errorMessage, setErrorMessage] = createSignal('');
  const [redirectPath, setRedirectPath] = createSignal<string>(getDefaultAuthRedirectPath());

  onMount(async () => {
    const url = window.location.href;
    const urlObj = new URL(url);

    // Determine if this is OIDC or SAML callback
    const code = urlObj.searchParams.get('code');
    const state = urlObj.searchParams.get('state');
    const error = urlObj.searchParams.get('error');
    const errorDescription = urlObj.searchParams.get('error_description');
    const samlResponse = urlObj.searchParams.get('SAMLResponse');
    const relayState = urlObj.searchParams.get('RelayState');

    // Handle OAuth error first
    if (error) {
      setStatus('error');
      setErrorMessage(errorDescription || error);
      return;
    }

    // Resolve redirect path from query + auth storage + OIDC storage
    const redirectFromQuery = urlObj.searchParams.get('redirect');
    if (redirectFromQuery) {
      setRedirectPath(normalizeAuthRedirectPath(redirectFromQuery));
    }

    const storedAuthRedirect = (() => {
      try {
        return sessionStorage.getItem(AUTH_REDIRECT_STORAGE_KEY);
      } catch {
        return null;
      }
    })();
    if (storedAuthRedirect) {
      setRedirectPath(normalizeAuthRedirectPath(storedAuthRedirect));
    }

    const storedRedirect = getOIDCRedirectUrl();
    if (storedRedirect) {
      setRedirectPath(normalizeAuthRedirectPath(storedRedirect));
    }

    try {
      if (code && state) {
        // OIDC callback
        await handleOIDC(code, state);
      } else if (samlResponse || relayState) {
        // SAML callback
        await handleSAML(samlResponse, relayState);
      } else {
        // Unknown callback
        setStatus('error');
        setErrorMessage('Invalid callback parameters');
      }
    } catch (err) {
      setStatus('error');
      setErrorMessage(err instanceof Error ? err.message : 'Authentication failed');
    }
  });

  async function handleOIDC(_code: string, _state: string) {
    const result = await handleOIDCCallback(window.location.href);

    if (result.success && result.tokens) {
      await loginWithToken(result.tokens.access_token, result.tokens.refresh_token);
      setStatus('success');

      // Small delay for UX
      setTimeout(() => {
        clearAllOIDCStorage();
        try {
          sessionStorage.removeItem(AUTH_REDIRECT_STORAGE_KEY);
        } catch {
          // noop
        }
        navigate(normalizeAuthRedirectPath(redirectPath()), { replace: true });
      }, 500);
    } else {
      setStatus('error');
      setErrorMessage(result.error || 'Authentication failed');
      clearAllOIDCStorage();
    }
  }

  async function handleSAML(samlResponse: string | null, relayState: string | null) {
    const result = await handleSAMLCallback(samlResponse || undefined, relayState || undefined);

    if (result.success && result.data) {
      await loginWithToken(result.data.token);
      setStatus('success');

      // Small delay for UX
      setTimeout(() => {
        clearAllSAMLStorage();
        try {
          sessionStorage.removeItem(AUTH_REDIRECT_STORAGE_KEY);
        } catch {
          // noop
        }
        navigate(normalizeAuthRedirectPath(redirectPath()), { replace: true });
      }, 500);
    } else {
      setStatus('error');
      setErrorMessage(result.error || 'SAML authentication failed');
      clearAllSAMLStorage();
    }
  }

  return (
    <div class="min-h-screen flex items-center justify-center bg-bg-app px-4">
      <div class="w-full max-w-md text-center space-y-6">
        {/* Processing State */}
        <Show when={status() === 'processing'}>
          <div class="inline-block w-12 h-12 border-3 border-brand border-t-transparent rounded-full animate-spin mb-4" />
          <h2 class="text-2xl font-semibold text-text-1">Completing Sign In...</h2>
          <p class="text-text-2">Please wait while we verify your credentials</p>
        </Show>

        {/* Success State */}
        <Show when={status() === 'success'}>
          <div class="mx-auto h-16 w-16 bg-success/20 rounded-full flex items-center justify-center mb-4">
            <svg class="h-8 w-8 text-success" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M5 13l4 4L19 7" />
            </svg>
          </div>
          <h2 class="text-2xl font-semibold text-text-1">Sign In Successful!</h2>
          <p class="text-text-2">Redirecting you to the app...</p>
        </Show>

        {/* Error State */}
        <Show when={status() === 'error'}>
          <div class="mx-auto h-16 w-16 bg-danger/20 rounded-full flex items-center justify-center mb-4">
            <svg class="h-8 w-8 text-danger" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </div>
          <h2 class="text-2xl font-semibold text-text-1">Sign In Failed</h2>
          <p class="text-text-2">{errorMessage() || 'An error occurred during authentication'}</p>

          <div class="flex flex-col gap-3 mt-6">
            <a
              href="/login"
              class="inline-flex items-center justify-center px-4 py-2 border border-transparent text-sm font-medium rounded-lg text-white bg-brand hover:bg-brand-hover transition-colors"
            >
              Try Again
            </a>
            <a
              href="/"
              class="inline-flex items-center justify-center px-4 py-2 border border-border-1 text-sm font-medium rounded-lg text-text-2 bg-transparent hover:bg-bg-surface-2 transition-colors"
            >
              Go to Home
            </a>
          </div>
        </Show>
      </div>
    </div>
  );
}
