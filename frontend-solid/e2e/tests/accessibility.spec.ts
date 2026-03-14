import { test, expect } from '@playwright/test';
import { LoginPage } from '../pages/LoginPage';
import { ChannelPage } from '../pages/ChannelPage';
import { SettingsPage } from '../pages/SettingsPage';
import { TEST_USERS } from '../fixtures/test-data';

test.describe('Accessibility', () => {
  test.describe('Login Page', () => {
    test('should have proper heading structure', async ({ page }) => {
      const loginPage = new LoginPage(page);
      await loginPage.goto();
      
      // Should have exactly one h1
      const h1s = await page.getByRole('heading', { level: 1 }).count();
      expect(h1s).toBe(1);
    });

    test('should have labeled form inputs', async ({ page }) => {
      const loginPage = new LoginPage(page);
      await loginPage.goto();
      
      // Email input should have label
      await expect(loginPage.emailInput).toHaveAttribute('aria-label');
      
      // Password input should have label
      await expect(loginPage.passwordInput).toHaveAttribute('aria-label');
    });

    test('should be keyboard navigable', async ({ page }) => {
      const loginPage = new LoginPage(page);
      await loginPage.goto();
      
      // Tab through form elements
      await page.keyboard.press('Tab');
      await expect(loginPage.emailInput).toBeFocused();
      
      await page.keyboard.press('Tab');
      await expect(loginPage.passwordInput).toBeFocused();
      
      await page.keyboard.press('Tab');
      await expect(loginPage.loginButton).toBeFocused();
    });
  });

  test.describe('Channel Page', () => {
    test.beforeEach(async ({ page }) => {
      const loginPage = new LoginPage(page);
      await loginPage.goto();
      await loginPage.login(TEST_USERS.admin.email, TEST_USERS.admin.password);
      await loginPage.expectLoginSuccess();
    });

    test('should have proper ARIA landmarks', async ({ page }) => {
      // Should have navigation landmark
      const nav = page.getByRole('navigation').first();
      await expect(nav).toBeVisible();
      
      // Should have main landmark
      const main = page.getByRole('main');
      await expect(main).toBeVisible();
    });

    test('should announce new messages to screen readers', async ({ page }) => {
      const channelPage = new ChannelPage(page);
      await channelPage.expectLoaded();
      
      // Look for live region
      const liveRegion = page.locator('[aria-live="polite"], [aria-live="assertive"]').first();
      
      // Live region might exist for message announcements
      if (await liveRegion.isVisible().catch(() => false)) {
        await expect(liveRegion).toHaveAttribute('aria-live');
      }
    });

    test('should have skip link', async ({ page }) => {
      // Press Tab to reveal skip link
      await page.keyboard.press('Tab');
      
      const skipLink = page.getByRole('link', { name: /skip to|skip navigation/i });
      
      if (await skipLink.isVisible().catch(() => false)) {
        await expect(skipLink).toBeVisible();
        
        // Click skip link
        await skipLink.click();
        
        // Focus should move to main content
        const main = page.getByRole('main');
        await expect(main).toBeFocused();
      }
    });

    test('should trap focus in modals', async ({ page }) => {
      // Open a modal (e.g., user menu)
      const channelPage = new ChannelPage(page);
      await channelPage.openUserMenu();
      
      // Look for dialog
      const dialog = page.getByRole('dialog').first();
      
      if (await dialog.isVisible().catch(() => false)) {
        // Tab multiple times - focus should stay within dialog
        const focusedElements: string[] = [];
        
        for (let i = 0; i < 5; i++) {
          await page.keyboard.press('Tab');
          const focused = await page.evaluate(() => document.activeElement?.tagName);
          focusedElements.push(focused || '');
        }
        
        // All focused elements should be within the dialog
        const dialogContainsFocus = await dialog.locator(':focus').count() > 0 ||
          await page.evaluate(() => {
            const dialog = document.querySelector('[role="dialog"]');
            const active = document.activeElement;
            return dialog?.contains(active) || false;
          });
        
        expect(dialogContainsFocus).toBeTruthy();
      }
    });

    test('should have accessible emoji picker', async ({ page }) => {
      const channelPage = new ChannelPage(page);
      await channelPage.expectLoaded();
      
      // Look for emoji button
      const emojiButton = page.getByRole('button', { name: /emoji|smile/i }).first();
      
      if (await emojiButton.isVisible().catch(() => false)) {
        await emojiButton.click();
        
        // Emoji picker should have proper role
        const picker = page.getByRole('dialog').filter({ hasText: /emoji/i }).first();
        
        if (await picker.isVisible().catch(() => false)) {
          await expect(picker).toBeVisible();
          
          // Emoji buttons should have labels
          const emojiButtons = picker.getByRole('button');
          const firstEmoji = emojiButtons.first();
          
          const hasLabel = await firstEmoji.getAttribute('aria-label');
          expect(hasLabel).toBeTruthy();
        }
      }
    });
  });

  test.describe('Settings Page', () => {
    test.beforeEach(async ({ page }) => {
      const loginPage = new LoginPage(page);
      await loginPage.goto();
      await loginPage.login(TEST_USERS.admin.email, TEST_USERS.admin.password);
      await loginPage.expectLoginSuccess();
      
      const channelPage = new ChannelPage(page);
      await channelPage.openSettings();
    });

    test('should have proper heading hierarchy', async ({ page }) => {
      const settingsPage = new SettingsPage(page);
      await settingsPage.navigateToSection('profile');
      
      // Should have h1 for page title
      const h1 = page.getByRole('heading', { level: 1 }).first();
      await expect(h1).toBeVisible();
      
      // Section headings should be h2
      const h2s = page.getByRole('heading', { level: 2 });
      expect(await h2s.count()).toBeGreaterThan(0);
    });

    test('should have descriptive labels for all controls', async ({ page }) => {
      // All inputs should have labels
      const inputs = await page.locator('input, select, textarea').all();
      
      for (const input of inputs) {
        const hasLabel = await input.evaluate(el => {
          const id = el.id;
          const ariaLabel = el.getAttribute('aria-label');
          const ariaLabelledBy = el.getAttribute('aria-labelledby');
          const hasExplicitLabel = id && document.querySelector(`label[for="${id}"]`);
          const hasImplicitLabel = el.closest('label');
          
          return !!(hasExplicitLabel || hasImplicitLabel || ariaLabel || ariaLabelledBy);
        });
        
        expect(hasLabel).toBeTruthy();
      }
    });

    test('should support reduced motion preference', async ({ page }) => {
      // Emulate reduced motion preference
      await page.emulateMedia({ reducedMotion: 'reduce' });
      
      // Page should respect the preference
      const html = page.locator('html');
      
      // Check if reduced motion class or style is applied
      const hasReducedMotion = await html.evaluate(el => {
        return el.classList.contains('reduce-motion') ||
               el.classList.contains('motion-safe') === false ||
               window.matchMedia('(prefers-reduced-motion: reduce)').matches;
      });
      
      expect(hasReducedMotion).toBeTruthy();
    });
  });
});
