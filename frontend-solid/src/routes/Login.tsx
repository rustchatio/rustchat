// ============================================
// Login Page with Form Validation & SSO
// ============================================

import { createSignal, Show, onMount, createEffect, For } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { authStore, login, loginWithToken, getAuthPolicy } from '../stores/auth';
import { resolveDefaultChannelPath } from '../stores/channels';
import Button from '../components/ui/Button';
import Input from '../components/ui/Input';
import { normalizeAuthRedirectPath, getDefaultAuthRedirectPath } from '../utils/authRedirect';

// ============================================
// Types
// ============================================

interface FormErrors {
  email?: string;
  password?: string;
  general?: string;
}

interface AuthConfig {
  enable_email_password: boolean;
  enable_sso: boolean;
  require_sso: boolean;
  allow_registration: boolean;
  enable_sign_in_with_email?: boolean;
}

interface OAuthProvider {
  id: string;
  provider_key: string;
  provider_type: string;
  display_name: string;
  login_url: string;
}

const AUTH_REDIRECT_STORAGE_KEY = 'rustchat_auth_redirect_to';

// ============================================
// Validation Helpers
// ============================================

function validateForm(email: string, password: string): FormErrors {
  const errors: FormErrors = {};

  if (!email.trim()) {
    errors.email = 'Email or username is required';
  }

  if (!password) {
    errors.password = 'Password is required';
  } else if (password.length < 1) {
    errors.password = 'Password is required';
  }

  return errors;
}

function getRequestedRedirectPath(): string {
  const redirect = new URL(window.location.href).searchParams.get('redirect');
  return normalizeAuthRedirectPath(redirect);
}

function stripOauthQueryParam(path: string): string {
  const url = new URL(path, window.location.origin);
  if (url.searchParams.has('oauth')) {
    url.searchParams.delete('oauth');
  }
  const query = url.searchParams.toString();
  return `${url.pathname}${query ? `?${query}` : ''}${url.hash}`;
}

function buildProviderLoginUrl(loginUrl: string, redirectPath: string): string {
  const url = new URL(loginUrl, window.location.origin);
  url.searchParams.set('redirect_uri', redirectPath);
  return url.toString();
}

// ============================================
// Component
// ============================================

