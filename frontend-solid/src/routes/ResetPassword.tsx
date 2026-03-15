// ============================================
// Reset Password Page
// ============================================

import { createSignal, Show, onMount } from 'solid-js';
import { useNavigate, useSearchParams } from '@solidjs/router';
import Button from '../components/ui/Button';
import Input from '../components/ui/Input';

// ============================================
// Types
// ============================================

interface FormErrors {
  password?: string;
  confirmPassword?: string;
  general?: string;
}

interface PasswordStrength {
  score: number; // 0-4
  label: string;
  color: string;
}

// ============================================
// Password Strength Calculator
// ============================================

function calculatePasswordStrength(password: string): PasswordStrength {
  if (!password) {
    return { score: 0, label: 'Enter password', color: 'bg-border-1' };
  }

  let score = 0;
  const checks = {
    length: password.length >= 8,
    lowercase: /[a-z]/.test(password),
    uppercase: /[A-Z]/.test(password),
    number: /\d/.test(password),
    special: /[!@#$%^&*(),.?":{}|<>]/.test(password),
  };

  const passedChecks = Object.values(checks).filter(Boolean).length;

  if (password.length < 6) {
    score = 0;
  } else {
    score = Math.min(4, Math.floor((passedChecks / 5) * 4) + (password.length >= 12 ? 1 : 0));
  }

  const labels = ['Very Weak', 'Weak', 'Fair', 'Good', 'Strong'];
  const colors = [
    'bg-danger',
    'bg-danger',
    'bg-warning',
    'bg-success',
    'bg-success',
  ];

  return {
    score,
    label: labels[score],
    color: colors[score],
  };
}

function getPasswordRequirements(password: string): {
  length: boolean;
  lowercase: boolean;
  uppercase: boolean;
  number: boolean;
  special: boolean;
} {
  return {
    length: password.length >= 8,
    lowercase: /[a-z]/.test(password),
    uppercase: /[A-Z]/.test(password),
    number: /\d/.test(password),
    special: /[!@#$%^&*(),.?":{}|<>]/.test(password),
  };
}

// ============================================
// Validation Helpers
// ============================================

function validateForm(password: string, confirmPassword: string): FormErrors {
  const errors: FormErrors = {};
  const requirements = getPasswordRequirements(password);

  if (!password) {
    errors.password = 'Password is required';
  } else if (password.length < 8) {
    errors.password = 'Password must be at least 8 characters';
  } else if (!requirements.lowercase || !requirements.uppercase) {
    errors.password = 'Password must contain both uppercase and lowercase letters';
  } else if (!requirements.number) {
    errors.password = 'Password must contain at least one number';
  } else if (!requirements.special) {
    errors.password = 'Password must contain at least one special character';
  }

  if (!confirmPassword) {
    errors.confirmPassword = 'Please confirm your password';
  } else if (password !== confirmPassword) {
    errors.confirmPassword = 'Passwords do not match';
  }

  return errors;
}

// ============================================
// Component
// ============================================

export default function ResetPassword() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  // Form state
  const [password, setPassword] = createSignal('');
  const [confirmPassword, setConfirmPassword] = createSignal('');
  const [isLoading, setIsLoading] = createSignal(false);
  const [isSuccess, setIsSuccess] = createSignal(false);
  const [errors, setErrors] = createSignal<FormErrors>({});
  const [showPassword, setShowPassword] = createSignal(false);
  const [isValidToken, setIsValidToken] = createSignal<boolean | null>(null);

  const token = () => searchParams.token;

  // Validate token on mount
  onMount(async () => {
    const resetToken = token();
    if (!resetToken) {
      setIsValidToken(false);
      setErrors({ general: 'Invalid or missing reset token' });
      return;
    }

    try {
      // Validate token with backend
      const response = await fetch('/api/v1/auth/password/validate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token: resetToken }),
      });
      if (!response.ok) {
        setIsValidToken(false);
        setErrors({ general: 'This reset link has expired or is invalid' });
        return;
      }

      const data = (await response.json()) as { valid?: boolean };
      setIsValidToken(Boolean(data.valid));
      if (!data.valid) {
        setErrors({ general: 'This reset link has expired or is invalid' });
      }
    } catch {
      setIsValidToken(false);
      setErrors({ general: 'Failed to validate reset token' });
    }
  });

  // Calculate password strength
  const strength = () => calculatePasswordStrength(password());
  const requirements = () => getPasswordRequirements(password());

  // Handle form submission
  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setErrors({});

    // Validate form
    const validationErrors = validateForm(password(), confirmPassword());
    if (Object.keys(validationErrors).length > 0) {
      setErrors(validationErrors);
      return;
    }

    setIsLoading(true);

    try {
      const response = await fetch('/api/v1/auth/password/reset', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          token: token(),
          new_password: password(),
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || 'Failed to reset password');
      }

      setIsSuccess(true);
    } catch (err) {
      setErrors({
        general: err instanceof Error ? err.message : 'Failed to reset password. Please try again.',
      });
    } finally {
      setIsLoading(false);
    }
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
          <h1 class="text-3xl font-bold text-text-1">Reset Password</h1>
          <p class="mt-2 text-text-2">Create a new password for your account</p>
        </div>

        {/* Invalid Token State */}
        <Show when={isValidToken() === false}>
          <div class="rounded-lg bg-danger/10 border border-danger/20 p-6 text-center">
            <div class="mx-auto h-12 w-12 bg-danger/20 rounded-full flex items-center justify-center mb-4">
              <svg class="h-6 w-6 text-danger" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <h3 class="text-lg font-medium text-text-1 mb-2">Invalid Reset Link</h3>
            <p class="text-sm text-text-2 mb-4">{errors().general}</p>
            <a
              href="/forgot-password"
              class="inline-flex items-center justify-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-brand hover:bg-brand-hover transition-colors"
            >
              Request New Link
            </a>
          </div>
        </Show>

        {/* Success State */}
        <Show when={isSuccess()}>
          <div class="rounded-lg bg-success/10 border border-success/20 p-6 text-center">
            <div class="mx-auto h-12 w-12 bg-success/20 rounded-full flex items-center justify-center mb-4">
              <svg class="h-6 w-6 text-success" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <h3 class="text-lg font-medium text-text-1 mb-2">Password Reset Successful</h3>
            <p class="text-sm text-text-2 mb-4">
              Your password has been successfully reset. You can now sign in with your new password.
            </p>
            <Button
              variant="primary"
              onClick={() => navigate('/login')}
              fullWidth
            >
              Sign In
            </Button>
          </div>
        </Show>

        {/* Form */}
        <Show when={isValidToken() === true && !isSuccess()}>
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

          <form onSubmit={handleSubmit} class="mt-8 space-y-6">
            <div class="space-y-4">
              {/* New Password */}
              <div>
                <div class="relative">
                  <Input
                    id="password"
                    name="newPassword"
                    type={showPassword() ? 'text' : 'password'}
                    autocomplete="new-password"
                    label="New Password"
                    value={password()}
                    onInput={(e) => {
                      setPassword(e.currentTarget.value);
                      if (errors().password) {
                        setErrors((prev) => ({ ...prev, password: undefined }));
                      }
                    }}
                    placeholder="Enter your new password"
                    required
                    disabled={isLoading()}
                    error={errors().password}
                    helperText={password() ? `Strength: ${strength().label}` : undefined}
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassword(!showPassword())}
                    class="absolute right-3 top-[34px] text-text-3 hover:text-text-2 transition-colors"
                    aria-label={showPassword() ? 'Hide password' : 'Show password'}
                  >
                    <Show
                      when={showPassword()}
                      fallback={
                        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                        </svg>
                      }
                    >
                      <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                      </svg>
                    </Show>
                  </button>
                </div>

                {/* Password Strength Meter */}
                <Show when={password()}>
                  <div class="mt-2">
                    <div class="h-2 w-full bg-bg-surface-2 rounded-full overflow-hidden">
                      <div
                        class={`h-full transition-all duration-300 ${strength().color}`}
                        style={{ width: `${(strength().score / 4) * 100}%` }}
                      />
                    </div>
                  </div>
                </Show>

                {/* Password Requirements */}
                <div class="mt-3 space-y-1">
                  <p class="text-xs font-medium text-text-2">Password requirements:</p>
                  <ul class="text-xs space-y-1">
                    <li class={`flex items-center gap-2 ${requirements().length ? 'text-success' : 'text-text-3'}`}>
                      <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <Show
                          when={requirements().length}
                          fallback={<path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />}
                        >
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M5 13l4 4L19 7" />
                        </Show>
                      </svg>
                      At least 8 characters
                    </li>
                    <li class={`flex items-center gap-2 ${requirements().lowercase ? 'text-success' : 'text-text-3'}`}>
                      <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <Show
                          when={requirements().lowercase}
                          fallback={<path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />}
                        >
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M5 13l4 4L19 7" />
                        </Show>
                      </svg>
                      Lowercase letter (a-z)
                    </li>
                    <li class={`flex items-center gap-2 ${requirements().uppercase ? 'text-success' : 'text-text-3'}`}>
                      <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <Show
                          when={requirements().uppercase}
                          fallback={<path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />}
                        >
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M5 13l4 4L19 7" />
                        </Show>
                      </svg>
                      Uppercase letter (A-Z)
                    </li>
                    <li class={`flex items-center gap-2 ${requirements().number ? 'text-success' : 'text-text-3'}`}>
                      <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <Show
                          when={requirements().number}
                          fallback={<path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />}
                        >
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M5 13l4 4L19 7" />
                        </Show>
                      </svg>
                      Number (0-9)
                    </li>
                    <li class={`flex items-center gap-2 ${requirements().special ? 'text-success' : 'text-text-3'}`}>
                      <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <Show
                          when={requirements().special}
                          fallback={<path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />}
                        >
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M5 13l4 4L19 7" />
                        </Show>
                      </svg>
                      Special character (!@#$...)
                    </li>
                  </ul>
                </div>
              </div>

              {/* Confirm Password */}
              <div>
                <Input
                  id="confirmPassword"
                  name="confirmPassword"
                  type={showPassword() ? 'text' : 'password'}
                  autocomplete="new-password"
                  label="Confirm Password"
                  value={confirmPassword()}
                  onInput={(e) => {
                    setConfirmPassword(e.currentTarget.value);
                    if (errors().confirmPassword) {
                      setErrors((prev) => ({ ...prev, confirmPassword: undefined }));
                    }
                  }}
                  placeholder="Confirm your new password"
                  required
                  disabled={isLoading()}
                  error={errors().confirmPassword}
                />
              </div>
            </div>

            <Button
              type="submit"
              variant="primary"
              size="lg"
              loading={isLoading()}
              disabled={isLoading() || !password() || !confirmPassword()}
              fullWidth
            >
              {isLoading() ? 'Resetting...' : 'Reset Password'}
            </Button>
          </form>
        </Show>
      </div>
    </div>
  );
}
