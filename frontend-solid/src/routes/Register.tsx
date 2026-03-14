// ============================================
// Register Page
// ============================================

import { createSignal, Show } from 'solid-js';
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

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    setErrors({});

    const validationErrors = validateForm(
      email(),
      username(),
      password(),
      confirmPassword()
    );
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
        }),
      });

      if (!response.ok) {
        throw new Error(await parseErrorMessage(response, 'Registration failed'));
      }

      setIsSuccess(true);
    } catch (err) {
      setErrors({
        general: err instanceof Error ? err.message : 'Registration failed',
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

            <Button
              type="submit"
              variant="primary"
              size="lg"
              loading={isLoading()}
              disabled={isLoading()}
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
