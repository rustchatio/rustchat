import { test, expect } from '@playwright/test';
import { LoginPage } from '../pages/LoginPage';
import { RegisterPage } from '../pages/RegisterPage';
import { ChannelPage } from '../pages/ChannelPage';
import { TEST_USERS, generateTestId } from '../fixtures/test-data';

test.describe('Authentication', () => {
  test.describe('Login', () => {
    test('should login with valid credentials', async ({ page }) => {
      const loginPage = new LoginPage(page);
      await loginPage.goto();
      await loginPage.login(TEST_USERS.admin.email, TEST_USERS.admin.password);
      await loginPage.expectLoginSuccess();
    });

    test('should show error with invalid password', async ({ page }) => {
      const loginPage = new LoginPage(page);
      await loginPage.goto();
      await loginPage.login(TEST_USERS.admin.email, 'wrongpassword');
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
      const loginPage = new LoginPage(page);
      await loginPage.goto();
      await loginPage.login(TEST_USERS.admin.email, TEST_USERS.admin.password);
      await loginPage.expectLoginSuccess();

      await page.getByRole('button', { name: /user menu/i }).click();
      await expect(page.getByRole('button', { name: /admin console/i })).toBeVisible();
    });
  });

  test.describe('Registration', () => {
    test('should register new user with valid data', async ({ page }) => {
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
      
      await registerPage.expectRegistrationSuccess();
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
      // Login first
      const loginPage = new LoginPage(page);
      await loginPage.goto();
      await loginPage.login(TEST_USERS.admin.email, TEST_USERS.admin.password);
      await loginPage.expectLoginSuccess();

      // Then logout
      const channelPage = new ChannelPage(page);
      await channelPage.logout();
      
      // Verify redirected to login
      await expect(page).toHaveURL(/\/login/);
    });

    test('should clear session on logout', async ({ page }) => {
      // Login
      const loginPage = new LoginPage(page);
      await loginPage.goto();
      await loginPage.login(TEST_USERS.admin.email, TEST_USERS.admin.password);
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
      // Try to access protected page
      await page.goto('/settings/profile');
      
      // Should be on login page with redirect param
      await expect(page).toHaveURL(/\/login/);
      
      // Login
      const loginPage = new LoginPage(page);
      await loginPage.login(TEST_USERS.admin.email, TEST_USERS.admin.password);
      
      // Should redirect to originally requested page
      await expect(page).toHaveURL(/\/settings/);
    });
  });
});
