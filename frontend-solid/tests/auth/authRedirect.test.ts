import { describe, expect, it } from 'vitest';
import { normalizeAuthRedirectPath } from '../../src/utils/authRedirect';
import { isAdminRole } from '../../src/utils/roles';

describe('auth redirect normalization', () => {
  it('uses default path for empty values', () => {
    expect(normalizeAuthRedirectPath(null)).toBe('/');
    expect(normalizeAuthRedirectPath(undefined)).toBe('/');
    expect(normalizeAuthRedirectPath('')).toBe('/');
  });

  it('rejects unsafe or auth-loop paths', () => {
    expect(normalizeAuthRedirectPath('//evil.example')).toBe('/');
    expect(normalizeAuthRedirectPath('/login')).toBe('/');
    expect(normalizeAuthRedirectPath('/login/callback')).toBe('/');
    expect(normalizeAuthRedirectPath('/channels/general')).toBe('/');
  });

  it('preserves valid in-app paths', () => {
    expect(normalizeAuthRedirectPath('/channels/abc123')).toBe('/channels/abc123');
    expect(normalizeAuthRedirectPath('/settings/profile')).toBe('/settings/profile');
    expect(normalizeAuthRedirectPath('/admin')).toBe('/admin');
  });
});

describe('admin role checks', () => {
  it('accepts valid admin roles', () => {
    expect(isAdminRole('system_admin')).toBe(true);
    expect(isAdminRole('org_admin')).toBe(true);
    expect(isAdminRole('admin')).toBe(true);
    expect(isAdminRole('administrator')).toBe(true);
  });

  it('rejects non-admin roles', () => {
    expect(isAdminRole('team_admin')).toBe(false);
    expect(isAdminRole('member')).toBe(false);
    expect(isAdminRole(undefined)).toBe(false);
  });
});
