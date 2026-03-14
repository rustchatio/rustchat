// ============================================
// Users API Methods
// ============================================

import { client } from './client';
import type {
  User,
  UpdateUserRequest,
  ChangePasswordRequest,
  UserStatus,
  UserStatusResponse,
  PaginationParams,
} from './types';

// ============================================
// User CRUD
// ============================================

export async function listUsers(params?: PaginationParams): Promise<User[]> {
  const response = await client.get<User[]>('/users', { params });
  return response.data;
}

export async function getUser(id: string): Promise<User> {
  const response = await client.get<User>(`/users/${id}`);
  return response.data;
}

export async function getUserByUsername(username: string): Promise<User> {
  const response = await client.get<User>(`/users/username/${username}`);
  return response.data;
}

export async function getUserByEmail(email: string): Promise<User> {
  const response = await client.get<User>(`/users/email/${email}`);
  return response.data;
}

export async function createUser(user: Partial<User>): Promise<User> {
  const response = await client.post<User>('/users', user);
  return response.data;
}

export async function updateUser(id: string, data: UpdateUserRequest): Promise<User> {
  const response = await client.put<User>(`/users/${id}`, data);
  return response.data;
}

export async function patchUser(id: string, data: Partial<UpdateUserRequest>): Promise<User> {
  const response = await client.patch<User>(`/users/${id}`, data);
  return response.data;
}

export async function deleteUser(id: string): Promise<void> {
  await client.delete(`/users/${id}`);
}

// ============================================
// Password
// ============================================

export async function changePassword(id: string, data: ChangePasswordRequest): Promise<void> {
  await client.post(`/users/${id}/password`, data);
}

export async function sendPasswordReset(email: string): Promise<void> {
  await client.post('/users/password/reset/send', { email });
}

export async function resetPassword(token: string, newPassword: string): Promise<void> {
  await client.post('/users/password/reset', { token, new_password: newPassword });
}

// ============================================
// Status
// ============================================

export async function getUserStatus(userId: string): Promise<UserStatus> {
  const response = await client.get<UserStatus>(`/users/${userId}/status`);
  return response.data;
}

export async function getStatusesByIds(userIds: string[]): Promise<UserStatusResponse[]> {
  const response = await client.post<UserStatusResponse[]>('/users/status/ids', userIds);
  return response.data;
}

// ============================================
// Preferences
// ============================================

export async function getUserPreferences(userId: string): Promise<Record<string, unknown>[]> {
  const response = await client.get<Record<string, unknown>[]>(`/users/${userId}/preferences`);
  return response.data;
}

export async function saveUserPreferences(
  userId: string,
  preferences: Record<string, unknown>[]
): Promise<void> {
  await client.put(`/users/${userId}/preferences`, preferences);
}

export async function deleteUserPreferences(
  userId: string,
  preferenceIds: string[]
): Promise<void> {
  await client.post(`/users/${userId}/preferences/delete`, preferenceIds);
}

// ============================================
// Teams
// ============================================

export async function getUserTeams(userId: string): Promise<unknown[]> {
  const response = await client.get<unknown[]>(`/users/${userId}/teams`);
  return response.data;
}

export async function getTeamMembers(userId: string, teamId: string): Promise<unknown> {
  const response = await client.get<unknown>(`/users/${userId}/teams/${teamId}/members`);
  return response.data;
}

// ============================================
// Users API Object
// ============================================

export const usersApi = {
  list: listUsers,
  get: getUser,
  getByUsername: getUserByUsername,
  getByEmail: getUserByEmail,
  create: createUser,
  update: updateUser,
  patch: patchUser,
  delete: deleteUser,
  changePassword,
  sendPasswordReset,
  resetPassword,
  getStatus: getUserStatus,
  getStatusesByIds,
  getPreferences: getUserPreferences,
  savePreferences: saveUserPreferences,
  deletePreferences: deleteUserPreferences,
  getTeams: getUserTeams,
  getTeamMember: getTeamMembers,
};

export default usersApi;
