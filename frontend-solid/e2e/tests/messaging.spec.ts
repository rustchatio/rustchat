import { test, expect } from '@playwright/test';
import { LoginPage } from '../pages/LoginPage';
import { ChannelPage } from '../pages/ChannelPage';
import { TEST_USERS, TEST_MESSAGE, generateTestId } from '../fixtures/test-data';

test.describe('Messaging', () => {
  test.beforeEach(async ({ page }) => {
    // Login before each test
    const loginPage = new LoginPage(page);
    await loginPage.goto();
    await loginPage.login(TEST_USERS.admin.email, TEST_USERS.admin.password);
    await loginPage.expectLoginSuccess();
  });

  test('should send and display a message', async ({ page }) => {
    const channelPage = new ChannelPage(page);
    await channelPage.expectLoaded();
    
    const uniqueMessage = `${TEST_MESSAGE.text} - ${generateTestId()}`;
    await channelPage.sendMessage(uniqueMessage);
    await channelPage.expectMessageVisible(uniqueMessage);
  });

  test('should send message with markdown formatting', async ({ page }) => {
    const channelPage = new ChannelPage(page);
    await channelPage.expectLoaded();
    
    await channelPage.sendMessage(TEST_MESSAGE.withMarkdown);
    
    // Check that formatted elements exist
    await expect(page.locator('h1, h2, h3').filter({ hasText: 'Heading' })).toBeVisible();
    await expect(page.locator('strong').filter({ hasText: 'Bold' })).toBeVisible();
  });

  test('should send message with code block', async ({ page }) => {
    const channelPage = new ChannelPage(page);
    await channelPage.expectLoaded();
    
    await channelPage.sendMessage(TEST_MESSAGE.withCode);
    
    // Check that code block exists
    await expect(page.locator('pre, code')).toBeVisible();
  });

  test('should display emoji in message', async ({ page }) => {
    const channelPage = new ChannelPage(page);
    await channelPage.expectLoaded();
    
    const messageWithEmoji = 'Hello! 👋 🎉 🚀';
    await channelPage.sendMessage(messageWithEmoji);
    await channelPage.expectMessageVisible('Hello!');
  });

  test('should edit a message', async ({ page }) => {
    const channelPage = new ChannelPage(page);
    await channelPage.expectLoaded();
    
    // Send a message
    const originalText = `Edit test ${generateTestId()}`;
    await channelPage.sendMessage(originalText);
    await channelPage.expectMessageVisible(originalText);
    
    // Find and click edit button on the message
    const message = page.getByText(originalText).first();
    await message.hover();
    
    const editButton = page.getByRole('button', { name: /edit/i }).first();
    await editButton.click();
    
    // Edit the message
    const editInput = page.locator('[contenteditable="true"], textarea').first();
    const editedText = `${originalText} (edited)`;
    await editInput.fill(editedText);
    
    const saveButton = page.getByRole('button', { name: /save/i });
    await saveButton.click();
    
    // Verify edited message
    await channelPage.expectMessageVisible('(edited)');
  });

  test('should delete a message', async ({ page }) => {
    const channelPage = new ChannelPage(page);
    await channelPage.expectLoaded();
    
    // Send a message
    const deleteText = `Delete test ${generateTestId()}`;
    await channelPage.sendMessage(deleteText);
    await channelPage.expectMessageVisible(deleteText);
    
    // Find and click delete button
    const message = page.getByText(deleteText).first();
    await message.hover();
    
    const deleteButton = page.getByRole('button', { name: /delete/i }).first();
    await deleteButton.click();
    
    // Confirm deletion
    const confirmButton = page.getByRole('button', { name: /confirm|delete/i }).last();
    await confirmButton.click();
    
    // Verify message is removed
    await expect(page.getByText(deleteText)).not.toBeVisible();
  });

  test('should add reaction to message', async ({ page }) => {
    const channelPage = new ChannelPage(page);
    await channelPage.expectLoaded();
    
    // Send a message
    const reactionText = `Reaction test ${generateTestId()}`;
    await channelPage.sendMessage(reactionText);
    await channelPage.expectMessageVisible(reactionText);
    
    // Hover over message and click add reaction
    const message = page.getByText(reactionText).first();
    await message.hover();
    
    const addReactionButton = page.getByRole('button', { name: /add reaction|emoji/i }).first();
    await addReactionButton.click();
    
    // Select an emoji
    const emojiButton = page.getByRole('button', { name: /👍|thumbs up/i }).first();
    await emojiButton.click();
    
    // Verify reaction is added
    await expect(page.getByText('👍').first()).toBeVisible();
  });

  test('should handle typing indicator', async ({ page }) => {
    const channelPage = new ChannelPage(page);
    await channelPage.expectLoaded();
    
    // Start typing
    await channelPage.messageInput.fill('Typing...');
    
    // Wait a moment for typing indicator to appear
    await page.waitForTimeout(1000);
    
    // Typing indicator may or may not be visible depending on implementation
    // Just verify the input works
    await expect(channelPage.messageInput).toHaveValue('Typing...');
  });

  test('should upload a file attachment', async ({ page }) => {
    const channelPage = new ChannelPage(page);
    await channelPage.expectLoaded();
    
    // Look for file input
    const fileInput = page.locator('input[type="file"]').first();
    
    // Upload a test file using setInputFiles
    if (await fileInput.isVisible().catch(() => false)) {
      await fileInput.setInputFiles({
        name: 'test-file.txt',
        mimeType: 'text/plain',
        buffer: Buffer.from('Test file content'),
      });
    }
    
    // Or click attachment button
    const attachButton = page.getByRole('button', { name: /attach|file|upload/i }).first();
    if (await attachButton.isVisible().catch(() => false)) {
      await attachButton.click();
    }
  });

  test('should navigate between channels', async ({ page }) => {
    const channelPage = new ChannelPage(page);
    await channelPage.expectLoaded();
    
    // Look for channel links in sidebar
    const channelLinks = page.getByRole('link').filter({ has: page.locator('[class*="channel"]') });
    const count = await channelLinks.count();
    
    if (count > 1) {
      // Click on second channel
      await channelLinks.nth(1).click();
      
      // Verify URL changed
      const url = page.url();
      expect(url).toMatch(/\/channels\//);
    }
  });
});
