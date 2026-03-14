import { Page, Locator, expect } from '@playwright/test';

/**
 * Page object for the channel/chat page
 */
export class ChannelPage {
  readonly page: Page;
  readonly messageInput: Locator;
  readonly sendButton: Locator;
  readonly messageList: Locator;
  readonly channelHeader: Locator;
  readonly memberList: Locator;
  readonly userMenu: Locator;
  readonly sidebar: Locator;

  constructor(page: Page) {
    this.page = page;
    this.messageInput = page.locator('[data-testid="message-input"] textarea, [placeholder*="message"], [contenteditable="true"]').first();
    this.sendButton = page.getByRole('button', { name: /send/i });
    this.messageList = page.locator('[data-testid="message-list"], .message-list, [class*="message"]').first();
    this.channelHeader = page.locator('[data-testid="channel-header"], header').first();
    this.memberList = page.locator('[data-testid="member-list"], [class*="member"]').first();
    this.userMenu = page.getByRole('button', { name: /user menu|account/i });
    this.sidebar = page.locator('[data-testid="sidebar"], nav, aside').first();
  }

  async expectLoaded() {
    await expect(this.sidebar).toBeVisible();
    await expect(this.messageList.or(this.messageInput)).toBeVisible();
  }

  async sendMessage(text: string) {
    await this.messageInput.fill(text);
    await this.sendButton.click();
  }

  async expectMessageVisible(text: string) {
    const message = this.page.getByText(text).first();
    await expect(message).toBeVisible();
  }

  async openChannel(channelName: string) {
    const channelLink = this.page.getByRole('link', { name: new RegExp(channelName, 'i') });
    await channelLink.click();
    await expect(this.page).toHaveURL(new RegExp(channelName, 'i'));
  }

  async openUserMenu() {
    await this.userMenu.click();
  }

  async logout() {
    await this.openUserMenu();
    const logoutButton = this.page.getByRole('menuitem', { name: /logout|sign out/i });
    await logoutButton.click();
    await expect(this.page).toHaveURL(/\/login/);
  }

  async openSettings() {
    await this.openUserMenu();
    const settingsLink = this.page.getByRole('menuitem', { name: /settings/i });
    await settingsLink.click();
    await expect(this.page).toHaveURL(/\/settings/);
  }
}
