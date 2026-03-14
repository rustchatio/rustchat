/**
 * Test data and utilities for E2E tests
 */

export const TEST_USERS = {
  admin: {
    email: process.env.E2E_ADMIN_EMAIL || 'admin@rustchat.local',
    password: process.env.E2E_ADMIN_PASSWORD || 'Admin123!',
    username: 'admin',
    firstName: 'Admin',
    lastName: 'User',
  },
  user1: {
    email: 'user1@rustchat.local',
    password: 'User123!',
    username: 'user1',
    firstName: 'Test',
    lastName: 'User1',
  },
  user2: {
    email: 'user2@rustchat.local',
    password: 'User123!',
    username: 'user2',
    firstName: 'Test',
    lastName: 'User2',
  },
} as const;

export const TEST_CHANNEL = {
  name: 'test-channel-e2e',
  displayName: 'Test Channel E2E',
  purpose: 'Channel for E2E testing',
  type: 'public' as const,
};

export const TEST_MESSAGE = {
  text: 'Hello from E2E test! 👋',
  editedText: 'Hello from E2E test! (edited) ✏️',
  withMarkdown: '# Heading\n\n**Bold** and *italic* text',
  withCode: '```typescript\nconst x = 1;\n```',
};

/**
 * Generate a unique test identifier to avoid collisions
 */
export function generateTestId(): string {
  return `e2e-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * Wait for a websocket message to be processed
 */
export async function waitForWebsocketDelay(ms = 500): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}
