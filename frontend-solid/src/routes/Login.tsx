// ============================================
// Login Page with Form Validation & SSO
// ============================================

import { createSignal, Show, onMount } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { authStore, login, loginWithToken } from '../stores/auth';
import { oidc, handleOIDCCallback } from '../auth/oidc';
import { saml, handleSAMLCallback } from '../auth/saml';
import { getAuthPolicy } from '../stores/auth';
import Button from '../components/ui/Button';
import Input from '../components/ui/Input';

// ============================================
// Types
// ============================================

interface FormErrors {
  username?: string;
  password?: string;
  general?: string;
}

interface AuthConfig {
  enable_email_password: boolean;
  enable_sso: boolean;
  require_sso: boolean;
  allow_registration: boolean;
  saml_enabled?: boolean;
  oidc_enabled?: boolean;
  oidc_provider_name?: string;
}

// ============================================
// Validation Helpers
// ============================================

function validateEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

function validateUsername(username: string): boolean {
  // Allow alphanumeric, dots, dashes, underscores
  const usernameRegex = /^[a-zA-Z0-9._-]+$/;
  return usernameRegex.test(username) && username.length >= 3;
}

function validateForm(username: string, password: string): FormErrors {
  const errors: FormErrors = {};

  if (!username.trim()) {
    errors.username = 'Username or email is required';
  } else if (username.includes('@')) {
    if (!validateEmail(username)) {
      errors.username = 'Please enter a valid email address';
    }
  } else if (!validateUsername(username)) {
    errors.username = 'Username must be at least 3 characters and contain only letters, numbers, dots, dashes, or underscores';
  }

  if (!password) {
    errors.password = 'Password is required';
  } else if (password.length < 1) {
    errors.password = 'Password is required';
  }

  return errors;
}

// ============================================
// Component
// ============================================

