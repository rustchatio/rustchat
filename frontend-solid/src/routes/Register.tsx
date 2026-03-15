// ============================================
// Register Page
// ============================================

import { createEffect, createSignal, onCleanup, onMount, Show } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import Button from '../components/ui/Button';
import Input from '../components/ui/Input';

interface FormErrors {
  email?: string;
  username?: string;
  password?: string;
  confirmPassword?: string;
  general?: string;
}

interface PublicAuthConfig {
  registration_enabled?: boolean;
  turnstile?: {
    enabled?: boolean;
    site_key?: string;
  };
}

type TurnstileWidgetId = string | number;

const TURNSTILE_SCRIPT_ID = 'rustchat-turnstile-script';
const TURNSTILE_SCRIPT_SRC = 'https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit';

declare global {
  interface Window {
    turnstile?: {
      render: (
        container: string | HTMLElement,
        options: {
          sitekey: string;
          callback: (token: string) => void;
          'expired-callback'?: () => void;
          'error-callback'?: () => void;
          theme?: 'light' | 'dark' | 'auto';
        }
      ) => TurnstileWidgetId;
      reset: (widgetId?: TurnstileWidgetId) => void;
      remove?: (widgetId: TurnstileWidgetId) => void;
    };
  }
}

function ensureTurnstileScript(): Promise<void> {
  if (typeof window === 'undefined') return Promise.resolve();
  if (window.turnstile) return Promise.resolve();

  return new Promise((resolve, reject) => {
    const timeoutAt = Date.now() + 8000;
    const existing = document.getElementById(TURNSTILE_SCRIPT_ID) as HTMLScriptElement | null;
    if (existing) {
      const waitForReady = () => {
        if (window.turnstile) {
          resolve();
          return;
        }
        if (Date.now() >= timeoutAt) {
          reject(new Error('Cloudflare verification widget did not load in time.'));
          return;
        }
        window.setTimeout(waitForReady, 100);
      };
      waitForReady();
      return;
    }

    const script = document.createElement('script');
    script.id = TURNSTILE_SCRIPT_ID;
    script.src = TURNSTILE_SCRIPT_SRC;
    script.async = true;
    script.defer = true;
    script.onload = () => resolve();
    script.onerror = () => reject(new Error('Failed to load Cloudflare verification widget.'));
    document.head.appendChild(script);

    const waitForReady = () => {
      if (window.turnstile) {
        resolve();
        return;
      }
      if (Date.now() >= timeoutAt) {
        reject(new Error('Cloudflare verification widget did not load in time.'));
        return;
      }
      window.setTimeout(waitForReady, 100);
    };
    waitForReady();
  });
}

function validateEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

function validateUsername(username: string): boolean {
  return /^[a-zA-Z0-9._-]{3,}$/.test(username);
}

function validateForm(
  email: string,
  username: string,
  password: string,
  confirmPassword: string
): FormErrors {
  const errors: FormErrors = {};

  if (!email.trim()) {
    errors.email = 'Email is required';
  } else if (!validateEmail(email)) {
    errors.email = 'Please enter a valid email address';
  }

  if (!username.trim()) {
    errors.username = 'Username is required';
  } else if (!validateUsername(username)) {
    errors.username = 'Username must be at least 3 characters (letters, numbers, dots, dashes, underscores)';
  }

  if (!password) {
    errors.password = 'Password is required';
  } else if (password.length < 8) {
    errors.password = 'Password must be at least 8 characters';
  }

  if (!confirmPassword) {
    errors.confirmPassword = 'Please confirm your password';
  } else if (password !== confirmPassword) {
    errors.confirmPassword = 'Passwords do not match';
  }

  return errors;
}

