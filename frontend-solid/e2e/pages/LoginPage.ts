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
    this.emailInput = page.getByLabel(/^Email$/i);
    this.passwordInput = page.getByLabel(/^Password$/i);
    this.loginButton = page.getByRole('button', { name: /sign in|login/i });
    this.errorMessage = page.locator('div[role="alert"]').filter({ hasText: /.+/ }).first();
    this.registerLink = page.getByRole('link', { name: /register|sign up|create account/i });
  }

  async goto() {
    await this.page.goto('/login');
    await expect(this.page.getByRole('heading', { name: /welcome to rustchat/i })).toBeVisible();
  }

  async login(email: string, password: string) {
    await this.emailInput.fill(email);
    await this.passwordInput.fill(password);
    await this.loginButton.click();
  }

  async expectLoginSuccess() {
    // Should redirect to main app
    await expect(this.page).toHaveURL(/\/channels\/|\/settings\//);
  }

  async expectLoginError(message?: string) {
    await expect(this.errorMessage).toBeVisible();
    if (message) {
      await expect(this.errorMessage).toContainText(message);
    }
  }

  async gotoRegister() {
    if (await this.registerLink.count()) {
      await this.registerLink.first().click();
    } else {
      await this.page.goto('/register');
    }
    await expect(this.page).toHaveURL(/\/register/);
  }
}
