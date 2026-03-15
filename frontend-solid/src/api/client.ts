import axios, { AxiosInstance, AxiosRequestConfig, AxiosError, AxiosResponse } from 'axios';
import { authStore, logout } from '../stores/auth';
import { generateUUID } from '../utils/uuid';

// ============================================
// Configuration
// ============================================

const API_BASE_URL = import.meta.env.VITE_API_URL || '/api/v1';
const WS_URL = import.meta.env.VITE_WS_URL || '/api/v4/websocket';

// ============================================
// Axios Instance
// ============================================

const client: AxiosInstance = axios.create({
  baseURL: API_BASE_URL,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// ============================================
// Request Deduplication
// ============================================

interface PendingRequest {
  promise: Promise<AxiosResponse<unknown>>;
  timestamp: number;
}

const pendingRequests = new Map<string, PendingRequest>();
const DEDUPLICATION_WINDOW = 500; // 500ms

function getRequestKey(config: AxiosRequestConfig): string {
  return `${config.method}:${config.url}:${JSON.stringify(config.params || {})}:${JSON.stringify(config.data || {})}`;
}

function dedupeRequest<T>(config: AxiosRequestConfig): Promise<AxiosResponse<T>> | null {
  const key = getRequestKey(config);
  const pending = pendingRequests.get(key);

  if (pending && Date.now() - pending.timestamp < DEDUPLICATION_WINDOW) {
    return pending.promise as Promise<AxiosResponse<T>>;
  }

  return null;
}

function trackRequest<T>(config: AxiosRequestConfig, promise: Promise<AxiosResponse<T>>): void {
  const key = getRequestKey(config);
  pendingRequests.set(key, { promise: promise as Promise<AxiosResponse<unknown>>, timestamp: Date.now() });

  // Clean up after deduplication window
  setTimeout(() => {
    pendingRequests.delete(key);
  }, DEDUPLICATION_WINDOW + 100);
}

// ============================================
// Request Interceptor
// ============================================

client.interceptors.request.use(
  (config) => {
    // Add auth token
    const token = authStore.token;
    console.log('[API Client] Request:', config.method?.toUpperCase(), config.url, 'Token:', token ? 'present' : 'missing');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }

    // Add request ID for debugging
    config.headers['X-Request-ID'] = generateUUID();

    return config;
  },
  (error) => Promise.reject(error)
);

// ============================================
// Response Interceptor
// ============================================

let isRefreshing = false;
let refreshSubscribers: Array<(token: string) => void> = [];

function subscribeTokenRefresh(callback: (token: string) => void) {
  refreshSubscribers.push(callback);
}

function onTokenRefreshed(newToken: string) {
  refreshSubscribers.forEach((callback) => callback(newToken));
  refreshSubscribers = [];
}

client.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    const originalRequest = error.config as AxiosRequestConfig & { _retry?: boolean };

    if (!originalRequest) {
      return Promise.reject(error);
    }

    // Handle 401 Unauthorized
    if (error.response?.status === 401 && !originalRequest._retry) {
      if (isRefreshing) {
        // Wait for token refresh
        return new Promise((resolve) => {
          subscribeTokenRefresh((newToken) => {
            originalRequest.headers = originalRequest.headers || {};
            originalRequest.headers.Authorization = `Bearer ${newToken}`;
            resolve(client(originalRequest));
          });
        });
      }

      originalRequest._retry = true;
      isRefreshing = true;

      try {
        // Try to refresh token
        const newToken = await authStore.refreshToken();
        if (!newToken) {
          throw new Error('Token refresh failed');
        }
        onTokenRefreshed(authStore.token);
        originalRequest.headers = originalRequest.headers || {};
        originalRequest.headers.Authorization = `Bearer ${authStore.token}`;
        return client(originalRequest);
      } catch (refreshError) {
        // Token refresh failed, logout
        await logout('expired');
        return Promise.reject(refreshError);
      } finally {
        isRefreshing = false;
      }
    }

    // Handle 403 Forbidden
    if (error.response?.status === 403) {
      console.error('Access forbidden:', error.response.data);
    }

    // Handle 404 Not Found
    if (error.response?.status === 404) {
      console.error('Resource not found:', originalRequest.url);
    }

    // Handle 500+ Server Errors with retry
    if (error.response && error.response.status >= 500 && !originalRequest._retry) {
      originalRequest._retry = true;
      // Retry once after 1 second
      await new Promise((resolve) => setTimeout(resolve, 1000));
      return client(originalRequest);
    }

    return Promise.reject(error);
  }
);

// ============================================
// API Methods with Deduplication
// ============================================

export async function get<T>(url: string, config?: AxiosRequestConfig): Promise<AxiosResponse<T>> {
  const deduped = dedupeRequest<T>({ ...config, method: 'GET', url });
  if (deduped) return deduped;

  const promise = client.get<T>(url, config);
  trackRequest({ ...config, method: 'GET', url }, promise);
  return promise;
}

export async function post<T>(
  url: string,
  data?: unknown,
  config?: AxiosRequestConfig
): Promise<AxiosResponse<T>> {
  return client.post<T>(url, data, config);
}

export async function put<T>(
  url: string,
  data?: unknown,
  config?: AxiosRequestConfig
): Promise<AxiosResponse<T>> {
  return client.put<T>(url, data, config);
}

export async function patch<T>(
  url: string,
  data?: unknown,
  config?: AxiosRequestConfig
): Promise<AxiosResponse<T>> {
  return client.patch<T>(url, data, config);
}

export async function del<T>(url: string, config?: AxiosRequestConfig): Promise<AxiosResponse<T>> {
  return client.delete<T>(url, config);
}

// ============================================
// Error Handling Helpers
// ============================================

export function getErrorMessage(error: unknown): string {
  if (axios.isAxiosError(error)) {
    const axiosError = error as AxiosError<{ message?: string; detailed_error?: string }>;
    return (
      axiosError.response?.data?.message ||
      axiosError.response?.data?.detailed_error ||
      axiosError.message ||
      'An error occurred'
    );
  }
  if (error instanceof Error) {
    return error.message;
  }
  return 'An unknown error occurred';
}

export function getErrorStatus(error: unknown): number | null {
  if (axios.isAxiosError(error)) {
    return error.response?.status || null;
  }
  return null;
}

// ============================================
// Exports
// ============================================

export { client, API_BASE_URL, WS_URL };
export default client;
