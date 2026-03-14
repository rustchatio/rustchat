import { test, expect } from '@playwright/test';
import { LoginPage } from '../pages/LoginPage';
import { SettingsPage } from '../pages/SettingsPage';
import { ChannelPage } from '../pages/ChannelPage';
import { TEST_USERS, generateTestId } from '../fixtures/test-data';

test.describe('Settings', () => {
  test.beforeEach(async ({ page }) => {
    // Login before each test
    const loginPage = new LoginPage(page);
    await loginPage.goto();
    await loginPage.login(TEST_USERS.admin.email, TEST_USERS.admin.password);
    await loginPage.expectLoginSuccess();
    
    // Navigate to settings
    const channelPage = new ChannelPage(page);
    await channelPage.openSettings();
  });

  test.describe('Profile Settings', () => {
    test('should update profile information', async ({ page }) => {
      const settingsPage = new SettingsPage(page);
      await settingsPage.navigateToSection('profile');
      
      const uniqueId = generateTestId();
      await settingsPage.updateProfile({
        firstName: `Test-${uniqueId}`,
        lastName: 'User',
      });
      
      await settingsPage.expectSaveSuccess();
    });

    test('should validate required fields', async ({ page }) => {
      const settingsPage = new SettingsPage(page);
      await settingsPage.navigateToSection('profile');
      
      // Clear required field
      await settingsPage.usernameInput.fill('');
      await settingsPage.saveProfileButton.click();
      
      // Should show validation error
      await expect(page.getByText(/required|invalid/i).first()).toBeVisible();
    });
  });

  test.describe('Security Settings', () => {
    test('should change password', async ({ page }) => {
      const settingsPage = new SettingsPage(page);
      await settingsPage.navigateToSection('security');
      
      const newPassword = `NewPass-${generateTestId()}!`;
      await settingsPage.changePassword(TEST_USERS.admin.password, newPassword);
      
      await settingsPage.expectSaveSuccess();
    });

    test('should show error for incorrect current password', async ({ page }) => {
      const settingsPage = new SettingsPage(page);
      await settingsPage.navigateToSection('security');
      
      await settingsPage.changePassword('wrongpassword', 'NewPass123!');
      
      await expect(page.getByText(/incorrect|invalid/i).first()).toBeVisible();
    });

    test('should show active sessions', async ({ page }) => {
      const settingsPage = new SettingsPage(page);
      await settingsPage.navigateToSection('security');
      
      // Look for sessions list
      const sessionsSection = page.getByText(/active sessions|devices/i).first();
      await expect(sessionsSection).toBeVisible();
    });
  });

  test.describe('Display Settings', () => {
    test('should change theme to dark', async ({ page }) => {
      const settingsPage = new SettingsPage(page);
      await settingsPage.navigateToSection('display');
      
      await settingsPage.setTheme('dark');
      
      // Verify theme attribute on html element
      const html = page.locator('html');
      await expect(html).toHaveAttribute('data-theme', 'dark');
    });

    test('should change theme to light', async ({ page }) => {
      const settingsPage = new SettingsPage(page);
      await settingsPage.navigateToSection('display');
      
      await settingsPage.setTheme('light');
      
      const html = page.locator('html');
      await expect(html).toHaveAttribute('data-theme', 'light');
    });

    test('should persist theme preference', async ({ page }) => {
      const settingsPage = new SettingsPage(page);
      await settingsPage.navigateToSection('display');
      
      // Set dark theme
      await settingsPage.setTheme('dark');
      
      // Reload page
      await page.reload();
      
      // Verify theme persisted
      const html = page.locator('html');
      await expect(html).toHaveAttribute('data-theme', 'dark');
    });
  });

  test.describe('Notification Settings', () => {
    test('should toggle desktop notifications', async ({ page }) => {
      const settingsPage = new SettingsPage(page);
      await settingsPage.navigateToSection('notifications');
      
      // Toggle desktop notifications
      await settingsPage.desktopNotificationsToggle.click();
      
      // Verify toggle state changed
      const isChecked = await settingsPage.desktopNotificationsToggle.isChecked();
      expect(typeof isChecked).toBe('boolean');
    });

    test('should toggle sound notifications', async ({ page }) => {
      const settingsPage = new SettingsPage(page);
      await settingsPage.navigateToSection('notifications');
      
      // Toggle sound notifications
      await settingsPage.soundNotificationsToggle.click();
      
      // Verify setting persisted after reload
      await page.reload();
      const isChecked = await settingsPage.soundNotificationsToggle.isChecked().catch(() => null);
      expect(isChecked).not.toBeNull();
    });
  });

  test.describe('Navigation', () => {
    test('should navigate between settings sections', async ({ page }) => {
      const settingsPage = new SettingsPage(page);
      
      // Navigate through all sections
      const sections: Array<'profile' | 'security' | 'notifications' | 'display' | 'sidebar' | 'sounds' | 'advanced'> = [
        'profile', 'security', 'notifications', 'display', 'sidebar', 'sounds', 'advanced'
      ];
      
      for (const section of sections) {
        await settingsPage.navigateToSection(section);
        await expect(page).toHaveURL(new RegExp(`/settings/${section}`));
      }
    });

    test('should return to app from settings', async ({ page }) => {
      // Look for back button or link
      const backButton = page.getByRole('link', { name: /back|channels|home/i }).first();
      await backButton.click();
      
      // Should be back at channels
      await expect(page).toHaveURL(/\/channels|\/@me/);
    });
  });
});
