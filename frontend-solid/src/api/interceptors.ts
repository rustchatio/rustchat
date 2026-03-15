// ============================================
// API Interceptors for Token Refresh
// ============================================

import { client } from './client';
import type { AxiosError, InternalAxiosRequestConfig } from 'axios';
import { authStore } from '../stores/auth';
import { generateUUID } from '../utils/uuid';

// ============================================
// Token Refresh Queue Management
// ============================================

interface QueuedRequest {
  resolve: (value: unknown) => void;
  reject: (reason?: unknown) => void;
  config: InternalAxiosRequestConfig;
}

let isRefreshing = false;
let failedQueue: QueuedRequest[] = [];

/**
 * Process queued requests after token refresh
 */
function processQueue(error: Error | null, token: string | null = null) {
  failedQueue.forEach((request) => {
    if (error) {
      request.reject(error);
    } else if (token) {
      request.config.headers.Authorization = `Bearer ${token}`;
      request.resolve(client(request.config));
    }
  });

  failedQueue = [];
}

/**
 * Add request to failed queue
 */
function addToQueue(config: InternalAxiosRequestConfig): Promise<unknown> {
  return new Promise((resolve, reject) => {
    failedQueue.push({ resolve, reject, config });
  });
}

// ============================================
// Request Interceptor
// ============================================

export function setupRequestInterceptor() {
  client.interceptors.request.use(
    (config: InternalAxiosRequestConfig) => {
      // Add auth token if available
      const token = authStore.token;
      if (token && config.headers) {
        config.headers.Authorization = `Bearer ${token}`;
      }

      // Add request ID for debugging
      if (config.headers) {
        config.headers['X-Request-ID'] = generateUUID();
      }

      return config;
    },
    (error: AxiosError) => {
      return Promise.reject(error);
    }
  );
}

// ============================================
// Response Interceptor with Token Refresh
// ============================================

export function setupResponseInterceptor() {
  client.interceptors.response.use(
    (response) => response,
    async (error: AxiosError) => {
      const originalRequest = error.config as InternalAxiosRequestConfig & { _retry?: boolean };

      if (!originalRequest) {
        return Promise.reject(error);
      }

      // Handle 401 Unauthorized
      if (error.response?.status === 401 && !originalRequest._retry) {
        // Check if this is a login or refresh request (avoid infinite loops)
        const isAuthEndpoint = originalRequest.url?.includes('/auth/login') ||
                              originalRequest.url?.includes('/auth/refresh') ||
                              originalRequest.url?.includes('/auth/oidc') ||
                              originalRequest.url?.includes('/auth/saml');

        if (isAuthEndpoint) {
          return Promise.reject(error);
        }

        // If already refreshing, queue this request
        if (isRefreshing) {
          try {
            return await addToQueue(originalRequest);
          } catch (err) {
            return Promise.reject(err);
          }
        }

        originalRequest._retry = true;
        isRefreshing = true;

        try {
          // Attempt to refresh token
          const refreshed = await authStore.refreshToken();

          if (!refreshed) {
            // Token refresh not available or failed - just logout
            throw new Error('Token refresh failed');
          }

          // Get new token and process queue
          const newToken = authStore.token;
          processQueue(null, newToken);

          // Retry original request with new token
          if (originalRequest.headers) {
            originalRequest.headers.Authorization = `Bearer ${newToken}`;
          }
          return client(originalRequest);
        } catch (refreshError) {
          // Token refresh failed, logout and redirect
          processQueue(refreshError as Error, null);
          // Don't logout here - let the app handle it
          return Promise.reject(refreshError);
        } finally {
          isRefreshing = false;
        }
      }

      // Handle 403 Forbidden
      if (error.response?.status === 403) {
        console.warn('Access forbidden:', originalRequest.url);
      }

      // Handle 404 Not Found
      if (error.response?.status === 404) {
        console.warn('Resource not found:', originalRequest.url);
      }

      // Handle network errors with retry
      if (!error.response && !originalRequest._retry) {
        originalRequest._retry = true;
        console.warn('Network error, retrying once:', originalRequest.url);
        await new Promise((resolve) => setTimeout(resolve, 1000));
        return client(originalRequest);
      }

      return Promise.reject(error);
    }
  );
}

// ============================================
// Initialize Interceptors
// ============================================

export function setupInterceptors() {
  setupRequestInterceptor();
  setupResponseInterceptor();
}

// ============================================
// Manual Token Refresh Trigger
// ============================================

/**
 * Manually trigger a token refresh
 * Useful for session extension user actions
 */
export async function triggerTokenRefresh(): Promise<boolean> {
  if (isRefreshing) {
    // Wait for current refresh to complete
    return new Promise((resolve, reject) => {
      failedQueue.push({
        resolve: () => resolve(true),
        reject: (err) => reject(err),
        config: {} as InternalAxiosRequestConfig,
      });
    });
  }

  return authStore.refreshToken();
}

// ============================================
// Clear Queue (for logout)
// ============================================

export function clearRequestQueue() {
  failedQueue = [];
  isRefreshing = false;
}

// ============================================
// Exports
// ============================================

export const interceptors = {
  setup: setupInterceptors,
  setupRequest: setupRequestInterceptor,
  setupResponse: setupResponseInterceptor,
  triggerRefresh: triggerTokenRefresh,
  clearQueue: clearRequestQueue,
};

export default interceptors;