export default function Login() {
  const navigate = useNavigate();

  // Form state
  const [username, setUsername] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [remember, setRemember] = createSignal(false);
  const [isLoading, setIsLoading] = createSignal(false);
  const [isProcessingCallback, setIsProcessingCallback] = createSignal(false);
  const [errors, setErrors] = createSignal<FormErrors>({});
  const [authConfig, setAuthConfig] = createSignal<AuthConfig | null>(null);

  // Redirect if already authenticated
  if (authStore.isAuthenticated) {
    navigate('/channels/general', { replace: true });
    return null;
  }

  // Check for OIDC/SAML callbacks on mount
  onMount(async () => {
    // Load auth config
    const config = await getAuthPolicy();
    if (config) {
      setAuthConfig(config);
    }

    // Check if this is an OIDC callback
    const url = window.location.href;
    const urlObj = new URL(url);
    const code = urlObj.searchParams.get('code');
    const state = urlObj.searchParams.get('state');
    const samlResponse = urlObj.searchParams.get('SAMLResponse');

    if (code && state) {
      // OIDC callback
      setIsProcessingCallback(true);
      try {
        const result = await handleOIDCCallback(url);
        if (result.success && result.tokens) {
          await loginWithToken(result.tokens.access_token, result.tokens.refresh_token);
          navigate('/channels/general', { replace: true });
        } else {
          setErrors({ general: result.error || 'Authentication failed' });
        }
      } catch (err) {
        setErrors({ general: 'Authentication failed' });
      } finally {
        setIsProcessingCallback(false);
      }
    } else if (samlResponse || urlObj.pathname.includes('/saml/callback')) {
      // SAML callback - handle via form post
      setIsProcessingCallback(true);
      try {
        const result = await handleSAMLCallback();
        if (result.success && result.data) {
          await loginWithToken(result.data.token);
          navigate('/channels/general', { replace: true });
        } else {
          setErrors({ general: result.error || 'SAML authentication failed' });
        }
      } catch (err) {
        setErrors({ general: 'SAML authentication failed' });
      } finally {
        setIsProcessingCallback(false);
      }
    }
  });

  // Handle form submission
  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setErrors({});

    // Validate form
    const validationErrors = validateForm(username(), password());
    if (Object.keys(validationErrors).length > 0) {
      setErrors(validationErrors);
      return;
    }

    setIsLoading(true);

    try {
      await login({
        username: username(),
        password: password(),
        remember: remember(),
      });
      navigate('/channels/general', { replace: true });
    } catch (err) {
      setErrors({
        general: err instanceof Error ? err.message : 'Login failed. Please check your credentials.',
      });
    } finally {
      setIsLoading(false);
    }
  };

  // Handle OIDC login
  const handleOIDCLogin = async () => {
    setIsLoading(true);
    try {
      // Fetch OIDC config from backend
      const response = await fetch('/api/v1/auth/oidc/config');
      if (!response.ok) {
        throw new Error('Failed to fetch OIDC configuration');
      }
      const config = await response.json();

      await oidc.initiateOIDCLogin({
        authorizationEndpoint: config.authorization_endpoint,
        clientId: config.client_id,
        redirectUri: `${window.location.origin}/login/callback`,
        scope: config.scope || 'openid email profile',
      });
    } catch (err) {
      setErrors({
        general: err instanceof Error ? err.message : 'Failed to initiate SSO login',
      });
      setIsLoading(false);
    }
  };

  // Handle SAML login
  const handleSAMLLogin = async () => {
    setIsLoading(true);
    try {
      const config = await saml.getSAMLConfig();
      if (!config || !config.enabled) {
        throw new Error('SAML is not enabled');
      }

      await saml.initiateSAMLLogin({
        idpUrl: config.idp_url,
      });
    } catch (err) {
      setErrors({
        general: err instanceof Error ? err.message : 'Failed to initiate SAML login',
      });
      setIsLoading(false);
    }
  };

  const ssoEnabled = () => authConfig()?.enable_sso ?? false;
  const oidcEnabled = () => authConfig()?.oidc_enabled ?? false;
  const samlEnabled = () => authConfig()?.saml_enabled ?? false;
  const emailPasswordEnabled = () => {
    if (authConfig()?.require_sso) return false;
    return authConfig()?.enable_email_password ?? true;
  };
  const oidcProviderName = () => authConfig()?.oidc_provider_name || 'SSO';

  return (
    <div class="min-h-screen flex items-center justify-center bg-bg-app px-4 py-12">
      <div class="w-full max-w-md space-y-8">
        {/* Logo */}
        <div class="text-center">
          <div class="mx-auto h-16 w-16 bg-brand rounded-2xl flex items-center justify-center mb-4 shadow-lg">
            <svg
              class="h-10 w-10 text-white"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width={2}
                d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
              />
            </svg>
          </div>
          <h1 class="text-3xl font-bold text-text-1">Welcome to RustChat</h1>
          <p class="mt-2 text-text-2">Sign in to your workspace</p>
        </div>

        {/* Loading state for callback processing */}
        <Show when={isProcessingCallback()}>
          <div class="text-center py-8">
            <div class="inline-block w-8 h-8 border-2 border-brand border-t-transparent rounded-full animate-spin mb-4" />
            <p class="text-text-2">Completing authentication...</p>
          </div>
        </Show>

        {/* Main content */}
        <Show when={!isProcessingCallback()}>
          {/* Error message */}
          <Show when={errors().general}>
            <div class="rounded-lg bg-danger/10 border border-danger/20 p-4" role="alert">
              <div class="flex items-start gap-3">
                <svg class="h-5 w-5 text-danger mt-0.5 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <p class="text-sm text-danger">{errors().general}</p>
              </div>
            </div>
          </Show>

          {/* Email/Password Form */}
          <Show when={emailPasswordEnabled()}>
            <form onSubmit={handleSubmit} class="mt-8 space-y-6">
              <div class="space-y-4">
                <div>
                  <Input
                    id="username"
                    type="text"
                    label="Username or Email"
                    value={username()}
                    onInput={(e) => {
                      setUsername(e.currentTarget.value);
                      if (errors().username) {
                        setErrors((prev) => ({ ...prev, username: undefined }));
                      }
                    }}
                    placeholder="Enter your username or email"
                    required
                    disabled={isLoading()}
                    error={errors().username}
                    autofocus
                  />
                </div>

                <div>
                  <Input
                    id="password"
                    type="password"
                    label="Password"
                    value={password()}
                    onInput={(e) => {
                      setPassword(e.currentTarget.value);
                      if (errors().password) {
                        setErrors((prev) => ({ ...prev, password: undefined }));
                      }
                    }}
                    placeholder="Enter your password"
                    required
                    disabled={isLoading()}
                    error={errors().password}
                  />
                </div>
              </div>

              <div class="flex items-center justify-between">
                <label class="flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    checked={remember()}
                    onChange={(e) => setRemember(e.currentTarget.checked)}
                    disabled={isLoading()}
                    class="h-4 w-4 rounded border-border-1 text-brand focus:ring-brand/50 cursor-pointer"
                  />
                  <span class="ml-2 text-sm text-text-2">Remember me</span>
                </label>

                <a
                  href="/forgot-password"
                  class="text-sm text-brand hover:text-brand-hover transition-colors"
                >
                  Forgot password?
                </a>
              </div>

              <Button
                type="submit"
                variant="primary"
                size="lg"
                loading={isLoading()}
                disabled={isLoading() || !username() || !password()}
                fullWidth
              >
                {isLoading() ? 'Signing in...' : 'Sign in'}
              </Button>
            </form>
          </Show>

          {/* SSO Section */}
          <Show when={ssoEnabled() && (oidcEnabled() || samlEnabled())}>
            <div class="relative mt-6">
              <div class="absolute inset-0 flex items-center" aria-hidden="true">
                <div class="w-full border-t border-border-1" />
              </div>
              <div class="relative flex justify-center text-sm">
                <span class="px-2 bg-bg-app text-text-3">Or continue with</span>
              </div>
            </div>

            <div class="mt-6 grid gap-3">
              {/* OIDC Button */}
              <Show when={oidcEnabled()}>
                <Button
                  type="button"
                  variant="secondary"
                  size="lg"
                  onClick={handleOIDCLogin}
                  disabled={isLoading()}
                  fullWidth
                >
                  <svg class="h-5 w-5 mr-2" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 0C5.373 0 0 5.373 0 12s5.373 12 12 12 12-5.373 12-12S18.627 0 12 0zm0 2c5.514 0 10 4.486 10 10s-4.486 10-10 10S2 17.514 2 12 6.486 2 12 2zm-1 5v4H7v2h4v4h2v-4h4v-2h-4V7h-2z" />
                  </svg>
                  Sign in with {oidcProviderName()}
                </Button>
              </Show>

              {/* SAML Button */}
              <Show when={samlEnabled()}>
                <Button
                  type="button"
                  variant="secondary"
                  size="lg"
                  onClick={handleSAMLLogin}
                  disabled={isLoading()}
                  fullWidth
                >
                  <svg class="h-5 w-5 mr-2" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z" />
                  </svg>
                  Sign in with SAML
                </Button>
              </Show>
            </div>
          </Show>

          {/* Registration Link */}
          <Show when={authConfig()?.allow_registration && emailPasswordEnabled()}>
            <p class="mt-8 text-center text-sm text-text-3">
              Don't have an account?{' '}
              <a href="/register" class="text-brand hover:text-brand-hover font-medium transition-colors">
                Sign up
              </a>
            </p>
          </Show>

          {/* SSO Required Message */}
          <Show when={authConfig()?.require_sso && !ssoEnabled()}>
            <div class="mt-6 p-4 rounded-lg bg-warning/10 border border-warning/20">
              <p class="text-sm text-warning text-center">
                Single Sign-On is required for this workspace.
                Please contact your administrator.
              </p>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  );
}
