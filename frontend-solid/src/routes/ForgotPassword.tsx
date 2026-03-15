// ============================================
// Forgot Password Page
// ============================================

import { createSignal, Show } from 'solid-js';
import Button from '../components/ui/Button';
import Input from '../components/ui/Input';

// ============================================
// Types
// ============================================

interface FormErrors {
  email?: string;
  general?: string;
}

// ============================================
// Validation Helpers
// ============================================

function validateEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

function validateForm(email: string): FormErrors {
  const errors: FormErrors = {};

  if (!email.trim()) {
    errors.email = 'Email address is required';
  } else if (!validateEmail(email)) {
    errors.email = 'Please enter a valid email address';
  }

  return errors;
}

// ============================================
// Component
// ============================================

export default function ForgotPassword() {
  // Form state
  const [email, setEmail] = createSignal('');
  const [isLoading, setIsLoading] = createSignal(false);
  const [isSuccess, setIsSuccess] = createSignal(false);
  const [errors, setErrors] = createSignal<FormErrors>({});

  // Handle form submission
  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setErrors({});

    // Validate form
    const validationErrors = validateForm(email());
    if (Object.keys(validationErrors).length > 0) {
      setErrors(validationErrors);
      return;
    }

    setIsLoading(true);

    try {
      const response = await fetch('/api/v1/auth/password/forgot', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email: email() }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || 'Failed to send reset email');
      }

      setIsSuccess(true);
    } catch (err) {
      setErrors({
        general: err instanceof Error ? err.message : 'Failed to send reset email. Please try again.',
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
          <h1 class="text-3xl font-bold text-text-1">Forgot Password</h1>
          <p class="mt-2 text-text-2">
            Enter your email address and we'll send you a link to reset your password
          </p>
        </div>

        {/* Success State */}
        <Show when={isSuccess()}>
          <div class="rounded-lg bg-success/10 border border-success/20 p-6 text-center">
            <div class="mx-auto h-12 w-12 bg-success/20 rounded-full flex items-center justify-center mb-4">
              <svg class="h-6 w-6 text-success" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <h3 class="text-lg font-medium text-text-1 mb-2">Check your email</h3>
            <p class="text-sm text-text-2 mb-4">
              We've sent a password reset link to <strong>{email()}</strong>. 
              Please check your inbox and follow the instructions.
            </p>
            <p class="text-xs text-text-3">
              Didn't receive the email? Check your spam folder or{' '}
              <button
                onClick={() => {
                  setIsSuccess(false);
                  setEmail('');
                }}
                class="text-brand hover:text-brand-hover underline"
              >
                try again
              </button>
            </p>
          </div>

          <div class="text-center">
            <a
              href="/login"
              class="text-sm text-brand hover:text-brand-hover font-medium transition-colors"
            >
              ← Back to sign in
            </a>
          </div>
        </Show>

        {/* Form */}
        <Show when={!isSuccess()}>
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
            <div>
              <Input
                id="email"
                name="email"
                type="email"
                autocomplete="email"
                label="Email Address"
                value={email()}
                onInput={(e) => {
                  setEmail(e.currentTarget.value);
                  if (errors().email) {
                    setErrors((prev) => ({ ...prev, email: undefined }));
                  }
                }}
                placeholder="Enter your email address"
                required
                disabled={isLoading()}
                error={errors().email}
                autofocus
              />
            </div>

            <Button
              type="submit"
              variant="primary"
              size="lg"
              loading={isLoading()}
              disabled={isLoading() || !email()}
              fullWidth
            >
              {isLoading() ? 'Sending...' : 'Send Reset Link'}
            </Button>
          </form>

          <div class="text-center mt-8">
            <a
              href="/login"
              class="text-sm text-brand hover:text-brand-hover font-medium transition-colors"
            >
              ← Back to sign in
            </a>
          </div>
        </Show>
      </div>
    </div>
  );
}
