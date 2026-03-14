const ADMIN_ROLES = new Set(['system_admin', 'org_admin', 'admin', 'administrator']);

function parseRoles(role: string): string[] {
  return role
    .split(/[\s,]+/)
    .map((item) => item.trim().toLowerCase())
    .filter((item) => item.length > 0);
}

export function isAdminRole(role: string | null | undefined): boolean {
  if (!role) return false;
  return parseRoles(role).some((item) => ADMIN_ROLES.has(item));
}

export default isAdminRole;