export default function Register() {
  const navigate = useNavigate();

  const [email, setEmail] = createSignal('');
  const [username, setUsername] = createSignal('');
  const [firstName, setFirstName] = createSignal('');
  const [lastName, setLastName] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [confirmPassword, setConfirmPassword] = createSignal('');
  const [isLoading, setIsLoading] = createSignal(false);
  const [isSuccess, setIsSuccess] = createSignal(false);
  const [errors, setErrors] = createSignal<FormErrors>({});
  const [authConfig, setAuthConfig] = createSignal<PublicAuthConfig | null>(null);
  const [isLoadingConfig, setIsLoadingConfig] = createSignal(true);
  const [turnstileToken, setTurnstileToken] = createSignal('');
  const [turnstileError, setTurnstileError] = createSignal<string | null>(null);
  const [honeypot, setHoneypot] = createSignal('');

  const registrationEnabled = () => authConfig()?.registration_enabled !== false;
  const isTurnstileEnabled = () => authConfig()?.turnstile?.enabled === true;
  const turnstileSiteKey = () => authConfig()?.turnstile?.site_key?.trim() || '';

  let turnstileContainer: HTMLDivElement | undefined;
  let turnstileWidgetId: TurnstileWidgetId | null = null;
  let renderedSiteKey: string | null = null;

  const resetTurnstile = () => {
    setTurnstileToken('');
    if (turnstileWidgetId !== null && window.turnstile?.reset) {
      window.turnstile.reset(turnstileWidgetId);
    }
  };

  const renderTurnstile = async () => {
    if (!isTurnstileEnabled()) return;
    if (!turnstileContainer) return;

    const siteKey = turnstileSiteKey();
    if (!siteKey) {
      setTurnstileError('Human verification is enabled, but no site key is configured.');
      return;
    }

    if (turnstileWidgetId !== null && renderedSiteKey === siteKey) return;

    try {
      await ensureTurnstileScript();
      if (!window.turnstile) {
        throw new Error('Cloudflare verification is unavailable right now.');
      }

      if (turnstileWidgetId !== null && window.turnstile.remove) {
        window.turnstile.remove(turnstileWidgetId);
        turnstileWidgetId = null;
      }

      turnstileWidgetId = window.turnstile.render(turnstileContainer, {
        sitekey: siteKey,
        theme: 'auto',
        callback: (token: string) => {
          setTurnstileToken(token);
          setTurnstileError(null);
        },
        'expired-callback': () => {
          setTurnstileToken('');
          setTurnstileError('Verification expired. Please verify again.');
        },
        'error-callback': () => {
          setTurnstileToken('');
          setTurnstileError('Verification failed. Please retry.');
        },
      });
      renderedSiteKey = siteKey;
      setTurnstileError(null);
    } catch (err) {
      setTurnstileError(
        err instanceof Error ? err.message : 'Failed to initialize human verification.'
      );
    }
  };

  onMount(async () => {
    try {
      const response = await fetch('/api/v1/auth/config');
      if (response.ok) {
        const data = (await response.json()) as PublicAuthConfig;
        setAuthConfig(data);
      }
    } catch {
      // Keep defaults if auth config is not reachable.
    } finally {
      setIsLoadingConfig(false);
    }
  });

  createEffect(() => {
    if (!isLoadingConfig() && isTurnstileEnabled()) {
      void renderTurnstile();
    }
  });

  onCleanup(() => {
    if (turnstileWidgetId !== null && window.turnstile?.remove) {
      window.turnstile.remove(turnstileWidgetId);
      turnstileWidgetId = null;
    }
  });

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setErrors({});

    if (!registrationEnabled()) {
      setErrors({ general: 'Registration is currently disabled.' });
      return;
    }

    const validationErrors = validateForm(
      email(),
      username(),
      password(),
      confirmPassword()
    );

    if (isTurnstileEnabled() && !turnstileToken().trim()) {
      validationErrors.general = 'Please complete human verification.';
    }

    if (Object.keys(validationErrors).length > 0) {
      setErrors(validationErrors);
      return;
    }

    const displayName = [firstName().trim(), lastName().trim()].filter(Boolean).join(' ');

    setIsLoading(true);
    try {
      const parseErrorMessage = async (response: Response, fallback: string): Promise<string> => {
        const contentType = response.headers.get('content-type') || '';
        if (!contentType.includes('application/json')) return fallback;
        try {
          const payload = (await response.json()) as { message?: string };
          return payload.message || fallback;
        } catch {
          return fallback;
        }
      };

      const response = await fetch('/api/v1/auth/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          email: email(),
          username: username(),
          password: password(),
          display_name: displayName || null,
          'cf-turnstile-response': turnstileToken() || undefined,
          website: honeypot() || undefined,
        }),
      });

      if (!response.ok) {
        throw new Error(await parseErrorMessage(response, 'Registration failed'));
      }

      setIsSuccess(true);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Registration failed';
      if (isTurnstileEnabled() && /verification|turnstile|captcha/i.test(message)) {
        resetTurnstile();
      }
      setErrors({
        general: message,
      });
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div class="min-h-screen flex items-center justify-center bg-bg-app px-4 py-12">
      <div class="w-full max-w-md space-y-8">
        <div class="text-center">
          <h1 class="text-3xl font-bold text-text-1">Create Account</h1>
          <p class="mt-2 text-text-2">Join your RustChat workspace</p>
        </div>

        <Show when={isSuccess()}>
          <div class="rounded-lg bg-success/10 border border-success/20 p-6 text-center">
            <h3 class="text-lg font-medium text-text-1 mb-2">Registration successful</h3>
            <p class="text-sm text-text-2 mb-4">
              Your account was created. You can sign in now.
            </p>
            <Button variant="primary" onClick={() => navigate('/login')} fullWidth>
              Go to Sign In
            </Button>
          </div>
        </Show>

        <Show when={!isSuccess()}>
          <Show when={errors().general}>
            <div class="rounded-lg bg-danger/10 border border-danger/20 p-4" role="alert">
              <p class="text-sm text-danger">{errors().general}</p>
            </div>
          </Show>

          <form onSubmit={handleSubmit} class="space-y-4">
            <Show when={!registrationEnabled() && !isLoadingConfig()}>
              <div class="rounded-lg bg-warning/10 border border-warning/25 p-3 text-sm text-warning">
                Registration is disabled by server policy.
              </div>
            </Show>

            <Input
              id="email"
              type="email"
              label="Email"
              value={email()}
              onInput={(e) => {
                setEmail(e.currentTarget.value);
                if (errors().email) setErrors((prev) => ({ ...prev, email: undefined }));
              }}
              required
              error={errors().email}
              disabled={isLoading()}
            />

            <Input
              id="username"
              type="text"
              label="Username"
              value={username()}
              onInput={(e) => {
                setUsername(e.currentTarget.value);
                if (errors().username) setErrors((prev) => ({ ...prev, username: undefined }));
              }}
              required
              error={errors().username}
              disabled={isLoading()}
            />

            <Input
              id="firstName"
              type="text"
              label="First Name"
              value={firstName()}
              onInput={(e) => setFirstName(e.currentTarget.value)}
              disabled={isLoading()}
            />

            <Input
              id="lastName"
              type="text"
              label="Last Name"
              value={lastName()}
              onInput={(e) => setLastName(e.currentTarget.value)}
              disabled={isLoading()}
            />

            <Input
              id="password"
              type="password"
              label="Password"
              value={password()}
              onInput={(e) => {
                setPassword(e.currentTarget.value);
                if (errors().password) setErrors((prev) => ({ ...prev, password: undefined }));
              }}
              required
              error={errors().password}
              disabled={isLoading()}
            />

            <Input
              id="confirmPassword"
              type="password"
              label="Confirm Password"
              value={confirmPassword()}
              onInput={(e) => {
                setConfirmPassword(e.currentTarget.value);
                if (errors().confirmPassword) {
                  setErrors((prev) => ({ ...prev, confirmPassword: undefined }));
                }
              }}
              required
              error={errors().confirmPassword}
              disabled={isLoading()}
            />

            <div class="sr-only" aria-hidden="true">
              <label for="website">Website</label>
              <input
                id="website"
                name="website"
                type="text"
                autocomplete="off"
                tabIndex={-1}
                value={honeypot()}
                onInput={(event) => setHoneypot(event.currentTarget.value)}
              />
            </div>

            <Show when={isTurnstileEnabled()}>
              <div class="space-y-2">
                <p class="text-xs text-text-3">Please complete human verification to continue.</p>
                <div
                  ref={(element) => {
                    turnstileContainer = element;
                  }}
                  class="min-h-[70px]"
                />
                <Show when={turnstileError()}>
                  <p class="text-xs text-danger">{turnstileError()}</p>
                </Show>
              </div>
            </Show>

            <Button
              type="submit"
              variant="primary"
              size="lg"
              loading={isLoading()}
              disabled={
                isLoading() ||
                isLoadingConfig() ||
                !registrationEnabled() ||
                (isTurnstileEnabled() && !turnstileToken())
              }
              fullWidth
            >
              {isLoading() ? 'Creating account...' : 'Create Account'}
            </Button>
          </form>

          <p class="text-center text-sm text-text-3">
            Already have an account?{' '}
            <a href="/login" class="text-brand hover:text-brand-hover font-medium transition-colors">
              Sign in
            </a>
          </p>
        </Show>
      </div>
    </div>
  );
}
