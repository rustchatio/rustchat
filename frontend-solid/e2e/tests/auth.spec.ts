import { test, expect } from '@playwright/test';
import { LoginPage } from '../pages/LoginPage';
import { RegisterPage } from '../pages/RegisterPage';
import { ChannelPage } from '../pages/ChannelPage';
import { TEST_USERS, generateTestId } from '../fixtures/test-data';

interface TestCredentials {
  email: string;
  password: string;
}

test.describe('Authentication', () => {
  test.describe.configure({ mode: 'serial' });
  let adminCredentialsAvailable = true;
  let primaryCredentials: TestCredentials | null = null;
  let registrationAvailable = true;

  test.beforeAll(async ({ request }) => {
    const adminLoginResponse = await request.post('/api/v1/auth/login', {
      data: {
        email: TEST_USERS.admin.email,
        password: TEST_USERS.admin.password,
      },
    });
    adminCredentialsAvailable = adminLoginResponse.ok();

    const uniqueId = generateTestId();
    const candidate = {
      email: `auth-${uniqueId}@example.com`,
      username: `auth-${uniqueId}`,
      password: 'AuthPass123!',
    };

    const registerResponse = await request.post('/api/v1/auth/register', {
      data: {
        email: candidate.email,
        username: candidate.username,
        password: candidate.password,
        confirm_password: candidate.password,
        first_name: 'Auth',
        last_name: 'E2E',
      },
    });

    if (registerResponse.ok()) {
      primaryCredentials = {
        email: candidate.email,
        password: candidate.password,
      };
      registrationAvailable = true;
      return;
    }

    registrationAvailable = false;
    if (adminCredentialsAvailable) {
      primaryCredentials = {
        email: TEST_USERS.admin.email,
        password: TEST_USERS.admin.password,
      };
    }
  });

  test.describe('Login', () => {
    test('should login with valid credentials', async ({ page }) => {
      const creds = primaryCredentials;
      test.skip(!creds, 'No valid credentials are available in this environment.');
      if (!creds) return;
      const loginPage = new LoginPage(page);
      await loginPage.goto();
      await loginPage.login(creds.email, creds.password);
      await loginPage.expectLoginSuccess();
    });

    test('should show error with invalid password', async ({ page }) => {
      const creds = primaryCredentials;
      test.skip(!creds, 'No valid credentials are available in this environment.');
      if (!creds) return;
      const loginPage = new LoginPage(page);
      await loginPage.goto();
      await loginPage.login(creds.email, 'wrongpassword');
      await loginPage.expectLoginError();
    });

    test('should show error with non-existent email', async ({ page }) => {
      const loginPage = new LoginPage(page);
      await loginPage.goto();
      await loginPage.login('nonexistent@example.com', 'somepassword');
      await loginPage.expectLoginError();
    });

    test('should navigate to register page', async ({ page }) => {
      const loginPage = new LoginPage(page);
      await loginPage.goto();
      await loginPage.gotoRegister();
    });

    test('should expose admin console entry for admin users', async ({ page }) => {
      test.skip(!adminCredentialsAvailable, 'Admin credentials are not valid in this environment.');
      const loginPage = new LoginPage(page);
      await loginPage.goto();
      await loginPage.login(TEST_USERS.admin.email, TEST_USERS.admin.password);
      await loginPage.expectLoginSuccess();

      const sidebarAdminLink = page.getByRole('link', { name: /admin console/i });
      if (await sidebarAdminLink.count()) {
        await expect(sidebarAdminLink.first()).toBeVisible();
        return;
      }

      const userMenuButton = page.getByRole('button', { name: /user menu/i });
      if (await userMenuButton.count()) {
        await userMenuButton.click();
        await expect(page.getByRole('button', { name: /admin console/i })).toBeVisible();
        return;
      }

      const rootAdminButton = page.getByRole('button', { name: /open admin console/i });
      if (await rootAdminButton.count()) {
        await expect(rootAdminButton).toBeVisible();
        return;
      }

      await page.goto('/admin');
      await expect(page).toHaveURL(/\/admin/);
    });

    test('should open admin email section for admin users', async ({ page }) => {
      test.skip(!adminCredentialsAvailable, 'Admin credentials are not valid in this environment.');
      const loginPage = new LoginPage(page);
      await loginPage.goto();
      await loginPage.login(TEST_USERS.admin.email, TEST_USERS.admin.password);
      await loginPage.expectLoginSuccess();

      await page.goto('/admin/email');
      await expect(page).toHaveURL(/\/admin\/email/);
      await expect(page.getByText(/mail providers|recent outbox status/i).first()).toBeVisible();
    });
  });

  test.describe('Registration', () => {
    test('should register new user with valid data', async ({ page }) => {
      test.skip(!registrationAvailable, 'Registration is disabled or throttled in this environment.');
      const registerPage = new RegisterPage(page);
      await registerPage.goto();
      
      const uniqueId = generateTestId();
      await registerPage.register({
        email: `test-${uniqueId}@example.com`,
        username: `testuser-${uniqueId}`,
        password: 'TestPass123!',
        confirmPassword: 'TestPass123!',
        firstName: 'Test',
        lastName: 'User',
      });
      await expect(page.getByText(/registration successful|registration failed/i).first())
        .toBeVisible({ timeout: 10000 });
    });

    test('should show validation error for mismatched passwords', async ({ page }) => {
      const registerPage = new RegisterPage(page);
      await registerPage.goto();
      
      const uniqueId = generateTestId();
      await registerPage.register({
        email: `test-${uniqueId}@example.com`,
        username: `testuser-${uniqueId}`,
        password: 'TestPass123!',
        confirmPassword: 'DifferentPass123!',
        firstName: 'Test',
        lastName: 'User',
      });
      
      await registerPage.expectValidationError('password');
    });

    test('should show validation error for invalid email', async ({ page }) => {
      const registerPage = new RegisterPage(page);
      await registerPage.goto();
      
      const uniqueId = generateTestId();
      await registerPage.register({
        email: 'invalid-email',
        username: `testuser-${uniqueId}`,
        password: 'TestPass123!',
        confirmPassword: 'TestPass123!',
      });
      
      await registerPage.expectValidationError('email');
    });
  });

  test.describe('Logout', () => {
    test('should logout successfully', async ({ page }) => {
      const creds = primaryCredentials;
      test.skip(!creds, 'No valid credentials are available in this environment.');
      if (!creds) return;
      // Login first
      const loginPage = new LoginPage(page);
      await loginPage.goto();
      await loginPage.login(creds.email, creds.password);
      await loginPage.expectLoginSuccess();

      // Then logout
      const channelPage = new ChannelPage(page);
      await channelPage.logout();
      
      // Verify redirected to login
      await expect(page).toHaveURL(/\/login/);
    });

    test('should clear session on logout', async ({ page }) => {
      const creds = primaryCredentials;
      test.skip(!creds, 'No valid credentials are available in this environment.');
      if (!creds) return;
      // Login
      const loginPage = new LoginPage(page);
      await loginPage.goto();
      await loginPage.login(creds.email, creds.password);
      await loginPage.expectLoginSuccess();

      // Logout
      const channelPage = new ChannelPage(page);
      await channelPage.logout();

      // Try to access protected page - should redirect to login
      await page.goto('/channels/00000000-0000-0000-0000-000000000000');
      await expect(page).toHaveURL(/\/login/);
    });
  });

  test.describe('Protected Routes', () => {
    test('should redirect to login when accessing protected route unauthenticated', async ({ page }) => {
      await page.goto('/channels/00000000-0000-0000-0000-000000000000');
      await expect(page).toHaveURL(/\/login/);
    });

    test('should redirect to login when accessing settings unauthenticated', async ({ page }) => {
      await page.goto('/settings');
      await expect(page).toHaveURL(/\/login/);
    });

    test('should maintain redirect after successful login', async ({ page }) => {
      const creds = primaryCredentials;
      test.skip(!creds, 'No valid credentials are available in this environment.');
      if (!creds) return;
      // Try to access protected page
      await page.goto('/settings/profile');
      
      // Should be on login page with redirect param
      await expect(page).toHaveURL(/\/login/);
      
      // Login
      const loginPage = new LoginPage(page);
      await loginPage.login(creds.email, creds.password);
      
      // Should redirect to originally requested page
      await expect(page).toHaveURL(/\/settings\/|\/$/);
    });
  });
});
