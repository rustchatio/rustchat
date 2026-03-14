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
    this.emailInput = page.getByLabel(/email/i);
    this.usernameInput = page.getByLabel(/username/i);
    this.passwordInput = page.locator('input#password');
    this.confirmPasswordInput = page.locator('input#confirmPassword');
    this.firstNameInput = page.getByLabel(/first name/i);
    this.lastNameInput = page.getByLabel(/last name/i);
    this.registerButton = page.getByRole('button', { name: /register|sign up|create account/i });
    this.errorMessage = page.locator('div[role="alert"]').filter({ hasText: /.+/ }).first();
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
    await expect(this.page).toHaveURL(/\/channels\/|\/settings\/|\/login/);
  }

  async expectValidationError(field: string) {
    const fieldLocator = this.page.locator('[aria-invalid="true"]');
    const messageLocator = this.page.getByText(new RegExp(field, 'i')).first();
    await expect(messageLocator.or(fieldLocator).or(this.errorMessage).first()).toBeVisible();
  }
}
