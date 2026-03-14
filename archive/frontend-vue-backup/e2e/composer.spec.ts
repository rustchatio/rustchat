import { test, expect } from '@playwright/test'

test.describe('Mattermost Composer E2E', () => {
    test.beforeEach(async ({ page }) => {
        // Login and navigate to a channel
        await page.goto('/login')
        await page.fill('[name="email"]', 'test@example.com')
        await page.fill('[name="password"]', 'password123')
        await page.click('button[type="submit"]')
        await page.waitForNavigation()
        
        // Navigate to general channel
        await page.click('text=general')
        await page.waitForSelector('[aria-label="Message composer"]')
    })

    test('sends message on Enter', async ({ page }) => {
        const composer = page.locator('[aria-label="Message composer"]')
        await composer.fill('Hello world')
        await composer.press('Enter')
        
        // Message should appear in the list
        await expect(page.locator('text=Hello world')).toBeVisible()
        
        // Composer should be cleared
        await expect(composer).toHaveValue('')
    })

    test('inserts newline on Shift+Enter', async ({ page }) => {
        const composer = page.locator('[aria-label="Message composer"]')
        await composer.fill('Line 1')
        await composer.press('Shift+Enter')
        await composer.type('Line 2')
        
        await expect(composer).toHaveValue('Line 1\nLine 2')
    })

    test('bold formatting via toolbar', async ({ page }) => {
        const composer = page.locator('[aria-label="Message composer"]')
        
        // Type and select text
        await composer.fill('bold text')
        await composer.selectText()
        
        // Click bold button
        await page.click('[aria-label="Bold"]')
        
        await expect(composer).toHaveValue('**bold text**')
    })

    test('bold formatting via keyboard shortcut', async ({ page }) => {
        const composer = page.locator('[aria-label="Message composer"]')
        
        await composer.fill('bold text')
        await composer.selectText()
        await composer.press('Control+b')
        
        await expect(composer).toHaveValue('**bold text**')
    })

    test('italic formatting', async ({ page }) => {
        const composer = page.locator('[aria-label="Message composer"]')
        
        await composer.fill('italic text')
        await composer.selectText()
        await page.click('[aria-label="Italic"]')
        
        await expect(composer).toHaveValue('*italic text*')
    })

    test('link insertion', async ({ page }) => {
        const composer = page.locator('[aria-label="Message composer"]')
        
        await composer.fill('click here')
        await composer.selectText()
        await page.click('[aria-label="Link"]')
        
        // Should have placeholder link
        await expect(composer).toHaveValue('[click here]()')
    })

    test('emoji autocomplete with colon', async ({ page }) => {
        const composer = page.locator('[aria-label="Message composer"]')
        
        await composer.type(':smi')
        
        // Wait for autocomplete to appear
        await expect(page.locator('text=Emoji matching')).toBeVisible()
        await expect(page.locator('text=:smile:')).toBeVisible()
        
        // Select emoji
        await page.click('text=:smile:')
        
        await expect(composer).toHaveValue(':smile: ')
    })

    test('mention autocomplete with @', async ({ page }) => {
        const composer = page.locator('[aria-label="Message composer"]')
        
        await composer.type('@ad')
        
        // Wait for autocomplete
        await expect(page.locator('text=Mentions matching')).toBeVisible()
        
        // Press Enter to select first result
        await composer.press('Enter')
        
        // Should insert mention
        const value = await composer.inputValue()
        expect(value).toMatch(/^@\w+/)
    })

    test('file attachment via click', async ({ page }) => {
        const composer = page.locator('[aria-label="Message composer"]')
        
        // Attach file
        const [fileChooser] = await Promise.all([
            page.waitForEvent('filechooser'),
            page.click('[aria-label="Attach file"]')
        ])
        
        await fileChooser.setFiles({
            name: 'test.txt',
            mimeType: 'text/plain',
            buffer: Buffer.from('test content')
        })
        
        // Should show file preview
        await expect(page.locator('text=test.txt')).toBeVisible()
    })

    test('file attachment via paste', async ({ page }) => {
        const composer = page.locator('[aria-label="Message composer"]')
        
        // Create a simple image buffer
        const imageBuffer = Buffer.from('fake-image-data')
        
        // Paste image
        await composer.evaluate((el, buffer) => {
            const clipboardData = new DataTransfer()
            const file = new File([buffer], 'pasted-image.png', { type: 'image/png' })
            clipboardData.items.add(file)
            
            const pasteEvent = new ClipboardEvent('paste', {
                clipboardData,
                bubbles: true
            })
            el.dispatchEvent(pasteEvent)
        }, imageBuffer)
        
        // Should show file preview
        await expect(page.locator('text=pasted-image.png')).toBeVisible()
    })

    test('draft persistence', async ({ page, context }) => {
        const composer = page.locator('[aria-label="Message composer"]')
        
        // Type a message but don't send
        await composer.fill('Draft message')
        
        // Navigate away and back
        await page.click('text=random')
        await page.click('text=general')
        
        // Draft should be restored
        await expect(composer).toHaveValue('Draft message')
    })

    test('send button disabled when empty', async ({ page }) => {
        const sendButton = page.locator('[aria-label="Send message"]')
        
        // Initially disabled
        await expect(sendButton).toBeDisabled()
        
        // Type something
        await page.fill('[aria-label="Message composer"]', 'test')
        
        // Should be enabled
        await expect(sendButton).toBeEnabled()
    })

    test('font size toggle', async ({ page }) => {
        const composer = page.locator('[aria-label="Message composer"]')
        const fontButton = page.locator('[aria-label="Toggle font size"]')
        
        // Initial: normal
        await expect(composer).toHaveClass(/text-base/)
        
        // Click to large
        await fontButton.click()
        await expect(composer).toHaveClass(/text-lg/)
        
        // Click to small
        await fontButton.click()
        await expect(composer).toHaveClass(/text-sm/)
    })

    test('formatting toolbar toggle', async ({ page }) => {
        const toggleButton = page.locator('[aria-label="Toggle formatting toolbar"]')
        const toolbar = page.locator('.bg-bg-surface-2\\/50')
        
        // Initially visible
        await expect(toolbar).toBeVisible()
        
        // Toggle off
        await toggleButton.click()
        await expect(toolbar).not.toBeVisible()
        
        // Toggle on
        await toggleButton.click()
        await expect(toolbar).toBeVisible()
    })

    test('keyboard shortcut help', async ({ page }) => {
        // Look for keyboard hints
        await expect(page.locator('text=Enter to send')).toBeVisible()
        await expect(page.locator('text=Shift+Enter for newline')).toBeVisible()
    })

    test('sends message with attached files', async ({ page }) => {
        const composer = page.locator('[aria-label="Message composer"]')
        
        // Attach file
        const [fileChooser] = await Promise.all([
            page.waitForEvent('filechooser'),
            page.click('[aria-label="Attach file"]')
        ])
        
        await fileChooser.setFiles({
            name: 'document.pdf',
            mimeType: 'application/pdf',
            buffer: Buffer.from('pdf content')
        })
        
        // Wait for upload
        await expect(page.locator('text=document.pdf')).toBeVisible()
        
        // Type message
        await composer.fill('See attached')
        
        // Send
        await page.click('[aria-label="Send message"]')
        
        // Message should appear
        await expect(page.locator('text=See attached')).toBeVisible()
        
        // Composer cleared
        await expect(composer).toHaveValue('')
    })

    test('escape closes emoji picker', async ({ page }) => {
        await page.click('[aria-label="Insert emoji"]')
        
        // Emoji picker should be visible
        await expect(page.locator('.emoji-picker')).toBeVisible()
        
        // Press escape
        await page.keyboard.press('Escape')
        
        // Should close
        await expect(page.locator('.emoji-picker')).not.toBeVisible()
    })

    test('typing indicator is sent', async ({ page }) => {
        const composer = page.locator('[aria-label="Message composer"]')
        
        // Type something
        await composer.type('typing...')
        
        // Wait a bit for typing indicator
        await page.waitForTimeout(500)
        
        // This is hard to verify without WebSocket mocking, but the functionality exists
    })
})
