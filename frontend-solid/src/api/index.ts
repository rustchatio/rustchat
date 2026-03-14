// ============================================
// API Client Exports
// ============================================

// Client
export { client, API_BASE_URL, WS_URL, get, post, put, patch, del } from './client';
export { getErrorMessage, getErrorStatus } from './client';

// Types
export type * from './types';

// API Modules
export { authApi } from './auth';
export { usersApi } from './users';
export { channelsApi } from './channels';
export { postsApi } from './messages';

// Interceptors
export { interceptors, setupInterceptors, triggerTokenRefresh, clearRequestQueue } from './interceptors';

// Re-export for convenience
export { default as clientDefault } from './client';
