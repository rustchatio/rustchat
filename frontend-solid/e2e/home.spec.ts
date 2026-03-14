import { test, expect } from '@playwright/test';

test.describe('Home Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should display the welcome message', async ({ page }) => {
    await expect(page.getByRole('heading', { name: 'RustChat Solid.js' })).toBeVisible();
  });

  test('should have working theme selector', async ({ page }) => {
    // Check that theme buttons are visible
    await expect(page.getByRole('button', { name: 'Light' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Dark' })).toBeVisible();

    // Click dark theme
    await page.getByRole('button', { name: 'Dark' }).click();

    // Verify theme changed
    const html = page.locator('html');
    await expect(html).toHaveAttribute('data-theme', 'dark');
  });

  test('should display all button variants', async ({ page }) => {
    await expect(page.getByRole('button', { name: 'Primary' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Secondary' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Ghost' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Danger' })).toBeVisible();
  });

  test('should open and close modal', async ({ page }) => {
    // Open modal
    await page.getByRole('button', { name: 'Open Modal' }).click();

    // Verify modal is visible
    await expect(page.getByRole('dialog')).toBeVisible();
    await expect(page.getByRole('heading', { name: 'Example Modal' })).toBeVisible();

    // Close modal using close button
    await page.getByRole('button', { name: 'Close modal' }).click();

    // Verify modal is closed
    await expect(page.getByRole('dialog')).not.toBeVisible();
  });

  test('should close modal with escape key', async ({ page }) => {
    // Open modal
    await page.getByRole('button', { name: 'Open Modal' }).click();
    await expect(page.getByRole('dialog')).toBeVisible();

    // Press escape
    await page.keyboard.press('Escape');

    // Verify modal is closed
    await expect(page.getByRole('dialog')).not.toBeVisible();
  });

  test('should have no accessibility violations', async ({ page }) => {
    // Basic accessibility check - verify landmarks exist
    await expect(page.locator('main')).toBeVisible();

    // Verify all interactive elements have accessible names
    const buttons = page.getByRole('button');
    for (const button of await buttons.all()) {
      await expect(button)
        .toHaveAttribute('aria-label')
        .catch(() => {
          // Buttons with text content don't need aria-label
          return button.textContent().then(text => expect(text?.trim()).not.toBe(''));
        });
    }
  });
});

test.describe('Navigation', () => {
  test('should navigate to about page', async ({ page }) => {
    await page.goto('/');
    await page.getByRole('link', { name: /about/i }).click();

    await expect(page.getByRole('heading', { name: 'About RustChat Solid.js' })).toBeVisible();
  });

  test('should show 404 page for unknown routes', async ({ page }) => {
    await page.goto('/nonexistent');

    await expect(page.getByRole('heading', { name: '404' })).toBeVisible();
    await expect(page.getByText(/page not found/i)).toBeVisible();
  });
});
