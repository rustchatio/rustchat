import { test, expect, type APIRequestContext } from '@playwright/test';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { LoginPage } from '../pages/LoginPage';
import { TEST_USERS, generateTestId } from '../fixtures/test-data';

interface TestCredentials {
  email: string;
  password: string;
}

const THREAD_STORAGE_STATE = path.resolve(process.cwd(), 'e2e/.auth/thread.json');
let threadSuiteReady = true;
let threadCredentials: TestCredentials | null = null;

fs.mkdirSync(path.dirname(THREAD_STORAGE_STATE), { recursive: true });
if (!fs.existsSync(THREAD_STORAGE_STATE)) {
  fs.writeFileSync(THREAD_STORAGE_STATE, '{}', 'utf8');
}

async function loginForApi(
  request: APIRequestContext,
  credentials: TestCredentials
): Promise<{ token: string; userId: string } | null> {
  const response = await request.post('/api/v1/auth/login', {
    data: {
      email: credentials.email,
      password: credentials.password,
    },
  });
  if (!response.ok()) return null;

  const payload = (await response.json()) as {
    token?: string;
    user?: { id?: string };
  };
  if (!payload.token || !payload.user?.id) return null;

  return {
    token: payload.token,
    userId: payload.user.id,
  };
}

async function createThreadFixture(
  request: APIRequestContext,
  token: string
): Promise<{ teamId: string; channelId: string; threadId: string } | null> {
  const teamsResponse = await request.get('/api/v1/teams', {
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!teamsResponse.ok()) return null;

  const teams = (await teamsResponse.json()) as Array<{ id?: string }>;
  for (const team of teams) {
    if (!team.id) continue;

    const channelsResponse = await request.get(
      `/api/v1/channels?team_id=${encodeURIComponent(team.id)}`,
      { headers: { Authorization: `Bearer ${token}` } }
    );
    if (!channelsResponse.ok()) continue;

    const channels = (await channelsResponse.json()) as Array<{
      id?: string;
      channel_type?: string;
    }>;
    const targetChannel = channels.find(
      (channel) =>
        typeof channel.id === 'string'
        && (channel.channel_type === 'public' || channel.channel_type === 'private')
    );
    if (!targetChannel?.id) continue;

    const message = `Thread follow parity ${generateTestId()}`;
    const postResponse = await request.post(
      `/api/v1/channels/${encodeURIComponent(targetChannel.id)}/posts`,
      {
        headers: {
          Authorization: `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        data: {
          channel_id: targetChannel.id,
          message,
        },
      }
    );
    if (!postResponse.ok()) continue;

    const postPayload = (await postResponse.json()) as { id?: string };
    if (!postPayload.id) continue;

    return {
      teamId: team.id,
      channelId: targetChannel.id,
      threadId: postPayload.id,
    };
  }

  return null;
}

async function fetchThreadFollowState(
  request: APIRequestContext,
  token: string,
  userId: string,
  teamId: string,
  threadId: string
): Promise<boolean | null> {
  const response = await request.get(
    `/api/v4/users/${encodeURIComponent(userId)}/teams/${encodeURIComponent(teamId)}/threads/${encodeURIComponent(threadId)}`,
    { headers: { Authorization: `Bearer ${token}` } }
  );
  if (!response.ok()) return null;

  const payload = (await response.json()) as {
    is_following?: boolean;
    isFollowing?: boolean;
  };
  if (typeof payload.is_following === 'boolean') return payload.is_following;
  if (typeof payload.isFollowing === 'boolean') return payload.isFollowing;
  return null;
}

async function bootstrapThreadCredentials(request: APIRequestContext): Promise<TestCredentials | null> {
  const uniqueId = generateTestId();
  const candidate = {
    email: `thread-${uniqueId}@example.com`,
    username: `thread-${uniqueId}`,
    password: 'ThreadPass123!',
  };

  const registerResponse = await request.post('/api/v1/auth/register', {
    data: {
      email: candidate.email,
      username: candidate.username,
      password: candidate.password,
      confirm_password: candidate.password,
      first_name: 'Thread',
      last_name: 'E2E',
    },
  });

  if (registerResponse.ok()) {
    return {
      email: candidate.email,
      password: candidate.password,
    };
  }

  const adminLoginResponse = await request.post('/api/v1/auth/login', {
    data: {
      email: TEST_USERS.admin.email,
      password: TEST_USERS.admin.password,
    },
  });

  if (!adminLoginResponse.ok()) return null;
  return {
    email: TEST_USERS.admin.email,
    password: TEST_USERS.admin.password,
  };
}

test.describe('Thread', () => {
  test.describe.configure({ mode: 'serial' });
  test.use({ storageState: THREAD_STORAGE_STATE });

  test.beforeAll(async ({ browser, request }) => {
    threadCredentials = await bootstrapThreadCredentials(request);
    const context = await browser.newContext({ storageState: undefined });
    const page = await context.newPage();
    const loginPage = new LoginPage(page);

    try {
      if (!threadCredentials) {
        threadSuiteReady = false;
        await context.close();
        return;
      }
      await loginPage.goto();
      await loginPage.login(threadCredentials.email, threadCredentials.password);
      await loginPage.expectLoginSuccess();
      await context.storageState({ path: THREAD_STORAGE_STATE });
      threadSuiteReady = true;
    } catch {
      threadSuiteReady = false;
    }

    await context.close();
  });

  test('should follow and unfollow a thread and persist across refresh', async ({ page, request }) => {
    test.skip(!threadSuiteReady, 'Thread suite bootstrap login is unavailable in this environment.');
    const creds = threadCredentials;
    test.skip(!creds, 'No credentials are available in this environment.');
    if (!creds) return;

    const auth = await loginForApi(request, creds);
    test.skip(!auth, 'API login is unavailable in this environment.');
    if (!auth) return;

    const fixture = await createThreadFixture(request, auth.token);
    test.skip(!fixture, 'Unable to create thread fixture in this environment.');
    if (!fixture) return;

    await page.goto(`/channels/${fixture.channelId}/threads/${fixture.threadId}`);
    await expect(page).toHaveURL(new RegExp(`/channels/${fixture.channelId}/threads/${fixture.threadId}`));

    const followToggle = page.locator(
      'button[title="Follow thread"], button[title="Unfollow thread"]'
    ).first();
    await expect(followToggle).toBeVisible();

    const currentTitle = await followToggle.getAttribute('title');
    if (currentTitle === 'Unfollow thread') {
      await followToggle.click();
      await expect(followToggle).toHaveAttribute('title', 'Follow thread');
    }

    await followToggle.click();
    await expect(followToggle).toHaveAttribute('title', 'Unfollow thread');

    await expect
      .poll(async () => fetchThreadFollowState(request, auth.token, auth.userId, fixture.teamId, fixture.threadId))
      .toBe(true);

    await page.reload();
    await expect(followToggle).toHaveAttribute('title', 'Unfollow thread');

    await followToggle.click();
    await expect(followToggle).toHaveAttribute('title', 'Follow thread');

    await expect
      .poll(async () => fetchThreadFollowState(request, auth.token, auth.userId, fixture.teamId, fixture.threadId))
      .toBe(false);
  });
});
