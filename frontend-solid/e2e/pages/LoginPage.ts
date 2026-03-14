import { Page, Locator, expect } from '@playwright/test';

/**
 * Page object for the login page
 */
export class LoginPage {
  readonly page: Page;
  readonly emailInput: Locator;
  readonly passwordInput: Locator;
  readonly loginButton: Locator;
  readonly errorMessage: Locator;
  readonly registerLink: Locator;

  constructor(page: Page) {
    this.page = page;
    this.emailInput = page.getByLabel('Email');
    this.passwordInput = page.getByLabel('Password');
    this.loginButton = page.getByRole('button', { name: /sign in|login/i });
    this.errorMessage = page.getByRole('alert');
    this.registerLink = page.getByRole('link', { name: /register|sign up/i });
  }

  async goto() {
    await this.page.goto('/login');
    await expect(this.page.getByRole('heading', { name: /sign in|login/i })).toBeVisible();
  }

  async login(email: string, password: string) {
    await this.emailInput.fill(email);
    await this.passwordInput.fill(password);
    await this.loginButton.click();
  }

  async expectLoginSuccess() {
    // Should redirect to main app
    await expect(this.page).toHaveURL(/\/channels|\/@me/);
  }

  async expectLoginError(message?: string) {
    await expect(this.errorMessage).toBeVisible();
    if (message) {
      await expect(this.errorMessage).toContainText(message);
    }
  }

  async gotoRegister() {
    await this.registerLink.click();
    await expect(this.page).toHaveURL(/\/register/);
  }
}