export default function Login() {
  const navigate = useNavigate();
  const redirectPath = getRequestedRedirectPath();
  const postAuthRedirectPath = stripOauthQueryParam(redirectPath);

  // Form state
  const [email, setEmail] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [remember, setRemember] = createSignal(false);
  const [isLoading, setIsLoading] = createSignal(false);
  const [isProcessingCallback, setIsProcessingCallback] = createSignal(false);
  const [isRedirectingAuthenticatedUser, setIsRedirectingAuthenticatedUser] = createSignal(false);
  const [errors, setErrors] = createSignal<FormErrors>({});
  const [authConfig, setAuthConfig] = createSignal<AuthConfig | null>(null);
  const [ssoProviders, setSsoProviders] = createSignal<OAuthProvider[]>([]);

  const resolveAuthenticatedDestination = async (): Promise<string> => {
    if (postAuthRedirectPath && postAuthRedirectPath !== '/') {
      return postAuthRedirectPath;
    }

    const workspacePath = await resolveDefaultChannelPath();
    if (workspacePath) {
      return workspacePath;
    }

    return '/settings/profile';
  };

  const redirectAuthenticatedUser = async () => {
    if (isRedirectingAuthenticatedUser()) return;
    setIsRedirectingAuthenticatedUser(true);

    try {
      const destination = await resolveAuthenticatedDestination();
      navigate(destination, { replace: true });
    } finally {
      setIsRedirectingAuthenticatedUser(false);
    }
  };

  // Redirect if already authenticated - use createEffect for reactivity
  createEffect(() => {
    if (authStore.isAuthenticated) {
      void redirectAuthenticatedUser();
    }
  });

  async function fetchSsoProviders() {
    try {
      const response = await fetch('/api/v1/oauth2/providers');
      if (!response.ok) {
        setSsoProviders([]);
        return;
      }
      const data = (await response.json()) as OAuthProvider[];
      setSsoProviders(Array.isArray(data) ? data : []);
    } catch {
      setSsoProviders([]);
    }
  }

  async function exchangeOauthCode() {
    const response = await fetch('/api/v1/oauth2/exchange', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({}),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => null);
      throw new Error(error?.message || 'Single Sign-On login failed');
    }

    const data = (await response.json()) as { token: string };
    if (!data?.token) {
      throw new Error('Single Sign-On login failed');
    }

    await loginWithToken(data.token);
  }

  onMount(async () => {
    try {
      sessionStorage.setItem(AUTH_REDIRECT_STORAGE_KEY, postAuthRedirectPath || getDefaultAuthRedirectPath());
    } catch {
      // noop
    }

    const config = await getAuthPolicy();
    if (config) {
      setAuthConfig(config);
      if (config.enable_sso) {
        await fetchSsoProviders();
      }
    }

    const urlObj = new URL(window.location.href);
    const oauth = urlObj.searchParams.get('oauth');
    const providerError = urlObj.searchParams.get('error');
    const redirectHasOauth = new URL(redirectPath, window.location.origin).searchParams.get('oauth') === '1';

    if (providerError) {
      setErrors({ general: providerError });
      return;
    }

    if (oauth === '1' || redirectHasOauth) {
      setIsProcessingCallback(true);
      try {
        await exchangeOauthCode();
        try {
          sessionStorage.removeItem(AUTH_REDIRECT_STORAGE_KEY);
        } catch {
          // noop
        }
        await redirectAuthenticatedUser();
      } catch (error) {
        setErrors({
          general: error instanceof Error ? error.message : 'Single Sign-On login failed',
        });
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
    const validationErrors = validateForm(email(), password());
    if (Object.keys(validationErrors).length > 0) {
      setErrors(validationErrors);
      return;
    }

    setIsLoading(true);

    try {
      await login({
        email: email(),
        password: password(),
        remember: remember(),
      });
      try {
        sessionStorage.removeItem(AUTH_REDIRECT_STORAGE_KEY);
      } catch {
        // noop
      }
      await redirectAuthenticatedUser();
    } catch (err) {
      setErrors({
        general: err instanceof Error ? err.message : 'Login failed. Please check your credentials.',
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleProviderLogin = (provider: OAuthProvider) => {
    setIsLoading(true);
    try {
      sessionStorage.setItem(AUTH_REDIRECT_STORAGE_KEY, postAuthRedirectPath || getDefaultAuthRedirectPath());
    } catch {
      // noop
    }
    window.location.href = buildProviderLoginUrl(provider.login_url, postAuthRedirectPath);
  };

  const ssoEnabled = () => authConfig()?.enable_sso ?? false;
  const hasSsoProviders = () => ssoProviders().length > 0;
  const allowRegistration = () => authConfig()?.allow_registration ?? true;
  const emailPasswordEnabled = () => {
    if (authConfig()?.require_sso) return false;
    return (authConfig()?.enable_email_password ?? true) &&
      (authConfig()?.enable_sign_in_with_email ?? true);
  };

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
                    id="email"
                    name="email"
                    type="text"
                    autocomplete="username"
                    label="Email or Username"
                    value={email()}
                    onInput={(e) => {
                      setEmail(e.currentTarget.value);
                      if (errors().email) {
                        setErrors((prev) => ({ ...prev, email: undefined }));
                      }
                    }}
                    placeholder="Enter your email or username"
                    required
                    disabled={isLoading()}
                    error={errors().email}
                    autofocus
                  />
                </div>

                <div>
                  <Input
                    id="password"
                    name="password"
                    type="password"
                    autocomplete="current-password"
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
                disabled={isLoading() || !email() || !password()}
                fullWidth
              >
                {isLoading() ? 'Signing in...' : 'Sign in'}
              </Button>
            </form>
          </Show>

          {/* SSO Section */}
          <Show when={ssoEnabled() && hasSsoProviders()}>
            <div class="relative mt-6">
              <div class="absolute inset-0 flex items-center" aria-hidden="true">
                <div class="w-full border-t border-border-1" />
              </div>
              <div class="relative flex justify-center text-sm">
                <span class="px-2 bg-bg-app text-text-3">Or continue with</span>
              </div>
            </div>

            <div class="mt-6 grid gap-3">
              <For each={ssoProviders()}>
                {(provider) => (
                  <Button
                    type="button"
                    variant="secondary"
                    size="lg"
                    onClick={() => handleProviderLogin(provider)}
                    disabled={isLoading()}
                    fullWidth
                  >
                    Sign in with {provider.display_name}
                  </Button>
                )}
              </For>
            </div>
          </Show>

          <Show when={authConfig()?.require_sso && !hasSsoProviders()}>
            <div class="mt-6 p-4 rounded-lg bg-warning/10 border border-warning/20">
              <p class="text-sm text-warning text-center">
                Single Sign-On is required for this workspace, but no SSO providers are configured.
                Please contact your administrator.
              </p>
            </div>
          </Show>

          {/* Registration Link */}
          <Show when={allowRegistration() && emailPasswordEnabled()}>
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
