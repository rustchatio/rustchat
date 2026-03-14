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
  readonly editProfileButton: Locator;
  readonly closeButton: Locator;

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
    
    this.firstNameInput = page.locator('label:has-text("First Name") + input');
    this.lastNameInput = page.locator('label:has-text("Last Name") + input');
    this.usernameInput = page.locator('label:has-text("Username") + input');
    this.emailInput = page.locator('label:has-text("Email") + input').first();
    this.editProfileButton = page.getByRole('button', { name: /^Edit$/i }).first();
    this.saveProfileButton = page.getByRole('button', { name: /save changes|save/i }).first();
    this.closeButton = page.getByRole('button', { name: /close settings/i }).first();

    this.currentPasswordInput = page.getByLabel(/current password/i);
    this.newPasswordInput = page.getByLabel(/new password/i);
    this.confirmPasswordInput = page.getByLabel(/confirm.*password/i);
    this.changePasswordButton = page.getByRole('button', { name: /change password|update password/i });

    this.themeButtons = page.getByRole('button', { name: /light|dark|system/i });
    this.densitySelect = page.getByLabel(/density|compact/i);

    this.desktopNotificationsToggle = page
      .locator('h3:has-text("Desktop Notifications")')
      .locator('..')
      .locator('input[type="checkbox"]')
      .first();
    this.soundNotificationsToggle = page
      .locator('h3:has-text("Enable Sounds")')
      .locator('..')
      .locator('input[type="checkbox"]')
      .first();
  }

  async goto() {
    await this.page.goto('/settings');
    await expect(this.page.getByRole('heading', { name: /settings/i })).toBeVisible();
  }

  async navigateToSection(section: 'profile' | 'security' | 'notifications' | 'display' | 'sidebar' | 'sounds' | 'advanced') {
    const trigger = this.page.getByRole('button', { name: new RegExp(`^${section}$`, 'i') }).first();
    await trigger.click();
    await expect(this.page).toHaveURL(new RegExp(`/settings/${section}`));
  }

  async enableProfileEditing() {
    if (await this.editProfileButton.isVisible()) {
      await this.editProfileButton.click();
    }
  }

  async updateProfile(data: { firstName?: string; lastName?: string; username?: string }) {
    await this.enableProfileEditing();
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

  async closeSettings() {
    await this.closeButton.click();
  }

  async expectSaveSuccess() {
    await expect(this.page.getByText(/saved|success|updated/i).first()).toBeVisible();
  }
}
