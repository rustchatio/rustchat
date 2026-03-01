import { expect, test } from '@playwright/test'

const LOGIN_ID = process.env.E2E_SETTINGS_LOGIN_ID || 'compat_smoke_1772369282@example.com'
const PASSWORD = process.env.E2E_SETTINGS_PASSWORD || 'Password123!'
const APP_BASE_URL = process.env.E2E_SETTINGS_BASE_URL || 'http://localhost:8080'

async function login(page: import('@playwright/test').Page) {
  await page.goto(`${APP_BASE_URL}/login`)

  const loginInput = page.locator('#email, input[type="email"]').first()
  const passwordInput = page.locator('#password, input[type="password"]').first()

  await expect(loginInput).toBeVisible()
  await expect(passwordInput).toBeVisible()

  await loginInput.fill(LOGIN_ID)
  await passwordInput.fill(PASSWORD)
  await page.locator('button[type="submit"]').first().click()

  await page.waitForLoadState('networkidle')
}

async function openSettingsFromUserMenu(page: import('@playwright/test').Page) {
  const avatarTrigger = page.locator('header div.ml-2.relative > div.cursor-pointer').first()
  await expect(avatarTrigger).toBeVisible({ timeout: 15000 })
  await avatarTrigger.click()

  const profileButton = page.getByRole('button', { name: 'Profile' })
  await expect(profileButton).toBeVisible({ timeout: 10000 })
  await profileButton.click()

  const settingsModal = page.locator('div[role="dialog"] > div.relative').first()
  await expect(settingsModal).toBeVisible({ timeout: 10000 })
  return settingsModal
}

async function captureTab(
  page: import('@playwright/test').Page,
  modal: import('@playwright/test').Locator,
  tabName: string,
  screenshotName: string,
  testInfo: import('@playwright/test').TestInfo,
) {
  await page.getByRole('button', { name: tabName }).first().click()
  await page.waitForTimeout(250)
  await modal.screenshot({ path: testInfo.outputPath(screenshotName) })
}

test('capture settings parity surfaces', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 1512, height: 982 })

  await login(page)
  const modal = await openSettingsFromUserMenu(page)

  await captureTab(page, modal, 'Notifications', 'settings-notifications.png', testInfo)
  await captureTab(page, modal, 'Display', 'settings-display.png', testInfo)
  await captureTab(page, modal, 'Sidebar', 'settings-sidebar.png', testInfo)
  await captureTab(page, modal, 'Advanced', 'settings-advanced.png', testInfo)
  await captureTab(page, modal, 'Calls', 'settings-calls.png', testInfo)
})
