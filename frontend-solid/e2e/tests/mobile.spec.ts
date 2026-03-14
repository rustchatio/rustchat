import { test, expect } from '@playwright/test';
import { LoginPage } from '../pages/LoginPage';
import { ChannelPage } from '../pages/ChannelPage';
import { TEST_USERS } from '../fixtures/test-data';

test.describe('Mobile Responsive', () => {
  test.use({ viewport: { width: 375, height: 667 } }); // iPhone SE size

  test.beforeEach(async ({ page }) => {
    const loginPage = new LoginPage(page);
    await loginPage.goto();
    await loginPage.login(TEST_USERS.admin.email, TEST_USERS.admin.password);
    await loginPage.expectLoginSuccess();
  });

  test('should display mobile layout', async ({ page }) => {
    const channelPage = new ChannelPage(page);
    await channelPage.expectLoaded();
    
    // Sidebar should be collapsed or hidden on mobile
    const sidebar = page.locator('[data-testid="sidebar"], nav').first();
    const isVisible = await sidebar.isVisible().catch(() => false);
    
    // Either sidebar is hidden or there's a menu toggle
    if (!isVisible) {
      const menuToggle = page.getByRole('button', { name: /menu|toggle/i }).first();
      await expect(menuToggle).toBeVisible();
    }
  });

  test('should open sidebar with menu toggle', async ({ page }) => {
    const menuToggle = page.getByRole('button', { name: /menu|toggle/i }).first();
    
    if (await menuToggle.isVisible().catch(() => false)) {
      await menuToggle.click();
      
      // Sidebar should now be visible
      const sidebar = page.locator('[data-testid="sidebar"], nav, aside').first();
      await expect(sidebar).toBeVisible();
    }
  });

  test('should have touch-friendly message input', async ({ page }) => {
    const channelPage = new ChannelPage(page);
    await channelPage.expectLoaded();
    
    // Message input should be visible and usable
    await expect(channelPage.messageInput).toBeVisible();
    
    // Input should be at least 44px tall (touch target size)
    const box = await channelPage.messageInput.boundingBox();
    if (box) {
      expect(box.height).toBeGreaterThanOrEqual(44);
    }
  });

  test('should have accessible touch targets', async ({ page }) => {
    const channelPage = new ChannelPage(page);
    await channelPage.expectLoaded();
    
    // Get all buttons
    const buttons = await page.getByRole('button').all();
    
    for (const button of buttons.slice(0, 5)) { // Check first 5 buttons
      const box = await button.boundingBox().catch(() => null);
      if (box) {
        // Touch targets should be at least 44x44px
        expect(box.width).toBeGreaterThanOrEqual(44);
        expect(box.height).toBeGreaterThanOrEqual(44);
      }
    }
  });
});

test.describe('Tablet Responsive', () => {
  test.use({ viewport: { width: 768, height: 1024 } }); // iPad size

  test.beforeEach(async ({ page }) => {
    const loginPage = new LoginPage(page);
    await loginPage.goto();
    await loginPage.login(TEST_USERS.admin.email, TEST_USERS.admin.password);
    await loginPage.expectLoginSuccess();
  });

  test('should display tablet layout', async ({ page }) => {
    const channelPage = new ChannelPage(page);
    await channelPage.expectLoaded();
    
    // Sidebar should be visible on tablet
    const sidebar = page.locator('[data-testid="sidebar"], nav').first();
    await expect(sidebar).toBeVisible();
  });

  test('should show member list on wider screens', async ({ page }) => {
    const channelPage = new ChannelPage(page);
    await channelPage.expectLoaded();
    
    // Member list might be visible on tablet
    const memberList = page.locator('[data-testid="member-list"]').first();
    const isVisible = await memberList.isVisible().catch(() => false);
    
    // Either visible or there's a toggle to show it
    if (!isVisible) {
      const memberToggle = page.getByRole('button', { name: /members|info/i }).first();
      expect(await memberToggle.isVisible().catch(() => false)).toBeTruthy();
    }
  });
});
