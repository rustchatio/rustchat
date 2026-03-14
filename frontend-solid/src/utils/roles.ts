const ADMIN_ROLES = new Set(['system_admin', 'org_admin', 'admin', 'administrator']);

export function isAdminRole(role: string | null | undefined): boolean {
  if (!role) return false;
  return ADMIN_ROLES.has(role);
}

export default isAdminRole;
