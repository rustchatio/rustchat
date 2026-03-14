import { Page, Locator, expect } from '@playwright/test';

/**
 * Page object for the registration page
 */
export class RegisterPage {
  readonly page: Page;
  readonly emailInput: Locator;
  readonly usernameInput: Locator;
  readonly passwordInput: Locator;
  readonly confirmPasswordInput: Locator;
  readonly firstNameInput: Locator;
  readonly lastNameInput: Locator;
  readonly registerButton: Locator;
  readonly errorMessage: Locator;
  readonly loginLink: Locator;

  constructor(page: Page) {
    this.page = page;
    this.emailInput = page.getByLabel('Email');
    this.usernameInput = page.getByLabel('Username');
    this.passwordInput = page.getByLabel('Password');
    this.confirmPasswordInput = page.getByLabel(/confirm password|password confirmation/i);
    this.firstNameInput = page.getByLabel(/first name/i);
    this.lastNameInput = page.getByLabel(/last name/i);
    this.registerButton = page.getByRole('button', { name: /register|sign up|create account/i });
    this.errorMessage = page.getByRole('alert');
    this.loginLink = page.getByRole('link', { name: /sign in|login/i });
  }

  async goto() {
    await this.page.goto('/register');
    await expect(this.page.getByRole('heading', { name: /register|create account/i })).toBeVisible();
  }

  async register(data: {
    email: string;
    username: string;
    password: string;
    confirmPassword: string;
    firstName?: string;
    lastName?: string;
  }) {
    await this.emailInput.fill(data.email);
    await this.usernameInput.fill(data.username);
    await this.passwordInput.fill(data.password);
    await this.confirmPasswordInput.fill(data.confirmPassword);
    if (data.firstName) await this.firstNameInput.fill(data.firstName);
    if (data.lastName) await this.lastNameInput.fill(data.lastName);
    await this.registerButton.click();
  }

  async expectRegistrationSuccess() {
    await expect(this.page).toHaveURL(/\/channels|\/@me|\/login/);
  }

  async expectValidationError(field: string) {
    const fieldLocator = this.page.locator(`[aria-invalid="true"]`).filter({ hasText: new RegExp(field, 'i') });
    await expect(fieldLocator.or(this.errorMessage)).toBeVisible();
  }
}
