import { Page, Locator, expect } from '@playwright/test';

/**
 * Page object for the settings page
 */
export class SettingsPage {
  readonly page: Page;
  readonly sidebar: Locator;
  
  // Profile section
  readonly firstNameInput: Locator;
  readonly lastNameInput: Locator;
  readonly usernameInput: Locator;
  readonly emailInput: Locator;
  readonly saveProfileButton: Locator;

  // Security section
  readonly currentPasswordInput: Locator;
  readonly newPasswordInput: Locator;
  readonly confirmPasswordInput: Locator;
  readonly changePasswordButton: Locator;

  // Display section
  readonly themeButtons: Locator;
  readonly densitySelect: Locator;

  // Notification section
  readonly desktopNotificationsToggle: Locator;
  readonly soundNotificationsToggle: Locator;

  constructor(page: Page) {
    this.page = page;
    this.sidebar = page.locator('[data-testid="settings-sidebar"]').first();
    
    this.firstNameInput = page.getByLabel(/first name/i);
    this.lastNameInput = page.getByLabel(/last name/i);
    this.usernameInput = page.getByLabel(/username/i);
    this.emailInput = page.getByLabel(/email/i).first();
    this.saveProfileButton = page.getByRole('button', { name: /save|update profile/i });

    this.currentPasswordInput = page.getByLabel(/current password/i);
    this.newPasswordInput = page.getByLabel(/new password/i);
    this.confirmPasswordInput = page.getByLabel(/confirm.*password/i);
    this.changePasswordButton = page.getByRole('button', { name: /change password|update password/i });

    this.themeButtons = page.getByRole('button', { name: /light|dark|system/i });
    this.densitySelect = page.getByLabel(/density|compact/i);

    this.desktopNotificationsToggle = page.getByLabel(/desktop notifications/i);
    this.soundNotificationsToggle = page.getByLabel(/sound|play sounds/i);
  }

  async goto() {
    await this.page.goto('/settings');
    await expect(this.page.getByRole('heading', { name: /settings/i })).toBeVisible();
  }

  async navigateToSection(section: 'profile' | 'security' | 'notifications' | 'display' | 'sidebar' | 'sounds' | 'advanced') {
    const link = this.page.getByRole('link', { name: new RegExp(section, 'i') });
    await link.click();
    await expect(this.page).toHaveURL(new RegExp(`/settings/${section}`));
  }

  async updateProfile(data: { firstName?: string; lastName?: string; username?: string }) {
    if (data.firstName) await this.firstNameInput.fill(data.firstName);
    if (data.lastName) await this.lastNameInput.fill(data.lastName);
    if (data.username) await this.usernameInput.fill(data.username);
    await this.saveProfileButton.click();
  }

  async changePassword(currentPassword: string, newPassword: string) {
    await this.currentPasswordInput.fill(currentPassword);
    await this.newPasswordInput.fill(newPassword);
    await this.confirmPasswordInput.fill(newPassword);
    await this.changePasswordButton.click();
  }

  async setTheme(theme: 'light' | 'dark' | 'system') {
    const themeButton = this.page.getByRole('button', { name: new RegExp(`^${theme}$`, 'i') });
    await themeButton.click();
  }

  async expectSaveSuccess() {
    const toast = this.page.getByRole('status').filter({ hasText: /saved|success|updated/i });
    await expect(toast).toBeVisible();
  }
}
