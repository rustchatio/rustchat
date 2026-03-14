import { createStore, produce } from 'solid-js/store';
import { createSignal } from 'solid-js';

// ============================================
// Types
// ============================================

export type Presence = 'online' | 'away' | 'dnd' | 'offline';

export interface PresenceUser {
  userId: string;
  username: string;
  presence: Presence;
  lastActiveAt?: string;
  avatarUrl?: string;
}

export interface TypingUser {
  userId: string;
  username: string;
  channelId: string;
  timestamp: number;
  threadRootId?: string;
}

// ============================================
// Store State
// ============================================

const TYPING_TIMEOUT = 5000; // 5 seconds
const CLEANUP_INTERVAL = 3000; // 3 seconds

// Current user's presence
const [selfPresence, setSelfPresenceSignal] = createSignal<PresenceUser | null>(null);

// Teammates presence map: userId -> PresenceUser
const [presenceMap, setPresenceMap] = createStore<Record<string, PresenceUser>>({});

// Typing users map: `${channelId}:${threadRootId || 'root'}:${userId}` -> TypingUser
const [typingUsers, setTypingUsers] = createStore<Record<string, TypingUser>>({});

// ============================================
// Cleanup Effect
// ============================================

// Clean up stale typing indicators
let cleanupInterval: ReturnType<typeof setInterval> | null = null;

export function startCleanupInterval() {
  if (cleanupInterval) return;

  cleanupInterval = setInterval(() => {
    const now = Date.now();
    setTypingUsers(
      produce((users) => {
        for (const key in users) {
          if (now - users[key].timestamp > TYPING_TIMEOUT) {
            delete users[key];
          }
        }
      })
    );
  }, CLEANUP_INTERVAL);
}

export function stopCleanupInterval() {
  if (cleanupInterval) {
    clearInterval(cleanupInterval);
    cleanupInterval = null;
  }
}

// Start cleanup on module load
startCleanupInterval();

// ============================================
// Self Presence Actions
// ============================================

export function setSelfPresence(userData: Partial<PresenceUser>) {
  const current = selfPresence();
  if (!current && userData.userId) {
    setSelfPresenceSignal({
      userId: userData.userId,
      username: userData.username || '',
      presence: userData.presence || 'online',
      lastActiveAt: userData.lastActiveAt || new Date().toISOString(),
      avatarUrl: userData.avatarUrl,
    });
  } else if (current) {
    setSelfPresenceSignal({
      ...current,
      ...userData,
      lastActiveAt: new Date().toISOString(),
    });
  }
}

export function updateSelfPresence(presence: Presence) {
  const current = selfPresence();
  if (current) {
    setSelfPresenceSignal({
      ...current,
      presence,
      lastActiveAt: new Date().toISOString(),
    });
  }
}

// ============================================
// User Presence Actions
// ============================================

export function setUserPresence(userId: string, username: string, presence: Presence, avatarUrl?: string) {
  setPresenceMap(userId, {
    userId,
    username,
    presence,
    lastActiveAt: new Date().toISOString(),
    avatarUrl,
  });
}

export function updatePresenceFromEvent(userId: string, presence: Presence) {
  const lowerPresence = presence.toLowerCase() as Presence;

  const self = selfPresence();
  if (self?.userId === userId) {
    updateSelfPresence(lowerPresence);
    return;
  }

  setPresenceMap(
    produce((map) => {
      const user = map[userId];
      if (user) {
        user.presence = lowerPresence;
        user.lastActiveAt = new Date().toISOString();
      } else {
        map[userId] = {
          userId,
          username: '',
          presence: lowerPresence,
          lastActiveAt: new Date().toISOString(),
        };
      }
    })
  );
}

export function getUserPresence(userId: string) {
  return () => {
    const self = selfPresence();
    if (self?.userId === userId) return self;
    return presenceMap[userId];
  };
}

// ============================================
// Typing Indicators
// ============================================

function getTypingKey(userId: string, channelId: string, threadRootId?: string): string {
  return `${channelId}:${threadRootId || 'root'}:${userId}`;
}

export function addTypingUser(
  userId: string,
  username: string,
  channelId: string,
  threadRootId?: string
) {
  const key = getTypingKey(userId, channelId, threadRootId);
  setTypingUsers(key, {
    userId,
    username,
    channelId,
    timestamp: Date.now(),
    threadRootId,
  });
}

export function removeTypingUser(userId: string, channelId: string, threadRootId?: string) {
  const key = getTypingKey(userId, channelId, threadRootId);
  setTypingUsers(produce((users) => { delete users[key]; }));
}

export function getTypingUsersForChannel(channelId: string, threadRootId?: string) {
  return () => {
    const users: TypingUser[] = [];
    for (const key in typingUsers) {
      const user = typingUsers[key];
      if (user.channelId === channelId) {
        if (threadRootId) {
          if (user.threadRootId === threadRootId) users.push(user);
        } else {
          if (!user.threadRootId) users.push(user);
        }
      }
    }
    return users;
  };
}

// ============================================
// Computed
// ============================================

export const onlineCount = () => {
  let count = 0;
  for (const key in presenceMap) {
    if (presenceMap[key].presence === 'online') count++;
  }
  const self = selfPresence();
  if (self?.presence === 'online') count++;
  return count;
};

export const onlineUsers = () => {
  const users: PresenceUser[] = [];
  const self = selfPresence();

  for (const key in presenceMap) {
    if (presenceMap[key].presence === 'online') {
      users.push(presenceMap[key]);
    }
  }

  if (self && self.presence === 'online') {
    users.push(self);
  }

  return users;
};

// ============================================
// Clear State
// ============================================

export function clear() {
  setSelfPresenceSignal(null);
  setPresenceMap({});
  setTypingUsers({});
}

// ============================================
// Exports
// ============================================

export const presenceStore = {
  // State
  selfPresence,
  presenceMap,
  typingUsers,

  // Computed
  onlineCount,
  onlineUsers,

  // Actions
  setSelfPresence,
  updateSelfPresence,
  setUserPresence,
  updatePresenceFromEvent,
  getUserPresence,
  addTypingUser,
  removeTypingUser,
  getTypingUsersForChannel,

  // Lifecycle
  startCleanupInterval,
  stopCleanupInterval,
  clear,
};

export default presenceStore;
