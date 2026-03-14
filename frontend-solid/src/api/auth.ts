// ============================================
// Auth API Methods
// ============================================

import { client } from './client';
import type {
  LoginRequest,
  LoginResponse,
  AuthPolicy,
  User,
  UpdateStatusRequest,
  UserStatus,
} from './types';

// ============================================
// Authentication
// ============================================

export async function login(credentials: LoginRequest): Promise<LoginResponse> {
  const response = await client.post<LoginResponse>('/auth/login', credentials);
  return response.data;
}

export async function logout(): Promise<void> {
  try {
    await client.post('/auth/logout');
  } catch (e) {
    // Ignore errors on logout
  }
}

export async function refreshToken(): Promise<string | null> {
  try {
    const response = await client.post<{ token: string }>('/auth/refresh');
    return response.data.token;
  } catch {
    return null;
  }
}

// ============================================
// Current User
// ============================================

export async function getMe(): Promise<User> {
  const response = await client.get<User>('/auth/me');
  return response.data;
}

export async function getAuthPolicy(): Promise<AuthPolicy> {
  const response = await client.get<AuthPolicy>('/auth/policy');
  return response.data;
}

// ============================================
// Status
// ============================================

export async function getMyStatus(): Promise<UserStatus> {
  const response = await client.get<UserStatus>('/users/me/status');
  return response.data;
}

export async function updateStatus(data: UpdateStatusRequest): Promise<UserStatus> {
  const response = await client.put<UserStatus>('/users/me/status', data);
  return response.data;
}

export async function clearStatus(): Promise<UserStatus> {
  const response = await client.delete<UserStatus>('/users/me/status');
  return response.data;
}

// ============================================
// Auth API Object
// ============================================

export const authApi = {
  login,
  logout,
  refreshToken,
  getMe,
  getAuthPolicy,
  getMyStatus,
  updateStatus,
  clearStatus,
};

export default authApi;
