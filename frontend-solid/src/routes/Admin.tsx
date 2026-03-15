import { For, Show, createEffect, createMemo, createSignal } from 'solid-js';
import { A, useLocation, useNavigate } from '@solidjs/router';
import { authStore } from '@/stores/auth';
import { isAdminRole } from '@/utils/roles';

interface AdminSection {
  id: string;
  label: string;
  description: string;
}

interface AdminStats {
  total_users: number;
  active_users: number;
  total_teams: number;
  total_channels: number;
  messages_24h: number;
  files_count: number;
  storage_used_mb: number;
}

interface AdminHealth {
  status: string;
  database: {
    connected: boolean;
    latency_ms: number;
  };
  storage: {
    connected: boolean;
    storage_type?: string;
    type?: string;
  };
  websocket: {
    active_connections: number;
  };
  version: string;
  uptime_seconds: number;
}

interface AdminUser {
  id: string;
  username: string;
  email: string;
  display_name?: string | null;
  role: string;
  is_active: boolean;
  deleted_at?: string | null;
  last_login_at?: string | null;
  created_at: string;
}

interface AdminUsersResponse {
  users: AdminUser[];
  total: number;
}

interface AdminTeam {
  id: string;
  name: string;
  display_name?: string | null;
  is_public: boolean;
  created_at: string;
  members_count: number;
  channels_count: number;
}

interface AdminTeamsResponse {
  teams: AdminTeam[];
  total: number;
}

interface AdminTeamMember {
  team_id: string;
  user_id: string;
  role: string;
  username: string;
  display_name?: string | null;
}

interface AdminChannel {
  id: string;
  team_id: string;
  channel_type: 'public' | 'private' | 'direct' | 'group';
  name: string;
  display_name?: string | null;
  members_count: number;
}

interface AdminChannelsResponse {
  channels: AdminChannel[];
  total: number;
}

interface AdminConfig {
  site: Record<string, unknown>;
  authentication: Record<string, unknown>;
  compliance: Record<string, unknown>;
  integrations: Record<string, unknown>;
  experimental: Record<string, unknown>;
}

interface AdminCallsPluginSettings {
  enabled: boolean;
  turn_server_enabled: boolean;
  turn_server_url: string;
  turn_server_username: string;
  turn_server_credential?: string | null;
  udp_port: number;
  tcp_port: number;
  ice_host_override?: string | null;
  stun_servers: string[];
}

interface AdminCallsPluginConfig {
  plugin_id: string;
  plugin_name: string;
  settings: AdminCallsPluginSettings;
}

interface AdminSsoConfig {
  id: string;
  provider_key: string;
  provider_type: string;
  display_name?: string | null;
  is_active: boolean;
  auto_provision: boolean;
}

interface AdminPermission {
  id: string;
  description?: string | null;
  category?: string | null;
}

interface AdminAuditLog {
  id: string;
  actor_user_id?: string | null;
  action: string;
  target_type: string;
  target_id?: string | null;
  created_at: string;
}

interface AdminMembershipPolicyTarget {
  id: string;
  policy_id: string;
  target_type: 'team' | 'channel';
  target_id: string;
  role_mode: 'member' | 'admin';
  created_at: string;
}

interface AdminMembershipPolicy {
  id: string;
  name: string;
  description?: string | null;
  scope_type: 'global' | 'team';
  team_id?: string | null;
  source_type: 'all_users' | 'auth_service' | 'group' | 'role' | 'org';
  source_config?: unknown;
  enabled: boolean;
  priority: number;
  created_at: string;
  updated_at: string;
  targets: AdminMembershipPolicyTarget[];
}

interface AdminMembershipAuditSummary {
  total_operations_24h: number;
  successful_operations_24h: number;
  failed_operations_24h: number;
  failure_rate_24h: number;
  pending_operations: number;
  policies_with_failures: number;
}

interface AdminEmailProvider {
  id: string;
  tenant_id?: string | null;
  provider_type: string;
  host: string;
  port: number;
  username: string;
  has_password?: boolean;
  tls_mode?: string;
  skip_cert_verify?: boolean;
  from_address: string;
  from_name: string;
  reply_to?: string | null;
  max_emails_per_minute?: number;
  max_emails_per_hour?: number;
  enabled: boolean;
  is_default: boolean;
  updated_at: string;
}

interface AdminEmailOutboxEntry {
  id: string;
  workflow_key?: string | null;
  recipient_email: string;
  subject: string;
  status: string;
  priority: string;
  attempt_count: number;
  max_attempts: number;
  sent_at?: string | null;
  created_at: string;
}

interface AdminEmailWorkflow {
  id: string;
  workflow_key: string;
  name: string;
  description?: string | null;
  category: string;
  enabled: boolean;
  system_required: boolean;
  can_disable: boolean;
  default_locale: string;
  selected_template_family_id?: string | null;
  policy?: Record<string, unknown>;
}

interface AdminEmailEvent {
  id: string;
  outbox_id?: string | null;
  workflow_key?: string | null;
  event_type: string;
  recipient_email: string;
  status_code?: number | null;
  error_category?: string | null;
  error_message?: string | null;
  created_at: string;
}

interface AdminEmailTemplateVariable {
  name: string;
  required: boolean;
  default_value?: string | null;
  description?: string | null;
}

interface AdminEmailTemplateFamily {
  id: string;
  tenant_id?: string | null;
  key: string;
  name: string;
  description?: string | null;
  workflow_key?: string | null;
  is_system: boolean;
  created_at: string;
  updated_at: string;
}

interface AdminEmailTemplateVersion {
  id: string;
  family_id: string;
  version: number;
  status: 'draft' | 'published' | 'archived' | string;
  locale: string;
  subject: string;
  body_text?: string | null;
  body_html?: string | null;
  variables: AdminEmailTemplateVariable[];
  is_compiled_from_mjml: boolean;
  created_at: string;
  published_at?: string | null;
}

interface AdminEmailTestResult {
  status?: string;
  success?: boolean;
  message?: string;
  error?: string;
  outbox_id?: string;
}

interface MembershipPolicyMetadata {
  source_types: Array<{ value: string; label: string }>;
  scope_types: Array<{ value: string; label: string }>;
  target_types: Array<{ value: string; label: string }>;
  role_modes: Array<{ value: string; label: string }>;
}

interface MembershipPolicyTargetDraft {
  target_type: 'team' | 'channel';
  target_id: string;
  role_mode: 'member' | 'admin';
}

const sections: AdminSection[] = [
  { id: '', label: 'Overview', description: 'System summary and quick links' },
  { id: 'audit-dashboard', label: 'Audit Dashboard', description: 'Operational audit metrics and summaries' },
  { id: 'users', label: 'Users', description: 'Manage users and account lifecycle' },
  { id: 'teams', label: 'Teams', description: 'Manage teams and channels' },
  { id: 'settings', label: 'Server Settings', description: 'Server and platform configuration' },
  { id: 'security', label: 'Security', description: 'Authentication and access controls' },
  { id: 'permissions', label: 'Permissions', description: 'Role-based permission assignments and inventory' },
  { id: 'sso', label: 'SSO', description: 'Single sign-on provider configuration and status' },
  { id: 'integrations', label: 'Integrations', description: 'Integration-related platform configuration' },
  { id: 'email', label: 'Email', description: 'Email providers and recent outbox activity' },
  { id: 'membership-policies', label: 'Membership Policies', description: 'Auto-membership policy controls and audit health' },
  { id: 'compliance', label: 'Compliance', description: 'Retention, exports, and compliance controls' },
  { id: 'audit', label: 'Audit Logs', description: 'Administrative events and audit trail' },
  { id: 'health', label: 'System Health', description: 'Service health and infrastructure status' },
];

function formatDateTime(dateValue?: string | null): string {
  if (!dateValue) return 'Never';
  const parsed = new Date(dateValue);
  if (Number.isNaN(parsed.getTime())) return 'Never';
  return parsed.toLocaleString();
}

function isValidEmail(value: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value.trim());
}

function parseRoleTokens(roleValue: string | null | undefined): string[] {
  if (!roleValue) return [];
  return roleValue
    .split(/[\s,]+/)
    .map((token) => token.trim().toLowerCase())
    .filter((token) => token.length > 0);
}

async function parseApiError(response: Response): Promise<Error> {
  let message = `Request failed (${response.status})`;

  try {
    const payload = (await response.json()) as {
      message?: string;
      error?: string;
      detailed_error?: string;
    };
    message = payload.message || payload.error || payload.detailed_error || message;
  } catch {
    // Keep fallback message when response is not JSON.
  }

  return new Error(message);
}

async function fetchAdminJson<T>(
  token: string,
  path: string,
  init: RequestInit = {}
): Promise<T> {
  return fetchAdminScopedJson<T>(token, '/api/v1', path, init);
}

async function fetchAdminScopedJson<T>(
  token: string,
  basePath: '/api/v1' | '/api/v4',
  path: string,
  init: RequestInit = {}
): Promise<T> {
  const headers = new Headers(init.headers);
  headers.set('Authorization', `Bearer ${token}`);
  if (!(init.body instanceof FormData)) {
    headers.set('Content-Type', 'application/json');
  }

  const response = await fetch(`${basePath}${path}`, {
    ...init,
    headers,
  });

  if (!response.ok) {
    throw await parseApiError(response);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return (await response.json()) as T;
}

function AdminOverviewSection() {
  const [stats, setStats] = createSignal<AdminStats | null>(null);
  const [health, setHealth] = createSignal<AdminHealth | null>(null);
  const [isLoading, setIsLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);

  const load = async () => {
    const token = authStore.token;
    if (!token) return;

    setIsLoading(true);
    setError(null);

    try {
      const [statsData, healthData] = await Promise.all([
        fetchAdminJson<AdminStats>(token, '/admin/stats'),
        fetchAdminJson<AdminHealth>(token, '/admin/health'),
      ]);
      setStats(statsData);
      setHealth(healthData);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load admin overview');
    } finally {
      setIsLoading(false);
    }
  };

  createEffect(() => {
    if (authStore.isAuthenticated) {
      void load();
    }
  });

  return (
    <div class="space-y-4">
      <Show when={error()}>
        <div class="rounded-lg border border-danger/30 bg-danger/10 px-4 py-3 text-sm text-danger">
          {error()}
        </div>
      </Show>

      <Show when={!isLoading()} fallback={<p class="text-sm text-text-3">Loading overview...</p>}>
        <div class="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-4">
          <OverviewCard label="Total Users" value={String(stats()?.total_users ?? 0)} />
          <OverviewCard label="Active Users" value={String(stats()?.active_users ?? 0)} />
          <OverviewCard label="Total Teams" value={String(stats()?.total_teams ?? 0)} />
          <OverviewCard label="Total Channels" value={String(stats()?.total_channels ?? 0)} />
        </div>

        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4 space-y-2">
          <h3 class="text-sm font-semibold text-text-1">System Health</h3>
          <p class="text-sm text-text-2">Status: <span class="font-medium">{health()?.status || 'unknown'}</span></p>
          <p class="text-sm text-text-2">
            Database: {health()?.database.connected ? 'Connected' : 'Disconnected'}
            {' · '}
            {health()?.database.latency_ms ?? 0}ms
          </p>
          <p class="text-sm text-text-2">WebSocket Connections: {health()?.websocket.active_connections ?? 0}</p>
          <p class="text-sm text-text-2">Version: {health()?.version || '-'}</p>
        </div>
      </Show>
    </div>
  );
}

function OverviewCard(props: { label: string; value: string }) {
  return (
    <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4">
      <p class="text-xs uppercase tracking-wide text-text-3">{props.label}</p>
      <p class="mt-2 text-2xl font-semibold text-text-1">{props.value}</p>
    </div>
  );
}

function AdminUsersSection() {
  const [users, setUsers] = createSignal<AdminUser[]>([]);
  const [total, setTotal] = createSignal(0);
  const [isLoading, setIsLoading] = createSignal(true);
  const [isMutatingUserId, setIsMutatingUserId] = createSignal<string | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [notice, setNotice] = createSignal<string | null>(null);
  const [search, setSearch] = createSignal('');
  const [status, setStatus] = createSignal<'all' | 'active' | 'inactive'>('all');
  const [includeDeleted, setIncludeDeleted] = createSignal(false);
  const canDeleteUsers = createMemo(() =>
    parseRoleTokens(authStore.user()?.role).includes('system_admin')
  );

  const loadUsers = async () => {
    const token = authStore.token;
    if (!token) return;

    setIsLoading(true);
    setError(null);
    setNotice(null);

    try {
      const params = new URLSearchParams({
        page: '1',
        per_page: '50',
      });

      if (search().trim()) {
        params.set('search', search().trim());
      }
      if (status() !== 'all') {
        params.set('status', status());
      }
      if (includeDeleted()) {
        params.set('include_deleted', 'true');
      }

      const response = await fetchAdminJson<AdminUsersResponse>(
        token,
        `/admin/users?${params.toString()}`
      );
      setUsers(response.users || []);
      setTotal(response.total || 0);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load users');
    } finally {
      setIsLoading(false);
    }
  };

  createEffect(() => {
    if (authStore.isAuthenticated) {
      void loadUsers();
    }
  });

  const toggleUserActive = async (user: AdminUser) => {
    const token = authStore.token;
    if (!token) return;

    setIsMutatingUserId(user.id);
    setError(null);
    setNotice(null);
    try {
      const endpoint = user.is_active
        ? `/admin/users/${user.id}/deactivate`
        : `/admin/users/${user.id}/reactivate`;
      await fetchAdminJson<{ status: string }>(token, endpoint, { method: 'POST' });
      setNotice(
        `${user.display_name || user.username} ${user.is_active ? 'deactivated' : 'reactivated'}.`
      );
      await loadUsers();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update user status');
    } finally {
      setIsMutatingUserId(null);
    }
  };

  const deleteUser = async (user: AdminUser) => {
    const token = authStore.token;
    if (!token) return;
    if (!canDeleteUsers()) {
      setError('Only system_admin can delete or wipe users.');
      return;
    }

    const confirmValue = window.prompt(
      `Type the username or email to delete this user:\n${user.username} / ${user.email}`,
      user.username
    );
    if (!confirmValue) return;
    const reason = window.prompt('Optional deletion reason:', 'Admin requested account removal');

    setIsMutatingUserId(user.id);
    setError(null);
    setNotice(null);
    try {
      await fetchAdminJson<{ status: string }>(token, `/admin/users/${user.id}`, {
        method: 'DELETE',
        body: JSON.stringify({
          confirm: confirmValue.trim(),
          reason: reason?.trim() || undefined,
        }),
      });
      setNotice(`${user.display_name || user.username} soft-deleted.`);
      await loadUsers();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete user');
    } finally {
      setIsMutatingUserId(null);
    }
  };

  const wipeUser = async (user: AdminUser) => {
    const token = authStore.token;
    if (!token) return;
    if (!canDeleteUsers()) {
      setError('Only system_admin can delete or wipe users.');
      return;
    }

    const confirmed = window.confirm(
      `Permanently wipe ${user.display_name || user.username}? This cannot be undone.`
    );
    if (!confirmed) return;

    setIsMutatingUserId(user.id);
    setError(null);
    setNotice(null);
    try {
      await fetchAdminJson<{ status: string; message?: string }>(token, `/admin/users/${user.id}/wipe`, {
        method: 'POST',
      });
      setNotice(`${user.display_name || user.username} permanently wiped.`);
      await loadUsers();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to wipe user');
    } finally {
      setIsMutatingUserId(null);
    }
  };

  return (
    <div class="space-y-4">
      <div class="flex flex-col gap-3 md:flex-row md:items-center">
        <input
          type="text"
          value={search()}
          onInput={(event) => setSearch(event.currentTarget.value)}
          placeholder="Search by username or email"
          class="w-full md:w-80 rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
        />
        <select
          value={status()}
          onChange={(event) => setStatus(event.currentTarget.value as 'all' | 'active' | 'inactive')}
          class="rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
        >
          <option value="all">All Users</option>
          <option value="active">Active</option>
          <option value="inactive">Inactive</option>
        </select>
        <label class="inline-flex items-center gap-2 text-sm text-text-2">
          <input
            type="checkbox"
            checked={includeDeleted()}
            onChange={(event) => setIncludeDeleted(event.currentTarget.checked)}
            class="h-4 w-4 rounded border-border-1 bg-bg-app"
          />
          Include deleted
        </label>
        <button
          type="button"
          class="rounded-lg bg-brand px-3 py-2 text-sm font-medium text-white hover:bg-brand-hover"
          onClick={() => {
            void loadUsers();
          }}
        >
          Apply
        </button>
        <p class="text-sm text-text-3 md:ml-auto">{total()} users</p>
      </div>

      <Show when={error()}>
        <div class="rounded-lg border border-danger/30 bg-danger/10 px-4 py-3 text-sm text-danger">
          {error()}
        </div>
      </Show>
      <Show when={notice()}>
        <div class="rounded-lg border border-success/30 bg-success/10 px-4 py-3 text-sm text-success">
          {notice()}
        </div>
      </Show>

      <div class="overflow-x-auto rounded-xl border border-border-1 bg-bg-surface-1">
        <table class="w-full text-left text-sm">
          <thead class="border-b border-border-1 bg-bg-surface-2 text-text-3">
            <tr>
              <th class="px-4 py-3 font-medium">User</th>
              <th class="px-4 py-3 font-medium">Role</th>
              <th class="px-4 py-3 font-medium">Status</th>
              <th class="px-4 py-3 font-medium">Last Login</th>
              <th class="px-4 py-3 font-medium text-right">Action</th>
            </tr>
          </thead>
          <tbody>
            <Show when={!isLoading()} fallback={<tr><td class="px-4 py-4 text-text-3" colSpan={5}>Loading users...</td></tr>}>
              <Show when={users().length > 0} fallback={<tr><td class="px-4 py-4 text-text-3" colSpan={5}>No users found.</td></tr>}>
                <For each={users()}>
                  {(user) => (
                    <tr class="border-b border-border-1/60 last:border-b-0">
                      <td class="px-4 py-3">
                        <p class="font-medium text-text-1">{user.display_name || user.username}</p>
                        <p class="text-xs text-text-3">{user.email}</p>
                      </td>
                      <td class="px-4 py-3 text-text-2">{user.role}</td>
                      <td class="px-4 py-3">
                        <Show
                          when={!user.deleted_at}
                          fallback={
                            <span class="rounded-full bg-danger/15 px-2 py-1 text-xs font-medium text-danger">
                              Deleted
                            </span>
                          }
                        >
                          <span class={`rounded-full px-2 py-1 text-xs font-medium ${user.is_active ? 'bg-success/15 text-success' : 'bg-warning/20 text-warning'}`}>
                            {user.is_active ? 'Active' : 'Inactive'}
                          </span>
                        </Show>
                      </td>
                      <td class="px-4 py-3 text-text-2">{formatDateTime(user.last_login_at)}</td>
                      <td class="px-4 py-3 text-right">
                        <div class="inline-flex items-center gap-2">
                          <Show when={!user.deleted_at}>
                            <button
                              type="button"
                              class={`rounded-lg px-3 py-1.5 text-xs font-medium ${user.is_active ? 'bg-warning/20 text-warning hover:bg-warning/30' : 'bg-success/20 text-success hover:bg-success/30'} disabled:opacity-60`}
                              onClick={() => {
                                void toggleUserActive(user);
                              }}
                              disabled={isMutatingUserId() === user.id}
                            >
                              {isMutatingUserId() === user.id
                                ? 'Working...'
                                : user.is_active
                                  ? 'Deactivate'
                                  : 'Reactivate'}
                            </button>
                          </Show>

                          <Show when={!user.deleted_at}>
                            <button
                              type="button"
                              class="rounded-lg border border-danger/40 px-3 py-1.5 text-xs font-medium text-danger hover:bg-danger/10 disabled:opacity-60"
                              onClick={() => {
                                void deleteUser(user);
                              }}
                              disabled={isMutatingUserId() === user.id || !canDeleteUsers()}
                              title={canDeleteUsers() ? 'Soft delete user' : 'Requires system_admin role'}
                            >
                              {isMutatingUserId() === user.id ? 'Working...' : 'Delete'}
                            </button>
                          </Show>

                          <Show when={!!user.deleted_at}>
                            <button
                              type="button"
                              class="rounded-lg border border-danger px-3 py-1.5 text-xs font-medium text-danger hover:bg-danger/10 disabled:opacity-60"
                              onClick={() => {
                                void wipeUser(user);
                              }}
                              disabled={isMutatingUserId() === user.id || !canDeleteUsers()}
                              title={canDeleteUsers() ? 'Permanently wipe user' : 'Requires system_admin role'}
                            >
                              {isMutatingUserId() === user.id ? 'Working...' : 'Wipe'}
                            </button>
                          </Show>
                        </div>
                      </td>
                    </tr>
                  )}
                </For>
              </Show>
            </Show>
          </tbody>
        </table>
      </div>
    </div>
  );
}

function AdminTeamsSection() {
  const [teams, setTeams] = createSignal<AdminTeam[]>([]);
  const [total, setTotal] = createSignal(0);
  const [selectedTeam, setSelectedTeam] = createSignal<AdminTeam | null>(null);
  const [members, setMembers] = createSignal<AdminTeamMember[]>([]);
  const [channels, setChannels] = createSignal<AdminChannel[]>([]);
  const [isLoading, setIsLoading] = createSignal(true);
  const [detailsLoading, setDetailsLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [search, setSearch] = createSignal('');

  const loadTeams = async () => {
    const token = authStore.token;
    if (!token) return;

    setIsLoading(true);
    setError(null);

    try {
      const params = new URLSearchParams({
        page: '1',
        per_page: '50',
      });

      if (search().trim()) {
        params.set('search', search().trim());
      }

      const response = await fetchAdminJson<AdminTeamsResponse>(
        token,
        `/admin/teams?${params.toString()}`
      );
      setTeams(response.teams || []);
      setTotal(response.total || 0);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load teams');
    } finally {
      setIsLoading(false);
    }
  };

  const loadTeamDetails = async (team: AdminTeam) => {
    const token = authStore.token;
    if (!token) return;

    setDetailsLoading(true);
    setError(null);
    setSelectedTeam(team);

    try {
      const [membersData, channelsData] = await Promise.all([
        fetchAdminJson<AdminTeamMember[]>(token, `/admin/teams/${team.id}/members`),
        fetchAdminJson<AdminChannelsResponse>(token, `/admin/channels?team_id=${team.id}&per_page=100`),
      ]);
      setMembers(membersData || []);
      setChannels(channelsData.channels || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load team details');
      setMembers([]);
      setChannels([]);
    } finally {
      setDetailsLoading(false);
    }
  };

  createEffect(() => {
    if (authStore.isAuthenticated) {
      void loadTeams();
    }
  });

  return (
    <div class="space-y-4">
      <div class="flex flex-col gap-3 md:flex-row md:items-center">
        <input
          type="text"
          value={search()}
          onInput={(event) => setSearch(event.currentTarget.value)}
          placeholder="Search teams"
          class="w-full md:w-80 rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
        />
        <button
          type="button"
          class="rounded-lg bg-brand px-3 py-2 text-sm font-medium text-white hover:bg-brand-hover"
          onClick={() => {
            void loadTeams();
          }}
        >
          Apply
        </button>
        <p class="text-sm text-text-3 md:ml-auto">{total()} teams</p>
      </div>

      <Show when={error()}>
        <div class="rounded-lg border border-danger/30 bg-danger/10 px-4 py-3 text-sm text-danger">
          {error()}
        </div>
      </Show>

      <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <div class="rounded-xl border border-border-1 bg-bg-surface-1 overflow-hidden">
          <div class="border-b border-border-1 px-4 py-3 text-sm font-semibold text-text-1">Teams</div>
          <div class="max-h-[420px] overflow-y-auto">
            <Show when={!isLoading()} fallback={<p class="px-4 py-4 text-sm text-text-3">Loading teams...</p>}>
              <Show when={teams().length > 0} fallback={<p class="px-4 py-4 text-sm text-text-3">No teams found.</p>}>
                <For each={teams()}>
                  {(team) => (
                    <button
                      type="button"
                      class={`w-full border-b border-border-1/60 px-4 py-3 text-left transition-colors hover:bg-bg-surface-2 ${selectedTeam()?.id === team.id ? 'bg-brand/10' : ''}`}
                      onClick={() => {
                        void loadTeamDetails(team);
                      }}
                    >
                      <p class="font-medium text-text-1">{team.display_name || team.name}</p>
                      <p class="text-xs text-text-3">
                        {team.members_count} members · {team.channels_count} channels · {team.is_public ? 'Public' : 'Private'}
                      </p>
                    </button>
                  )}
                </For>
              </Show>
            </Show>
          </div>
        </div>

        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4">
          <Show when={selectedTeam()} fallback={<p class="text-sm text-text-3">Select a team to inspect channels and members.</p>}>
            {(team) => (
              <div class="space-y-4">
                <div>
                  <h3 class="text-base font-semibold text-text-1">{team().display_name || team().name}</h3>
                  <p class="text-sm text-text-3">Created: {formatDateTime(team().created_at)}</p>
                </div>

                <Show when={!detailsLoading()} fallback={<p class="text-sm text-text-3">Loading team details...</p>}>
                  <div class="grid grid-cols-1 gap-4">
                    <div>
                      <h4 class="text-sm font-semibold text-text-1">Channels ({channels().length})</h4>
                      <div class="mt-2 max-h-36 overflow-y-auto rounded-lg border border-border-1">
                        <Show when={channels().length > 0} fallback={<p class="px-3 py-2 text-xs text-text-3">No channels.</p>}>
                          <For each={channels()}>
                            {(channel) => (
                              <div class="border-b border-border-1/50 px-3 py-2 text-xs last:border-b-0">
                                <p class="font-medium text-text-1">{channel.display_name || channel.name}</p>
                                <p class="text-text-3">#{channel.name} · {channel.channel_type} · {channel.members_count} members</p>
                              </div>
                            )}
                          </For>
                        </Show>
                      </div>
                    </div>

                    <div>
                      <h4 class="text-sm font-semibold text-text-1">Members ({members().length})</h4>
                      <div class="mt-2 max-h-36 overflow-y-auto rounded-lg border border-border-1">
                        <Show when={members().length > 0} fallback={<p class="px-3 py-2 text-xs text-text-3">No members.</p>}>
                          <For each={members()}>
                            {(member) => (
                              <div class="border-b border-border-1/50 px-3 py-2 text-xs last:border-b-0">
                                <p class="font-medium text-text-1">{member.display_name || member.username}</p>
                                <p class="text-text-3">@{member.username} · {member.role}</p>
                              </div>
                            )}
                          </For>
                        </Show>
                      </div>
                    </div>
                  </div>
                </Show>
              </div>
            )}
          </Show>
        </div>
      </div>
    </div>
  );
}

function AdminSettingsSection() {
  const [config, setConfig] = createSignal<AdminConfig | null>(null);
  const [siteName, setSiteName] = createSignal('');
  const [siteUrl, setSiteUrl] = createSignal('');
  const [supportEmail, setSupportEmail] = createSignal('');
  const [allowRegistration, setAllowRegistration] = createSignal(false);
  const [hasCallsPlugin, setHasCallsPlugin] = createSignal(false);
  const [callsPluginName, setCallsPluginName] = createSignal('RustChat Calls Plugin');
  const [callsEnabled, setCallsEnabled] = createSignal(false);
  const [turnEnabled, setTurnEnabled] = createSignal(false);
  const [turnServerUrl, setTurnServerUrl] = createSignal('');
  const [turnServerUsername, setTurnServerUsername] = createSignal('');
  const [turnServerCredential, setTurnServerCredential] = createSignal('');
  const [udpPort, setUdpPort] = createSignal(3478);
  const [tcpPort, setTcpPort] = createSignal(3478);
  const [iceHostOverride, setIceHostOverride] = createSignal('');
  const [stunServersText, setStunServersText] = createSignal('');
  const [isLoading, setIsLoading] = createSignal(true);
  const [isSaving, setIsSaving] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [notice, setNotice] = createSignal<string | null>(null);

  const loadConfig = async () => {
    const token = authStore.token;
    if (!token) return;

    setIsLoading(true);
    setError(null);
    setNotice(null);

    try {
      const [data, callsPlugin] = await Promise.all([
        fetchAdminJson<AdminConfig>(token, '/admin/config'),
        fetchAdminJson<AdminCallsPluginConfig>(token, '/admin/plugins/calls').catch(() => null),
      ]);
      setConfig(data);
      setSiteName(String(data.site?.site_name || ''));
      setSiteUrl(String(data.site?.site_url || ''));
      setSupportEmail(String(data.site?.support_email || ''));
      setAllowRegistration(Boolean(data.authentication?.allow_registration));
      if (callsPlugin) {
        setHasCallsPlugin(true);
        setCallsPluginName(callsPlugin.plugin_name || 'RustChat Calls Plugin');
        setCallsEnabled(Boolean(callsPlugin.settings.enabled));
        setTurnEnabled(Boolean(callsPlugin.settings.turn_server_enabled));
        setTurnServerUrl(callsPlugin.settings.turn_server_url || '');
        setTurnServerUsername(callsPlugin.settings.turn_server_username || '');
        setTurnServerCredential('');
        setUdpPort(Number(callsPlugin.settings.udp_port || 3478));
        setTcpPort(Number(callsPlugin.settings.tcp_port || 3478));
        setIceHostOverride(callsPlugin.settings.ice_host_override || '');
        setStunServersText((callsPlugin.settings.stun_servers || []).join('\n'));
      } else {
        setHasCallsPlugin(false);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load server config');
    } finally {
      setIsLoading(false);
    }
  };

  createEffect(() => {
    if (authStore.isAuthenticated) {
      void loadConfig();
    }
  });

  const saveConfig = async () => {
    const token = authStore.token;
    const current = config();
    if (!token || !current) return;

    setIsSaving(true);
    setError(null);
    setNotice(null);

    try {
      const nextSite = {
        ...current.site,
        site_name: siteName(),
        site_url: siteUrl(),
        support_email: supportEmail(),
      };
      const nextAuth = {
        ...current.authentication,
        allow_registration: allowRegistration(),
      };

      await fetchAdminJson<Record<string, unknown>>(token, '/admin/config/site', {
        method: 'PATCH',
        body: JSON.stringify(nextSite),
      });

      await fetchAdminJson<Record<string, unknown>>(token, '/admin/config/authentication', {
        method: 'PATCH',
        body: JSON.stringify(nextAuth),
      });

      if (hasCallsPlugin()) {
        const normalizedStunServers = stunServersText()
          .split(/[\n,]+/)
          .map((entry) => entry.trim())
          .filter((entry) => entry.length > 0);
        const pluginPayload: Record<string, unknown> = {
          enabled: callsEnabled(),
          turn_server_enabled: turnEnabled(),
          turn_server_url: turnServerUrl().trim(),
          turn_server_username: turnServerUsername().trim(),
          udp_port: Math.max(1, Math.floor(udpPort())),
          tcp_port: Math.max(1, Math.floor(tcpPort())),
          ice_host_override: iceHostOverride().trim() || null,
          stun_servers: normalizedStunServers,
        };
        const normalizedCredential = turnServerCredential().trim();
        if (normalizedCredential) {
          pluginPayload.turn_server_credential = normalizedCredential;
        }
        await fetchAdminJson<Record<string, unknown>>(token, '/admin/plugins/calls', {
          method: 'PUT',
          body: JSON.stringify(pluginPayload),
        });
      }

      setConfig({
        ...current,
        site: nextSite,
        authentication: nextAuth,
      });
      setNotice('Configuration saved.');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save configuration');
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div class="space-y-4">
      <Show when={error()}>
        <div class="rounded-lg border border-danger/30 bg-danger/10 px-4 py-3 text-sm text-danger">
          {error()}
        </div>
      </Show>

      <Show when={notice()}>
        <div class="rounded-lg border border-success/30 bg-success/10 px-4 py-3 text-sm text-success">
          {notice()}
        </div>
      </Show>

      <Show when={!isLoading()} fallback={<p class="text-sm text-text-3">Loading server settings...</p>}>
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
          <label class="space-y-1">
            <span class="text-sm font-medium text-text-2">Site Name</span>
            <input
              type="text"
              value={siteName()}
              onInput={(event) => setSiteName(event.currentTarget.value)}
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
            />
          </label>

          <label class="space-y-1">
            <span class="text-sm font-medium text-text-2">Site URL</span>
            <input
              type="url"
              value={siteUrl()}
              onInput={(event) => setSiteUrl(event.currentTarget.value)}
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
            />
          </label>

          <label class="space-y-1 md:col-span-2">
            <span class="text-sm font-medium text-text-2">Support Email</span>
            <input
              type="email"
              value={supportEmail()}
              onInput={(event) => setSupportEmail(event.currentTarget.value)}
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
            />
          </label>

          <label class="inline-flex items-center gap-2 md:col-span-2">
            <input
              type="checkbox"
              checked={allowRegistration()}
              onChange={(event) => setAllowRegistration(event.currentTarget.checked)}
              class="h-4 w-4 rounded border-border-1"
            />
            <span class="text-sm text-text-2">Allow user self-registration</span>
          </label>
        </div>

        <Show when={hasCallsPlugin()}>
          <div class="rounded-lg border border-border-1 bg-bg-app p-4 space-y-4">
            <div>
              <h4 class="text-sm font-semibold text-text-1">{callsPluginName()}</h4>
              <p class="text-xs text-text-3 mt-1">Calls plugin and TURN/STUN configuration</p>
            </div>

            <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
              <label class="inline-flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={callsEnabled()}
                  onChange={(event) => setCallsEnabled(event.currentTarget.checked)}
                  class="h-4 w-4 rounded border-border-1"
                />
                <span class="text-sm text-text-2">Enable calls plugin</span>
              </label>

              <label class="inline-flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={turnEnabled()}
                  onChange={(event) => setTurnEnabled(event.currentTarget.checked)}
                  class="h-4 w-4 rounded border-border-1"
                />
                <span class="text-sm text-text-2">Enable TURN server</span>
              </label>

              <label class="space-y-1">
                <span class="text-sm font-medium text-text-2">TURN Server URL</span>
                <input
                  type="text"
                  value={turnServerUrl()}
                  onInput={(event) => setTurnServerUrl(event.currentTarget.value)}
                  class="w-full rounded-lg border border-border-1 bg-bg-surface-1 px-3 py-2 text-sm text-text-1"
                />
              </label>

              <label class="space-y-1">
                <span class="text-sm font-medium text-text-2">TURN Username</span>
                <input
                  type="text"
                  value={turnServerUsername()}
                  onInput={(event) => setTurnServerUsername(event.currentTarget.value)}
                  class="w-full rounded-lg border border-border-1 bg-bg-surface-1 px-3 py-2 text-sm text-text-1"
                />
              </label>

              <label class="space-y-1">
                <span class="text-sm font-medium text-text-2">TURN Credential</span>
                <input
                  type="password"
                  value={turnServerCredential()}
                  onInput={(event) => setTurnServerCredential(event.currentTarget.value)}
                  placeholder="Leave blank to keep existing value"
                  class="w-full rounded-lg border border-border-1 bg-bg-surface-1 px-3 py-2 text-sm text-text-1"
                />
              </label>

              <label class="space-y-1">
                <span class="text-sm font-medium text-text-2">ICE Host Override</span>
                <input
                  type="text"
                  value={iceHostOverride()}
                  onInput={(event) => setIceHostOverride(event.currentTarget.value)}
                  class="w-full rounded-lg border border-border-1 bg-bg-surface-1 px-3 py-2 text-sm text-text-1"
                />
              </label>

              <label class="space-y-1">
                <span class="text-sm font-medium text-text-2">UDP Port</span>
                <input
                  type="number"
                  min="1"
                  value={udpPort()}
                  onInput={(event) => setUdpPort(Number(event.currentTarget.value))}
                  class="w-full rounded-lg border border-border-1 bg-bg-surface-1 px-3 py-2 text-sm text-text-1"
                />
              </label>

              <label class="space-y-1">
                <span class="text-sm font-medium text-text-2">TCP Port</span>
                <input
                  type="number"
                  min="1"
                  value={tcpPort()}
                  onInput={(event) => setTcpPort(Number(event.currentTarget.value))}
                  class="w-full rounded-lg border border-border-1 bg-bg-surface-1 px-3 py-2 text-sm text-text-1"
                />
              </label>

              <label class="space-y-1 md:col-span-2">
                <span class="text-sm font-medium text-text-2">STUN Servers</span>
                <textarea
                  rows={3}
                  value={stunServersText()}
                  onInput={(event) => setStunServersText(event.currentTarget.value)}
                  placeholder="One server per line (or comma-separated)"
                  class="w-full rounded-lg border border-border-1 bg-bg-surface-1 px-3 py-2 text-sm text-text-1"
                />
              </label>
            </div>
          </div>
        </Show>

        <div class="flex items-center gap-3">
          <button
            type="button"
            class="rounded-lg bg-brand px-4 py-2 text-sm font-medium text-white hover:bg-brand-hover disabled:opacity-60"
            onClick={() => {
              void saveConfig();
            }}
            disabled={isSaving()}
          >
            {isSaving() ? 'Saving...' : 'Save Changes'}
          </button>
          <button
            type="button"
            class="rounded-lg border border-border-1 px-4 py-2 text-sm font-medium text-text-2 hover:bg-bg-surface-2"
            onClick={() => {
              void loadConfig();
            }}
            disabled={isSaving()}
          >
            Reload
          </button>
        </div>
      </Show>
    </div>
  );
}

function AdminSecuritySection() {
  const [ssoConfigs, setSsoConfigs] = createSignal<AdminSsoConfig[]>([]);
  const [permissions, setPermissions] = createSignal<AdminPermission[]>([]);
  const [systemAdminPermissions, setSystemAdminPermissions] = createSignal<string[]>([]);
  const [roleOptions, setRoleOptions] = createSignal<string[]>([]);
  const [roleUserCounts, setRoleUserCounts] = createSignal<Record<string, number>>({});
  const [selectedRole, setSelectedRole] = createSignal('system_admin');
  const [rolePermissions, setRolePermissions] = createSignal<string[]>([]);
  const [isLoading, setIsLoading] = createSignal(true);
  const [isLoadingRolePermissions, setIsLoadingRolePermissions] = createSignal(false);
  const [isSavingRolePermissions, setIsSavingRolePermissions] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [notice, setNotice] = createSignal<string | null>(null);

  const permissionsByCategory = createMemo(() => {
    const categories = new Map<string, number>();
    for (const permission of permissions()) {
      const category = permission.category || 'uncategorized';
      categories.set(category, (categories.get(category) || 0) + 1);
    }
    return Array.from(categories.entries()).sort((a, b) => b[1] - a[1]);
  });

  const permissionGroups = createMemo(() => {
    const groups = new Map<string, AdminPermission[]>();
    for (const permission of permissions()) {
      const category = permission.category || 'uncategorized';
      const list = groups.get(category) || [];
      list.push(permission);
      groups.set(category, list);
    }
    return Array.from(groups.entries())
      .map(([category, list]) => [category, list.sort((a, b) => a.id.localeCompare(b.id))] as const)
      .sort((a, b) => a[0].localeCompare(b[0]));
  });

  const loadRolePermissions = async (role: string) => {
    const token = authStore.token;
    if (!token || !role) return;
    setIsLoadingRolePermissions(true);
    setError(null);
    try {
      const roleData = await fetchAdminJson<string[]>(token, `/admin/roles/${role}/permissions`);
      setRolePermissions(roleData || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load role permissions');
      setRolePermissions([]);
    } finally {
      setIsLoadingRolePermissions(false);
    }
  };

  const loadSecurityData = async () => {
    const token = authStore.token;
    if (!token) return;

    setIsLoading(true);
    setError(null);
    setNotice(null);

    try {
      const [ssoData, permissionsData, systemAdminData, usersData] = await Promise.all([
        fetchAdminJson<AdminSsoConfig[]>(token, '/admin/sso'),
        fetchAdminJson<AdminPermission[]>(token, '/admin/permissions'),
        fetchAdminJson<string[]>(token, '/admin/roles/system_admin/permissions'),
        fetchAdminJson<AdminUsersResponse>(token, '/admin/users?page=1&per_page=200&include_deleted=true').catch(
          () => ({ users: [], total: 0 })
        ),
      ]);
      setSsoConfigs(ssoData || []);
      setPermissions(permissionsData || []);
      setSystemAdminPermissions(systemAdminData || []);

      const knownRoles = new Set(['system_admin', 'org_admin', 'team_admin', 'member', 'guest']);
      const counts: Record<string, number> = {};
      for (const user of usersData.users || []) {
        const tokens = parseRoleTokens(user.role);
        for (const role of tokens) {
          knownRoles.add(role);
          counts[role] = (counts[role] || 0) + 1;
        }
      }
      const sortedRoles = Array.from(knownRoles).sort((a, b) => a.localeCompare(b));
      setRoleOptions(sortedRoles);
      setRoleUserCounts(counts);

      const preferredRole = sortedRoles.includes(selectedRole())
        ? selectedRole()
        : sortedRoles.includes('system_admin')
          ? 'system_admin'
          : sortedRoles[0] || 'member';
      setSelectedRole(preferredRole);
      await loadRolePermissions(preferredRole);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load security data');
    } finally {
      setIsLoading(false);
    }
  };

  createEffect(() => {
    if (authStore.isAuthenticated) {
      void loadSecurityData();
    }
  });

  const toggleRolePermission = (permissionId: string, enabled: boolean) => {
    setRolePermissions((current) => {
      if (enabled) {
        if (current.includes(permissionId)) return current;
        return [...current, permissionId].sort();
      }
      return current.filter((item) => item !== permissionId);
    });
  };

  const saveRolePermissions = async () => {
    const token = authStore.token;
    if (!token) return;
    const role = selectedRole();
    if (!role) return;

    setIsSavingRolePermissions(true);
    setError(null);
    setNotice(null);
    try {
      const updated = await fetchAdminJson<string[]>(token, `/admin/roles/${role}/permissions`, {
        method: 'PUT',
        body: JSON.stringify({ permissions: rolePermissions() }),
      });
      setRolePermissions(updated || []);
      if (role === 'system_admin') {
        setSystemAdminPermissions(updated || []);
      }
      setNotice(`Permissions for role "${role}" updated.`);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update role permissions');
    } finally {
      setIsSavingRolePermissions(false);
    }
  };

  return (
    <div class="space-y-4">
      <Show when={error()}>
        <div class="rounded-lg border border-danger/30 bg-danger/10 px-4 py-3 text-sm text-danger">
          {error()}
        </div>
      </Show>
      <Show when={notice()}>
        <div class="rounded-lg border border-success/30 bg-success/10 px-4 py-3 text-sm text-success">
          {notice()}
        </div>
      </Show>

      <Show when={!isLoading()} fallback={<p class="text-sm text-text-3">Loading security data...</p>}>
        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4 space-y-3">
          <h3 class="text-sm font-semibold text-text-1">SSO Providers</h3>
          <Show
            when={ssoConfigs().length > 0}
            fallback={<p class="text-sm text-text-3">No SSO providers configured.</p>}
          >
            <For each={ssoConfigs()}>
              {(config) => (
                <div class="flex items-center justify-between rounded-lg border border-border-1 px-3 py-2 text-sm">
                  <div>
                    <p class="font-medium text-text-1">{config.display_name || config.provider_key}</p>
                    <p class="text-text-3">{config.provider_type}</p>
                  </div>
                  <span class={`rounded-full px-2 py-1 text-xs font-medium ${config.is_active ? 'bg-success/15 text-success' : 'bg-warning/20 text-warning'}`}>
                    {config.is_active ? 'Active' : 'Disabled'}
                  </span>
                </div>
              )}
            </For>
          </Show>
        </div>

        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4 space-y-3">
          <h3 class="text-sm font-semibold text-text-1">Permission Inventory</h3>
          <p class="text-sm text-text-2">
            {permissions().length} total permissions, {systemAdminPermissions().length} assigned to
            <code class="mx-1 rounded bg-bg-app px-1.5 py-0.5 text-xs">system_admin</code>.
          </p>
          <div class="grid grid-cols-1 gap-2 md:grid-cols-2">
            <For each={permissionsByCategory()}>
              {(entry) => (
                <div class="rounded-lg border border-border-1 px-3 py-2 text-sm">
                  <p class="font-medium text-text-1">{entry[0]}</p>
                  <p class="text-text-3">{entry[1]} permissions</p>
                </div>
              )}
            </For>
          </div>
        </div>

        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4 space-y-4">
          <div class="flex flex-wrap items-center justify-between gap-3">
            <div>
              <h3 class="text-sm font-semibold text-text-1">User Types & Permissions</h3>
              <p class="text-sm text-text-3">Manage role-level permission assignments.</p>
            </div>
            <button
              type="button"
              class="rounded-lg border border-border-1 px-3 py-1.5 text-sm font-medium text-text-2 hover:bg-bg-surface-2"
              onClick={() => {
                void loadSecurityData();
              }}
              disabled={isSavingRolePermissions()}
            >
              Reload
            </button>
          </div>

          <div class="grid grid-cols-1 gap-3 md:grid-cols-3">
            <For each={roleOptions()}>
              {(role) => (
                <div class="rounded-lg border border-border-1 px-3 py-2 text-sm">
                  <p class="font-medium text-text-1">{role}</p>
                  <p class="text-text-3">{roleUserCounts()[role] || 0} users</p>
                </div>
              )}
            </For>
          </div>

          <div class="grid grid-cols-1 gap-3 md:grid-cols-[minmax(0,240px)_1fr]">
            <div class="space-y-2">
              <label class="block">
                <span class="block text-xs font-medium text-text-3 mb-1.5">Role</span>
                <select
                  value={selectedRole()}
                  onChange={(event) => {
                    const role = event.currentTarget.value;
                    setSelectedRole(role);
                    void loadRolePermissions(role);
                  }}
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                >
                  <For each={roleOptions()}>
                    {(role) => <option value={role}>{role}</option>}
                  </For>
                </select>
              </label>

              <button
                type="button"
                class="w-full rounded-lg bg-brand px-3 py-2 text-sm font-medium text-white hover:bg-brand-hover disabled:opacity-60"
                onClick={() => {
                  void saveRolePermissions();
                }}
                disabled={isSavingRolePermissions() || isLoadingRolePermissions()}
              >
                {isSavingRolePermissions() ? 'Saving...' : 'Save Permissions'}
              </button>
            </div>

            <div class="rounded-lg border border-border-1 p-3">
              <Show
                when={!isLoadingRolePermissions()}
                fallback={<p class="text-sm text-text-3">Loading role permissions...</p>}
              >
                <div class="max-h-[360px] overflow-y-auto space-y-4">
                  <For each={permissionGroups()}>
                    {([category, group]) => (
                      <div class="space-y-2">
                        <p class="text-xs uppercase tracking-wide text-text-3">{category}</p>
                        <div class="space-y-1">
                          <For each={group}>
                            {(permission) => (
                              <label class="flex items-start gap-2 text-sm text-text-2">
                                <input
                                  type="checkbox"
                                  checked={rolePermissions().includes(permission.id)}
                                  onChange={(event) => {
                                    toggleRolePermission(permission.id, event.currentTarget.checked);
                                  }}
                                  class="mt-0.5 h-4 w-4 rounded border-border-1 bg-bg-app"
                                />
                                <span>
                                  <span class="font-medium text-text-1">{permission.id}</span>
                                  <Show when={permission.description}>
                                    <span class="block text-xs text-text-3">{permission.description}</span>
                                  </Show>
                                </span>
                              </label>
                            )}
                          </For>
                        </div>
                      </div>
                    )}
                  </For>
                </div>
              </Show>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}

function AdminEmailSection() {
  const [providers, setProviders] = createSignal<AdminEmailProvider[]>([]);
  const [outbox, setOutbox] = createSignal<AdminEmailOutboxEntry[]>([]);
  const [workflows, setWorkflows] = createSignal<AdminEmailWorkflow[]>([]);
  const [events, setEvents] = createSignal<AdminEmailEvent[]>([]);
  const [templateFamilies, setTemplateFamilies] = createSignal<AdminEmailTemplateFamily[]>([]);
  const [templateVersions, setTemplateVersions] = createSignal<AdminEmailTemplateVersion[]>([]);
  const [workflowEdits, setWorkflowEdits] = createSignal<
    Record<string, { default_locale: string; selected_template_family_id: string }>
  >({});
  const [isLoading, setIsLoading] = createSignal(true);
  const [isSettingDefaultId, setIsSettingDefaultId] = createSignal<string | null>(null);
  const [isMutatingOutboxId, setIsMutatingOutboxId] = createSignal<string | null>(null);
  const [isSavingWorkflowId, setIsSavingWorkflowId] = createSignal<string | null>(null);
  const [isCreatingProvider, setIsCreatingProvider] = createSignal(false);
  const [isCreatingWorkflow, setIsCreatingWorkflow] = createSignal(false);
  const [isCreatingTemplateFamily, setIsCreatingTemplateFamily] = createSignal(false);
  const [isCreatingTemplateVersion, setIsCreatingTemplateVersion] = createSignal(false);
  const [isPublishingTemplateVersionId, setIsPublishingTemplateVersionId] = createSignal<string | null>(null);
  const [isSendingTemplatePreviewId, setIsSendingTemplatePreviewId] = createSignal<string | null>(null);
  const [isSendingWorkflowTest, setIsSendingWorkflowTest] = createSignal(false);
  const [isRunningSmtpTest, setIsRunningSmtpTest] = createSignal(false);
  const [testRecipient, setTestRecipient] = createSignal('');
  const [selectedProviderId, setSelectedProviderId] = createSignal('');
  const [selectedWorkflowId, setSelectedWorkflowId] = createSignal('');
  const [selectedTemplateFamilyId, setSelectedTemplateFamilyId] = createSignal('');

  const [providerHost, setProviderHost] = createSignal('');
  const [providerPort, setProviderPort] = createSignal('587');
  const [providerUsername, setProviderUsername] = createSignal('');
  const [providerPassword, setProviderPassword] = createSignal('');
  const [providerTlsMode, setProviderTlsMode] = createSignal('starttls');
  const [providerFromAddress, setProviderFromAddress] = createSignal('');
  const [providerFromName, setProviderFromName] = createSignal('RustChat');
  const [providerEnabled, setProviderEnabled] = createSignal(true);
  const [providerIsDefault, setProviderIsDefault] = createSignal(false);

  const [createWorkflowKey, setCreateWorkflowKey] = createSignal('weekly_digest');
  const [createWorkflowName, setCreateWorkflowName] = createSignal('');
  const [createWorkflowDescription, setCreateWorkflowDescription] = createSignal('');
  const [createWorkflowCategory, setCreateWorkflowCategory] = createSignal('notification');
  const [createWorkflowLocale, setCreateWorkflowLocale] = createSignal('en');
  const [createWorkflowEnabled, setCreateWorkflowEnabled] = createSignal(true);
  const [createWorkflowTemplateFamilyId, setCreateWorkflowTemplateFamilyId] = createSignal('');

  const [templateFamilyKey, setTemplateFamilyKey] = createSignal('');
  const [templateFamilyName, setTemplateFamilyName] = createSignal('');
  const [templateFamilyDescription, setTemplateFamilyDescription] = createSignal('');
  const [templateFamilyWorkflowKey, setTemplateFamilyWorkflowKey] = createSignal('');

  const [templateVersionLocale, setTemplateVersionLocale] = createSignal('en');
  const [templateVersionSubject, setTemplateVersionSubject] = createSignal('');
  const [templateVersionBodyText, setTemplateVersionBodyText] = createSignal('');
  const [templateVersionBodyHtml, setTemplateVersionBodyHtml] = createSignal('');
  const [templateVersionVariablesJson, setTemplateVersionVariablesJson] = createSignal('[]');

  const [notice, setNotice] = createSignal<string | null>(null);
  const [error, setError] = createSignal<string | null>(null);

  const workflowKeyOptions = [
    'user_registration',
    'email_verification',
    'password_reset',
    'password_changed',
    'security_alert',
    'announcements',
    'offline_messages',
    'mention_notifications',
    'admin_invite',
    'weekly_digest',
  ];

  const outboxStatusCounts = createMemo(() => {
    const counts = new Map<string, number>();
    for (const entry of outbox()) {
      counts.set(entry.status, (counts.get(entry.status) || 0) + 1);
    }
    return Array.from(counts.entries()).sort((a, b) => b[1] - a[1]);
  });

  const selectedWorkflow = createMemo(() =>
    workflows().find((workflow) => workflow.id === selectedWorkflowId()) || null
  );

  const loadTemplateVersions = async (familyId: string) => {
    const token = authStore.token;
    if (!token || !familyId) {
      setTemplateVersions([]);
      return;
    }

    try {
      const data = await fetchAdminJson<AdminEmailTemplateVersion[]>(
        token,
        `/admin/email/template-families/${familyId}/versions`
      );
      setTemplateVersions(data || []);
    } catch {
      setTemplateVersions([]);
    }
  };

  const resetProviderForm = () => {
    setProviderHost('');
    setProviderPort('587');
    setProviderUsername('');
    setProviderPassword('');
    setProviderTlsMode('starttls');
    setProviderFromAddress('');
    setProviderFromName('RustChat');
    setProviderEnabled(true);
    setProviderIsDefault(false);
  };

  const resetWorkflowForm = () => {
    setCreateWorkflowKey('weekly_digest');
    setCreateWorkflowName('');
    setCreateWorkflowDescription('');
    setCreateWorkflowCategory('notification');
    setCreateWorkflowLocale('en');
    setCreateWorkflowEnabled(true);
    setCreateWorkflowTemplateFamilyId('');
  };

  const resetTemplateFamilyForm = () => {
    setTemplateFamilyKey('');
    setTemplateFamilyName('');
    setTemplateFamilyDescription('');
    setTemplateFamilyWorkflowKey('');
  };

  const resetTemplateVersionForm = () => {
    setTemplateVersionLocale('en');
    setTemplateVersionSubject('');
    setTemplateVersionBodyText('');
    setTemplateVersionBodyHtml('');
    setTemplateVersionVariablesJson('[]');
  };

  const loadEmailData = async () => {
    const token = authStore.token;
    if (!token) return;

    setIsLoading(true);
    setError(null);
    setNotice(null);
    try {
      const [providerData, outboxData, workflowData, eventData, templateFamilyData] = await Promise.all([
        fetchAdminJson<AdminEmailProvider[]>(token, '/admin/email/providers').catch(() => []),
        fetchAdminJson<AdminEmailOutboxEntry[]>(token, '/admin/email/outbox?page=1&per_page=50').catch(
          () => []
        ),
        fetchAdminJson<AdminEmailWorkflow[]>(token, '/admin/email/workflows').catch(() => []),
        fetchAdminJson<AdminEmailEvent[]>(token, '/admin/email/events?page=1&per_page=50').catch(() => []),
        fetchAdminJson<AdminEmailTemplateFamily[]>(token, '/admin/email/template-families').catch(
          () => []
        ),
      ]);
      setProviders(providerData || []);
      setOutbox(outboxData || []);
      setWorkflows(workflowData || []);
      setEvents(eventData || []);
      setTemplateFamilies(templateFamilyData || []);

      setWorkflowEdits(
        Object.fromEntries(
          (workflowData || []).map((workflow) => [
            workflow.id,
            {
              default_locale: workflow.default_locale,
              selected_template_family_id: workflow.selected_template_family_id || '',
            },
          ])
        )
      );

      if (!testRecipient() && authStore.user()?.email) {
        setTestRecipient(authStore.user()?.email || '');
      }
      if (!selectedProviderId() && providerData.length > 0) {
        const defaultProvider = providerData.find((provider) => provider.is_default) || providerData[0];
        setSelectedProviderId(defaultProvider?.id || '');
      }
      if (!selectedWorkflowId() && workflowData.length > 0) {
        const firstEnabled = workflowData.find((workflow) => workflow.enabled) || workflowData[0];
        setSelectedWorkflowId(firstEnabled?.id || '');
      }
      const activeFamilyId =
        selectedTemplateFamilyId() ||
        createWorkflowTemplateFamilyId() ||
        templateFamilyData[0]?.id ||
        '';
      setSelectedTemplateFamilyId(activeFamilyId);
      if (activeFamilyId) {
        await loadTemplateVersions(activeFamilyId);
      } else {
        setTemplateVersions([]);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load email admin data');
    } finally {
      setIsLoading(false);
    }
  };

  createEffect(() => {
    if (authStore.isAuthenticated) {
      void loadEmailData();
    }
  });

  createEffect(() => {
    const familyId = selectedTemplateFamilyId();
    if (authStore.isAuthenticated && familyId) {
      void loadTemplateVersions(familyId);
    }
  });

  const setDefaultProvider = async (providerId: string) => {
    const token = authStore.token;
    if (!token) return;

    setIsSettingDefaultId(providerId);
    setError(null);
    try {
      await fetchAdminJson<{ success: boolean }>(token, `/admin/email/providers/${providerId}/default`, {
        method: 'POST',
      });
      setNotice('Default provider updated.');
      await loadEmailData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to set default provider');
    } finally {
      setIsSettingDefaultId(null);
    }
  };

  const createProvider = async () => {
    const token = authStore.token;
    if (!token) return;

    const host = providerHost().trim();
    const username = providerUsername().trim();
    const password = providerPassword();
    const fromAddress = providerFromAddress().trim();
    const fromName = providerFromName().trim();
    const port = Number(providerPort());

    if (!host || !username || !password || !fromAddress || !fromName) {
      setError('Host, username, password, from address, and from name are required.');
      return;
    }
    if (!Number.isFinite(port) || port <= 0) {
      setError('Port must be a valid positive number.');
      return;
    }
    if (!isValidEmail(fromAddress)) {
      setError('From address must be a valid email.');
      return;
    }

    setIsCreatingProvider(true);
    setError(null);
    setNotice(null);
    try {
      await fetchAdminJson<AdminEmailProvider>(token, '/admin/email/providers', {
        method: 'POST',
        body: JSON.stringify({
          provider_type: 'smtp',
          host,
          port,
          username,
          password,
          tls_mode: providerTlsMode(),
          skip_cert_verify: false,
          from_address: fromAddress,
          from_name: fromName,
          enabled: providerEnabled(),
          is_default: providerIsDefault(),
        }),
      });
      resetProviderForm();
      setNotice('SMTP provider created.');
      await loadEmailData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create SMTP provider');
    } finally {
      setIsCreatingProvider(false);
    }
  };

  const createWorkflow = async () => {
    const token = authStore.token;
    if (!token) return;

    const workflowKey = createWorkflowKey().trim();
    const name = createWorkflowName().trim();
    if (!workflowKey || !name) {
      setError('Workflow key and workflow name are required.');
      return;
    }

    setIsCreatingWorkflow(true);
    setError(null);
    setNotice(null);
    try {
      const created = await fetchAdminJson<AdminEmailWorkflow>(token, '/admin/email/workflows', {
        method: 'POST',
        body: JSON.stringify({
          workflow_key: workflowKey,
          name,
          description: createWorkflowDescription().trim() || undefined,
          category: createWorkflowCategory(),
          enabled: createWorkflowEnabled(),
          default_locale: createWorkflowLocale().trim() || 'en',
          selected_template_family_id: createWorkflowTemplateFamilyId() || undefined,
        }),
      });
      setSelectedWorkflowId(created.id);
      resetWorkflowForm();
      setNotice(`Workflow "${created.name}" created.`);
      await loadEmailData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create workflow');
    } finally {
      setIsCreatingWorkflow(false);
    }
  };

  const createTemplateFamily = async () => {
    const token = authStore.token;
    if (!token) return;

    const key = templateFamilyKey().trim();
    const name = templateFamilyName().trim();
    if (!key || !name) {
      setError('Template family key and name are required.');
      return;
    }

    setIsCreatingTemplateFamily(true);
    setError(null);
    setNotice(null);
    try {
      const created = await fetchAdminJson<AdminEmailTemplateFamily>(token, '/admin/email/template-families', {
        method: 'POST',
        body: JSON.stringify({
          key,
          name,
          description: templateFamilyDescription().trim() || undefined,
          workflow_key: templateFamilyWorkflowKey() || undefined,
        }),
      });
      setSelectedTemplateFamilyId(created.id);
      setCreateWorkflowTemplateFamilyId(created.id);
      resetTemplateFamilyForm();
      setNotice(`Template family "${created.name}" created.`);
      await loadEmailData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create template family');
    } finally {
      setIsCreatingTemplateFamily(false);
    }
  };

  const createTemplateVersion = async () => {
    const token = authStore.token;
    if (!token) return;
    const familyId = selectedTemplateFamilyId();
    if (!familyId) {
      setError('Select a template family first.');
      return;
    }

    const locale = templateVersionLocale().trim();
    const subject = templateVersionSubject().trim();
    const bodyText = templateVersionBodyText().trim();
    const bodyHtml = templateVersionBodyHtml().trim();
    if (!locale || !subject || !bodyText || !bodyHtml) {
      setError('Locale, subject, body text, and body HTML are required.');
      return;
    }

    let variables: AdminEmailTemplateVariable[] = [];
    try {
      const parsed = JSON.parse(templateVersionVariablesJson().trim() || '[]') as unknown;
      if (!Array.isArray(parsed)) {
        throw new Error('Variables must be an array.');
      }
      variables = parsed as AdminEmailTemplateVariable[];
    } catch (err) {
      setError(
        err instanceof Error
          ? `Variables JSON is invalid: ${err.message}`
          : 'Variables JSON is invalid.'
      );
      return;
    }

    setIsCreatingTemplateVersion(true);
    setError(null);
    setNotice(null);
    try {
      await fetchAdminJson<AdminEmailTemplateVersion>(
        token,
        `/admin/email/template-families/${familyId}/versions`,
        {
          method: 'POST',
          body: JSON.stringify({
            locale,
            subject,
            body_text: bodyText,
            body_html: bodyHtml,
            variables,
          }),
        }
      );
      resetTemplateVersionForm();
      setNotice('Template version created as draft.');
      await loadTemplateVersions(familyId);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create template version');
    } finally {
      setIsCreatingTemplateVersion(false);
    }
  };

  const publishTemplateVersion = async (versionId: string) => {
    const token = authStore.token;
    if (!token) return;
    setIsPublishingTemplateVersionId(versionId);
    setError(null);
    setNotice(null);
    try {
      await fetchAdminJson<AdminEmailTemplateVersion>(
        token,
        `/admin/email/template-versions/${versionId}/publish`,
        { method: 'POST' }
      );
      setNotice('Template version published.');
      await loadTemplateVersions(selectedTemplateFamilyId());
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to publish template version');
    } finally {
      setIsPublishingTemplateVersionId(null);
    }
  };

  const sendTemplatePreview = async (versionId: string) => {
    const token = authStore.token;
    if (!token) return;
    const recipient = testRecipient().trim();
    if (!isValidEmail(recipient)) {
      setError('Enter a valid recipient email to send template preview.');
      return;
    }
    setIsSendingTemplatePreviewId(versionId);
    setError(null);
    setNotice(null);
    try {
      await fetchAdminJson<{ success?: boolean; message?: string }>(
        token,
        `/admin/email/template-versions/${versionId}/send-preview`,
        {
          method: 'POST',
          body: JSON.stringify({ to_email: recipient }),
        }
      );
      setNotice('Template preview email sent.');
      await loadEmailData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to send template preview');
    } finally {
      setIsSendingTemplatePreviewId(null);
    }
  };

  const toggleWorkflowEnabled = async (workflow: AdminEmailWorkflow) => {
    const token = authStore.token;
    if (!token) return;
    if (!workflow.can_disable && workflow.enabled) return;

    setIsSavingWorkflowId(workflow.id);
    setError(null);
    setNotice(null);

    try {
      const updated = await fetchAdminJson<AdminEmailWorkflow>(token, `/admin/email/workflows/${workflow.id}`, {
        method: 'PATCH',
        body: JSON.stringify({ enabled: !workflow.enabled }),
      });
      setWorkflows((current) =>
        current.map((item) => (item.id === updated.id ? updated : item))
      );
      setNotice(`Workflow "${updated.name}" ${updated.enabled ? 'enabled' : 'disabled'}.`);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update workflow');
    } finally {
      setIsSavingWorkflowId(null);
    }
  };

  const saveWorkflowSettings = async (workflow: AdminEmailWorkflow) => {
    const token = authStore.token;
    if (!token) return;
    const draft = workflowEdits()[workflow.id];
    if (!draft) return;

    setIsSavingWorkflowId(workflow.id);
    setError(null);
    setNotice(null);
    try {
      const updated = await fetchAdminJson<AdminEmailWorkflow>(token, `/admin/email/workflows/${workflow.id}`, {
        method: 'PATCH',
        body: JSON.stringify({
          default_locale: draft.default_locale.trim() || workflow.default_locale,
          selected_template_family_id: draft.selected_template_family_id || undefined,
        }),
      });
      setWorkflows((current) =>
        current.map((item) => (item.id === updated.id ? updated : item))
      );
      setWorkflowEdits((current) => ({
        ...current,
        [workflow.id]: {
          default_locale: updated.default_locale,
          selected_template_family_id: updated.selected_template_family_id || '',
        },
      }));
      setNotice(`Workflow "${updated.name}" settings saved.`);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save workflow settings');
    } finally {
      setIsSavingWorkflowId(null);
    }
  };

  const runSmtpTest = async () => {
    const token = authStore.token;
    if (!token) return;
    const recipient = testRecipient().trim();
    if (!isValidEmail(recipient)) {
      setError('Enter a valid recipient email to run SMTP test.');
      return;
    }

    setIsRunningSmtpTest(true);
    setError(null);
    setNotice(null);

    try {
      const result = await fetchAdminJson<AdminEmailTestResult>(token, '/admin/email/send-test', {
        method: 'POST',
        body: JSON.stringify({
          provider_id: selectedProviderId() || undefined,
          to_email: recipient,
          subject: 'RustChat SMTP Test',
          body_text: 'This is a test email from RustChat SMTP admin console.',
        }),
      });
      if (result.status === 'error' || result.success === false) {
        throw new Error(result.error || 'SMTP test failed');
      }
      setNotice(result.message || 'SMTP test email sent successfully.');
      await loadEmailData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'SMTP test failed');
    } finally {
      setIsRunningSmtpTest(false);
    }
  };

  const sendWorkflowTest = async () => {
    const token = authStore.token;
    if (!token) return;
    const recipient = testRecipient().trim();
    if (!isValidEmail(recipient)) {
      setError('Enter a valid recipient email to queue a workflow test.');
      return;
    }

    const workflow = selectedWorkflow();
    if (!workflow) {
      setError('Select a workflow to test.');
      return;
    }

    setIsSendingWorkflowTest(true);
    setError(null);
    setNotice(null);
    try {
      const result = await fetchAdminJson<AdminEmailTestResult>(token, '/admin/email/send-test', {
        method: 'POST',
        body: JSON.stringify({
          to_email: recipient,
          workflow_key: workflow.workflow_key,
        }),
      });
      if (result.success === false) {
        throw new Error(result.error || 'Failed to queue workflow test');
      }
      setNotice(
        result.message ||
          (result.outbox_id
            ? `Workflow test queued (${result.outbox_id}).`
            : 'Workflow test queued successfully.')
      );
      await loadEmailData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to queue workflow test');
    } finally {
      setIsSendingWorkflowTest(false);
    }
  };

  const mutateOutbox = async (outboxId: string, action: 'retry' | 'cancel') => {
    const token = authStore.token;
    if (!token) return;

    setError(null);
    setNotice(null);
    setIsMutatingOutboxId(outboxId);
    try {
      await fetchAdminJson<{ status: string }>(
        token,
        `/admin/email/outbox/${outboxId}/${action}`,
        { method: 'POST' }
      );
      setNotice(action === 'retry' ? 'Outbox entry queued for retry.' : 'Queued outbox entry cancelled.');
      await loadEmailData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update outbox entry');
    } finally {
      setIsMutatingOutboxId(null);
    }
  };

  return (
    <div class="space-y-4">
      <Show when={error()}>
        <div class="rounded-lg border border-danger/30 bg-danger/10 px-4 py-3 text-sm text-danger">
          {error()}
        </div>
      </Show>

      <Show when={notice()}>
        <div class="rounded-lg border border-success/30 bg-success/10 px-4 py-3 text-sm text-success">
          {notice()}
        </div>
      </Show>

      <Show when={!isLoading()} fallback={<p class="text-sm text-text-3">Loading email configuration...</p>}>
        <div class="grid grid-cols-1 gap-3 md:grid-cols-4">
          <OverviewCard label="Providers" value={String(providers().length)} />
          <OverviewCard
            label="Enabled Providers"
            value={String(providers().filter((provider) => provider.enabled).length)}
          />
          <OverviewCard label="Outbox Entries" value={String(outbox().length)} />
          <OverviewCard label="Workflows" value={String(workflows().length)} />
        </div>

        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4 space-y-3">
          <h3 class="text-sm font-semibold text-text-1">SMTP & Mail Tests</h3>
          <label class="block">
            <span class="block text-xs font-medium text-text-3 mb-1.5">Recipient email</span>
            <input
              type="email"
              value={testRecipient()}
              onInput={(event) => setTestRecipient(event.currentTarget.value)}
              placeholder="admin@example.com"
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
            />
          </label>

          <label class="block">
            <span class="block text-xs font-medium text-text-3 mb-1.5">SMTP provider for direct test</span>
            <select
              value={selectedProviderId()}
              onChange={(event) => setSelectedProviderId(event.currentTarget.value)}
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
            >
              <option value="">Use default provider</option>
              <For each={providers()}>
                {(provider) => (
                  <option value={provider.id}>
                    {provider.host}:{provider.port} ({provider.provider_type})
                  </option>
                )}
              </For>
            </select>
          </label>

          <div class="flex flex-wrap items-center gap-2">
            <button
              type="button"
              class="rounded-lg border border-border-1 px-3 py-1.5 text-sm font-medium text-text-2 hover:bg-bg-surface-2 disabled:opacity-60"
              onClick={() => {
                void runSmtpTest();
              }}
              disabled={isRunningSmtpTest()}
            >
              {isRunningSmtpTest() ? 'Running SMTP Test...' : 'Run SMTP Test'}
            </button>
          </div>

          <div class="grid grid-cols-1 gap-2 md:grid-cols-[minmax(0,1fr)_auto]">
            <select
              value={selectedWorkflowId()}
              onChange={(event) => setSelectedWorkflowId(event.currentTarget.value)}
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
            >
              <option value="" disabled>
                Select workflow
              </option>
              <For each={workflows()}>
                {(workflow) => (
                  <option value={workflow.id}>
                    {workflow.name} ({workflow.workflow_key})
                  </option>
                )}
              </For>
            </select>

            <button
              type="button"
              class="rounded-lg bg-brand px-3 py-2 text-sm font-medium text-white hover:bg-brand-hover disabled:opacity-60"
              onClick={() => {
                void sendWorkflowTest();
              }}
              disabled={isSendingWorkflowTest() || !selectedWorkflowId()}
            >
              {isSendingWorkflowTest() ? 'Queueing...' : 'Queue Workflow Test'}
            </button>
          </div>
        </div>

        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4 space-y-4">
          <h3 class="text-sm font-semibold text-text-1">Create SMTP Provider</h3>
          <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
            <label class="block">
              <span class="block text-xs text-text-3 mb-1">Host</span>
              <input
                type="text"
                value={providerHost()}
                onInput={(event) => setProviderHost(event.currentTarget.value)}
                placeholder="smtp.example.com"
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
              />
            </label>
            <label class="block">
              <span class="block text-xs text-text-3 mb-1">Port</span>
              <input
                type="number"
                min="1"
                value={providerPort()}
                onInput={(event) => setProviderPort(event.currentTarget.value)}
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
              />
            </label>
            <label class="block">
              <span class="block text-xs text-text-3 mb-1">Username</span>
              <input
                type="text"
                value={providerUsername()}
                onInput={(event) => setProviderUsername(event.currentTarget.value)}
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
              />
            </label>
            <label class="block">
              <span class="block text-xs text-text-3 mb-1">Password</span>
              <input
                type="password"
                value={providerPassword()}
                onInput={(event) => setProviderPassword(event.currentTarget.value)}
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
              />
            </label>
            <label class="block">
              <span class="block text-xs text-text-3 mb-1">TLS Mode</span>
              <select
                value={providerTlsMode()}
                onChange={(event) => setProviderTlsMode(event.currentTarget.value)}
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
              >
                <option value="starttls">STARTTLS</option>
                <option value="implicit_tls">Implicit TLS</option>
                <option value="none">No TLS</option>
              </select>
            </label>
            <label class="block">
              <span class="block text-xs text-text-3 mb-1">From Name</span>
              <input
                type="text"
                value={providerFromName()}
                onInput={(event) => setProviderFromName(event.currentTarget.value)}
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
              />
            </label>
            <label class="block md:col-span-2">
              <span class="block text-xs text-text-3 mb-1">From Address</span>
              <input
                type="email"
                value={providerFromAddress()}
                onInput={(event) => setProviderFromAddress(event.currentTarget.value)}
                placeholder="noreply@example.com"
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
              />
            </label>
          </div>
          <div class="flex flex-wrap items-center gap-4">
            <label class="inline-flex items-center gap-2 text-sm text-text-2">
              <input
                type="checkbox"
                checked={providerEnabled()}
                onChange={(event) => setProviderEnabled(event.currentTarget.checked)}
                class="h-4 w-4 rounded border-border-1 bg-bg-app"
              />
              Enabled
            </label>
            <label class="inline-flex items-center gap-2 text-sm text-text-2">
              <input
                type="checkbox"
                checked={providerIsDefault()}
                onChange={(event) => setProviderIsDefault(event.currentTarget.checked)}
                class="h-4 w-4 rounded border-border-1 bg-bg-app"
              />
              Set as default
            </label>
          </div>
          <div class="flex items-center gap-2">
            <button
              type="button"
              class="rounded-lg bg-brand px-3 py-2 text-sm font-medium text-white hover:bg-brand-hover disabled:opacity-60"
              onClick={() => {
                void createProvider();
              }}
              disabled={isCreatingProvider()}
            >
              {isCreatingProvider() ? 'Creating...' : 'Create SMTP Provider'}
            </button>
            <button
              type="button"
              class="rounded-lg border border-border-1 px-3 py-2 text-sm font-medium text-text-2 hover:bg-bg-surface-2"
              onClick={resetProviderForm}
              disabled={isCreatingProvider()}
            >
              Reset
            </button>
          </div>
        </div>

        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4 space-y-4">
          <h3 class="text-sm font-semibold text-text-1">Create Workflow</h3>
          <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
            <label class="block">
              <span class="block text-xs text-text-3 mb-1">Workflow Key</span>
              <select
                value={createWorkflowKey()}
                onChange={(event) => setCreateWorkflowKey(event.currentTarget.value)}
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
              >
                <For each={workflowKeyOptions}>
                  {(workflowKey) => (
                    <option value={workflowKey}>{workflowKey}</option>
                  )}
                </For>
              </select>
            </label>
            <label class="block">
              <span class="block text-xs text-text-3 mb-1">Display Name</span>
              <input
                type="text"
                value={createWorkflowName()}
                onInput={(event) => setCreateWorkflowName(event.currentTarget.value)}
                placeholder="Weekly Digest Custom"
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
              />
            </label>
            <label class="block">
              <span class="block text-xs text-text-3 mb-1">Category</span>
              <select
                value={createWorkflowCategory()}
                onChange={(event) => setCreateWorkflowCategory(event.currentTarget.value)}
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
              >
                <option value="system">system</option>
                <option value="notification">notification</option>
                <option value="marketing">marketing</option>
              </select>
            </label>
            <label class="block">
              <span class="block text-xs text-text-3 mb-1">Default Locale</span>
              <input
                type="text"
                value={createWorkflowLocale()}
                onInput={(event) => setCreateWorkflowLocale(event.currentTarget.value)}
                placeholder="en"
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
              />
            </label>
            <label class="block md:col-span-2">
              <span class="block text-xs text-text-3 mb-1">Template Family</span>
              <select
                value={createWorkflowTemplateFamilyId()}
                onChange={(event) => setCreateWorkflowTemplateFamilyId(event.currentTarget.value)}
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
              >
                <option value="">No template family</option>
                <For each={templateFamilies()}>
                  {(family) => (
                    <option value={family.id}>
                      {family.name} ({family.key})
                    </option>
                  )}
                </For>
              </select>
            </label>
            <label class="block md:col-span-2">
              <span class="block text-xs text-text-3 mb-1">Description</span>
              <input
                type="text"
                value={createWorkflowDescription()}
                onInput={(event) => setCreateWorkflowDescription(event.currentTarget.value)}
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
              />
            </label>
          </div>
          <label class="inline-flex items-center gap-2 text-sm text-text-2">
            <input
              type="checkbox"
              checked={createWorkflowEnabled()}
              onChange={(event) => setCreateWorkflowEnabled(event.currentTarget.checked)}
              class="h-4 w-4 rounded border-border-1 bg-bg-app"
            />
            Enabled
          </label>
          <div class="flex items-center gap-2">
            <button
              type="button"
              class="rounded-lg bg-brand px-3 py-2 text-sm font-medium text-white hover:bg-brand-hover disabled:opacity-60"
              onClick={() => {
                void createWorkflow();
              }}
              disabled={isCreatingWorkflow()}
            >
              {isCreatingWorkflow() ? 'Creating...' : 'Create Workflow'}
            </button>
            <button
              type="button"
              class="rounded-lg border border-border-1 px-3 py-2 text-sm font-medium text-text-2 hover:bg-bg-surface-2"
              onClick={resetWorkflowForm}
              disabled={isCreatingWorkflow()}
            >
              Reset
            </button>
          </div>
        </div>

        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4 space-y-4">
          <h3 class="text-sm font-semibold text-text-1">Mail Templates</h3>
          <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
            <div class="rounded-lg border border-border-1 p-3 space-y-3">
              <p class="text-sm font-medium text-text-1">Create Template Family</p>
              <label class="block">
                <span class="block text-xs text-text-3 mb-1">Family Key</span>
                <input
                  type="text"
                  value={templateFamilyKey()}
                  onInput={(event) => setTemplateFamilyKey(event.currentTarget.value)}
                  placeholder="weekly_digest_custom"
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                />
              </label>
              <label class="block">
                <span class="block text-xs text-text-3 mb-1">Family Name</span>
                <input
                  type="text"
                  value={templateFamilyName()}
                  onInput={(event) => setTemplateFamilyName(event.currentTarget.value)}
                  placeholder="Weekly Digest Custom"
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                />
              </label>
              <label class="block">
                <span class="block text-xs text-text-3 mb-1">Workflow Key (optional)</span>
                <select
                  value={templateFamilyWorkflowKey()}
                  onChange={(event) => setTemplateFamilyWorkflowKey(event.currentTarget.value)}
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                >
                  <option value="">None</option>
                  <For each={workflowKeyOptions}>
                    {(workflowKey) => (
                      <option value={workflowKey}>{workflowKey}</option>
                    )}
                  </For>
                </select>
              </label>
              <label class="block">
                <span class="block text-xs text-text-3 mb-1">Description</span>
                <input
                  type="text"
                  value={templateFamilyDescription()}
                  onInput={(event) => setTemplateFamilyDescription(event.currentTarget.value)}
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                />
              </label>
              <div class="flex items-center gap-2">
                <button
                  type="button"
                  class="rounded-lg bg-brand px-3 py-2 text-sm font-medium text-white hover:bg-brand-hover disabled:opacity-60"
                  onClick={() => {
                    void createTemplateFamily();
                  }}
                  disabled={isCreatingTemplateFamily()}
                >
                  {isCreatingTemplateFamily() ? 'Creating...' : 'Create Family'}
                </button>
                <button
                  type="button"
                  class="rounded-lg border border-border-1 px-3 py-2 text-sm font-medium text-text-2 hover:bg-bg-surface-2"
                  onClick={resetTemplateFamilyForm}
                  disabled={isCreatingTemplateFamily()}
                >
                  Reset
                </button>
              </div>
            </div>

            <div class="rounded-lg border border-border-1 p-3 space-y-3">
              <p class="text-sm font-medium text-text-1">Create Template Version</p>
              <label class="block">
                <span class="block text-xs text-text-3 mb-1">Template Family</span>
                <select
                  value={selectedTemplateFamilyId()}
                  onChange={(event) => setSelectedTemplateFamilyId(event.currentTarget.value)}
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                >
                  <option value="">Select family</option>
                  <For each={templateFamilies()}>
                    {(family) => (
                      <option value={family.id}>
                        {family.name} ({family.key})
                      </option>
                    )}
                  </For>
                </select>
              </label>
              <label class="block">
                <span class="block text-xs text-text-3 mb-1">Locale</span>
                <input
                  type="text"
                  value={templateVersionLocale()}
                  onInput={(event) => setTemplateVersionLocale(event.currentTarget.value)}
                  placeholder="en"
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                />
              </label>
              <label class="block">
                <span class="block text-xs text-text-3 mb-1">Subject</span>
                <input
                  type="text"
                  value={templateVersionSubject()}
                  onInput={(event) => setTemplateVersionSubject(event.currentTarget.value)}
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                />
              </label>
              <label class="block">
                <span class="block text-xs text-text-3 mb-1">Body Text</span>
                <textarea
                  value={templateVersionBodyText()}
                  onInput={(event) => setTemplateVersionBodyText(event.currentTarget.value)}
                  rows={3}
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-xs text-text-1"
                />
              </label>
              <label class="block">
                <span class="block text-xs text-text-3 mb-1">Body HTML</span>
                <textarea
                  value={templateVersionBodyHtml()}
                  onInput={(event) => setTemplateVersionBodyHtml(event.currentTarget.value)}
                  rows={3}
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-xs text-text-1"
                />
              </label>
              <label class="block">
                <span class="block text-xs text-text-3 mb-1">Variables JSON</span>
                <textarea
                  value={templateVersionVariablesJson()}
                  onInput={(event) => setTemplateVersionVariablesJson(event.currentTarget.value)}
                  rows={3}
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 font-mono text-xs text-text-1"
                />
              </label>
              <div class="flex items-center gap-2">
                <button
                  type="button"
                  class="rounded-lg bg-brand px-3 py-2 text-sm font-medium text-white hover:bg-brand-hover disabled:opacity-60"
                  onClick={() => {
                    void createTemplateVersion();
                  }}
                  disabled={isCreatingTemplateVersion()}
                >
                  {isCreatingTemplateVersion() ? 'Creating...' : 'Create Version'}
                </button>
                <button
                  type="button"
                  class="rounded-lg border border-border-1 px-3 py-2 text-sm font-medium text-text-2 hover:bg-bg-surface-2"
                  onClick={resetTemplateVersionForm}
                  disabled={isCreatingTemplateVersion()}
                >
                  Reset
                </button>
              </div>
            </div>
          </div>

          <div class="rounded-lg border border-border-1 p-3 space-y-2">
            <p class="text-sm font-medium text-text-1">Template Versions</p>
            <Show when={templateVersions().length > 0} fallback={<p class="text-sm text-text-3">No versions for selected family.</p>}>
              <div class="space-y-2">
                <For each={templateVersions()}>
                  {(version) => (
                    <div class="rounded-lg border border-border-1 px-3 py-3">
                      <div class="flex flex-wrap items-center justify-between gap-2">
                        <div class="space-y-1">
                          <p class="text-sm font-medium text-text-1">
                            v{version.version} · {version.locale}
                          </p>
                          <p class="text-xs text-text-3">
                            {version.status} · {version.subject}
                          </p>
                        </div>
                        <div class="flex items-center gap-2">
                          <button
                            type="button"
                            class="rounded-lg border border-border-1 px-2.5 py-1 text-xs font-medium text-text-2 hover:bg-bg-surface-2 disabled:opacity-60"
                            onClick={() => {
                              void sendTemplatePreview(version.id);
                            }}
                            disabled={isSendingTemplatePreviewId() === version.id}
                          >
                            {isSendingTemplatePreviewId() === version.id ? 'Sending...' : 'Send Preview'}
                          </button>
                          <Show when={version.status === 'draft'}>
                            <button
                              type="button"
                              class="rounded-lg border border-border-1 px-2.5 py-1 text-xs font-medium text-text-2 hover:bg-bg-surface-2 disabled:opacity-60"
                              onClick={() => {
                                void publishTemplateVersion(version.id);
                              }}
                              disabled={isPublishingTemplateVersionId() === version.id}
                            >
                              {isPublishingTemplateVersionId() === version.id ? 'Publishing...' : 'Publish'}
                            </button>
                          </Show>
                        </div>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </div>
        </div>

        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4 space-y-3">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-semibold text-text-1">Notification Workflows</h3>
            <button
              type="button"
              class="rounded-lg border border-border-1 px-3 py-1.5 text-sm font-medium text-text-2 hover:bg-bg-surface-2"
              onClick={() => {
                void loadEmailData();
              }}
            >
              Reload
            </button>
          </div>

          <Show when={workflows().length > 0} fallback={<p class="text-sm text-text-3">No workflows found.</p>}>
            <div class="space-y-2">
              <For each={workflows()}>
                {(workflow) => (
                  <div class="rounded-lg border border-border-1 px-3 py-3">
                    <div class="flex flex-wrap items-center justify-between gap-3">
                      <div class="space-y-1">
                        <p class="text-sm font-medium text-text-1">{workflow.name}</p>
                        <p class="text-xs text-text-3">
                          {workflow.workflow_key} · {workflow.category} · locale {workflow.default_locale}
                        </p>
                      </div>
                      <div class="flex flex-wrap items-center gap-2">
                        <span
                          class={`rounded-full px-2 py-1 text-xs font-medium ${
                            workflow.enabled ? 'bg-success/15 text-success' : 'bg-warning/20 text-warning'
                          }`}
                        >
                          {workflow.enabled ? 'Enabled' : 'Disabled'}
                        </span>
                        <button
                          type="button"
                          class="rounded-lg border border-border-1 px-3 py-1.5 text-xs font-medium text-text-2 hover:bg-bg-surface-2 disabled:opacity-60"
                          disabled={
                            isSavingWorkflowId() === workflow.id ||
                            (!workflow.can_disable && workflow.enabled)
                          }
                          onClick={() => {
                            void toggleWorkflowEnabled(workflow);
                          }}
                        >
                          {isSavingWorkflowId() === workflow.id
                            ? 'Updating...'
                            : workflow.enabled
                              ? workflow.can_disable
                                ? 'Disable'
                                : 'Required'
                              : 'Enable'}
                        </button>
                        <button
                          type="button"
                          class="rounded-lg border border-border-1 px-3 py-1.5 text-xs font-medium text-text-2 hover:bg-bg-surface-2 disabled:opacity-60"
                          onClick={() => {
                            void saveWorkflowSettings(workflow);
                          }}
                          disabled={isSavingWorkflowId() === workflow.id}
                        >
                          {isSavingWorkflowId() === workflow.id ? 'Saving...' : 'Save Settings'}
                        </button>
                      </div>
                    </div>
                    <div class="mt-3 grid grid-cols-1 gap-2 md:grid-cols-2">
                      <label class="block">
                        <span class="block text-xs text-text-3 mb-1">Default Locale</span>
                        <input
                          type="text"
                          value={workflowEdits()[workflow.id]?.default_locale || workflow.default_locale}
                          onInput={(event) =>
                            setWorkflowEdits((current) => ({
                              ...current,
                              [workflow.id]: {
                                default_locale: event.currentTarget.value,
                                selected_template_family_id:
                                  current[workflow.id]?.selected_template_family_id ||
                                  workflow.selected_template_family_id ||
                                  '',
                              },
                            }))
                          }
                          class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                        />
                      </label>
                      <label class="block">
                        <span class="block text-xs text-text-3 mb-1">Template Family</span>
                        <select
                          value={
                            workflowEdits()[workflow.id]?.selected_template_family_id ||
                            workflow.selected_template_family_id ||
                            ''
                          }
                          onChange={(event) =>
                            setWorkflowEdits((current) => ({
                              ...current,
                              [workflow.id]: {
                                default_locale:
                                  current[workflow.id]?.default_locale || workflow.default_locale,
                                selected_template_family_id: event.currentTarget.value,
                              },
                            }))
                          }
                          class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                        >
                          <option value="">Keep current</option>
                          <For each={templateFamilies()}>
                            {(family) => (
                              <option value={family.id}>
                                {family.name} ({family.key})
                              </option>
                            )}
                          </For>
                        </select>
                      </label>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>

        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4 space-y-3">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-semibold text-text-1">Mail Providers</h3>
            <button
              type="button"
              class="rounded-lg border border-border-1 px-3 py-1.5 text-sm font-medium text-text-2 hover:bg-bg-surface-2"
              onClick={() => {
                void loadEmailData();
              }}
            >
              Reload
            </button>
          </div>

          <Show when={providers().length > 0} fallback={<p class="text-sm text-text-3">No mail providers configured.</p>}>
            <div class="space-y-2">
              <For each={providers()}>
                {(provider) => (
                  <div class="rounded-lg border border-border-1 px-3 py-3">
                    <div class="flex flex-wrap items-center justify-between gap-3">
                      <div class="space-y-1">
                        <p class="text-sm font-medium text-text-1">
                          {provider.provider_type.toUpperCase()} · {provider.host}:{provider.port}
                        </p>
                        <p class="text-xs text-text-3">
                          From {provider.from_name} &lt;{provider.from_address}&gt;
                        </p>
                        <p class="text-xs text-text-3">
                          TLS {provider.tls_mode || 'starttls'} · User {provider.username}
                        </p>
                      </div>

                      <div class="flex items-center gap-2">
                        <span
                          class={`rounded-full px-2 py-1 text-xs font-medium ${
                            provider.enabled ? 'bg-success/15 text-success' : 'bg-warning/20 text-warning'
                          }`}
                        >
                          {provider.enabled ? 'Enabled' : 'Disabled'}
                        </span>
                        <button
                          type="button"
                          class="rounded-lg border border-border-1 px-3 py-1.5 text-xs font-medium text-text-2 hover:bg-bg-surface-2 disabled:opacity-60"
                          onClick={() => {
                            void setDefaultProvider(provider.id);
                          }}
                          disabled={provider.is_default || isSettingDefaultId() === provider.id}
                        >
                          {provider.is_default
                            ? 'Default'
                            : isSettingDefaultId() === provider.id
                              ? 'Applying...'
                              : 'Set Default'}
                        </button>
                      </div>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>

        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4 space-y-3">
          <h3 class="text-sm font-semibold text-text-1">Recent Outbox Status</h3>
          <Show when={outboxStatusCounts().length > 0} fallback={<p class="text-sm text-text-3">Outbox is empty.</p>}>
            <div class="grid grid-cols-2 gap-2 md:grid-cols-4">
              <For each={outboxStatusCounts()}>
                {(entry) => (
                  <div class="rounded-lg border border-border-1 px-3 py-2 text-sm">
                    <p class="font-medium text-text-1 capitalize">{entry[0]}</p>
                    <p class="text-text-3">{entry[1]} messages</p>
                  </div>
                )}
              </For>
            </div>
          </Show>

          <Show when={outbox().length > 0}>
            <div class="overflow-x-auto rounded-lg border border-border-1">
              <table class="w-full text-left text-xs">
                <thead class="border-b border-border-1 bg-bg-surface-2 text-text-3">
                  <tr>
                    <th class="px-3 py-2 font-medium">Recipient</th>
                    <th class="px-3 py-2 font-medium">Subject</th>
                    <th class="px-3 py-2 font-medium">Status</th>
                    <th class="px-3 py-2 font-medium">Workflow</th>
                    <th class="px-3 py-2 font-medium">Attempts</th>
                    <th class="px-3 py-2 font-medium">Created</th>
                    <th class="px-3 py-2 font-medium">Sent</th>
                    <th class="px-3 py-2 font-medium text-right">Action</th>
                  </tr>
                </thead>
                <tbody>
                  <For each={outbox()}>
                    {(entry) => (
                      <tr class="border-b border-border-1/60 last:border-b-0">
                        <td class="px-3 py-2 text-text-2">{entry.recipient_email}</td>
                        <td class="px-3 py-2 text-text-2">{entry.subject}</td>
                        <td class="px-3 py-2 text-text-2 capitalize">{entry.status}</td>
                        <td class="px-3 py-2 text-text-2">{entry.workflow_key || '-'}</td>
                        <td class="px-3 py-2 text-text-2">{entry.attempt_count}/{entry.max_attempts}</td>
                        <td class="px-3 py-2 text-text-2">{formatDateTime(entry.created_at)}</td>
                        <td class="px-3 py-2 text-text-2">{formatDateTime(entry.sent_at)}</td>
                        <td class="px-3 py-2 text-right">
                          <Show when={entry.status === 'failed'}>
                            <button
                              type="button"
                              class="rounded-lg border border-border-1 px-2 py-1 font-medium text-text-2 hover:bg-bg-surface-2 disabled:opacity-60"
                              onClick={() => {
                                void mutateOutbox(entry.id, 'retry');
                              }}
                              disabled={isMutatingOutboxId() === entry.id}
                            >
                              {isMutatingOutboxId() === entry.id ? 'Working...' : 'Retry'}
                            </button>
                          </Show>
                          <Show when={entry.status === 'queued'}>
                            <button
                              type="button"
                              class="rounded-lg border border-warning/40 px-2 py-1 font-medium text-warning hover:bg-warning/10 disabled:opacity-60"
                              onClick={() => {
                                void mutateOutbox(entry.id, 'cancel');
                              }}
                              disabled={isMutatingOutboxId() === entry.id}
                            >
                              {isMutatingOutboxId() === entry.id ? 'Working...' : 'Cancel'}
                            </button>
                          </Show>
                        </td>
                      </tr>
                    )}
                  </For>
                </tbody>
              </table>
            </div>
          </Show>
        </div>

        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4 space-y-3">
          <h3 class="text-sm font-semibold text-text-1">Recent Email Events</h3>
          <Show when={events().length > 0} fallback={<p class="text-sm text-text-3">No email events captured yet.</p>}>
            <div class="overflow-x-auto rounded-lg border border-border-1">
              <table class="w-full text-left text-xs">
                <thead class="border-b border-border-1 bg-bg-surface-2 text-text-3">
                  <tr>
                    <th class="px-3 py-2 font-medium">Event</th>
                    <th class="px-3 py-2 font-medium">Workflow</th>
                    <th class="px-3 py-2 font-medium">Recipient</th>
                    <th class="px-3 py-2 font-medium">Error</th>
                    <th class="px-3 py-2 font-medium">Time</th>
                  </tr>
                </thead>
                <tbody>
                  <For each={events()}>
                    {(event) => (
                      <tr class="border-b border-border-1/60 last:border-b-0">
                        <td class="px-3 py-2 text-text-2 capitalize">{event.event_type}</td>
                        <td class="px-3 py-2 text-text-2">{event.workflow_key || '-'}</td>
                        <td class="px-3 py-2 text-text-2">{event.recipient_email}</td>
                        <td class="px-3 py-2 text-text-2">
                          {event.error_category || event.error_message || '-'}
                        </td>
                        <td class="px-3 py-2 text-text-2">{formatDateTime(event.created_at)}</td>
                      </tr>
                    )}
                  </For>
                </tbody>
              </table>
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
}

function AdminComplianceSection() {
  const [complianceConfig, setComplianceConfig] = createSignal<Record<string, unknown> | null>(null);
  const [messageRetentionDays, setMessageRetentionDays] = createSignal(0);
  const [fileRetentionDays, setFileRetentionDays] = createSignal(0);
  const [isLoading, setIsLoading] = createSignal(true);
  const [isSaving, setIsSaving] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [notice, setNotice] = createSignal<string | null>(null);

  const loadComplianceConfig = async () => {
    const token = authStore.token;
    if (!token) return;

    setIsLoading(true);
    setError(null);
    setNotice(null);

    try {
      const response = await fetchAdminJson<AdminConfig>(token, '/admin/config');
      const compliance = response.compliance || {};
      setComplianceConfig(compliance);
      setMessageRetentionDays(Number(compliance.message_retention_days || 0));
      setFileRetentionDays(Number(compliance.file_retention_days || 0));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load compliance settings');
    } finally {
      setIsLoading(false);
    }
  };

  createEffect(() => {
    if (authStore.isAuthenticated) {
      void loadComplianceConfig();
    }
  });

  const saveComplianceConfig = async () => {
    const token = authStore.token;
    if (!token || !complianceConfig()) return;

    setIsSaving(true);
    setError(null);
    setNotice(null);

    try {
      const nextCompliance = {
        ...complianceConfig(),
        message_retention_days: Math.max(0, Math.floor(messageRetentionDays())),
        file_retention_days: Math.max(0, Math.floor(fileRetentionDays())),
      };

      await fetchAdminJson<Record<string, unknown>>(token, '/admin/config/compliance', {
        method: 'PATCH',
        body: JSON.stringify(nextCompliance),
      });

      setComplianceConfig(nextCompliance);
      setNotice('Compliance settings saved.');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save compliance settings');
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div class="space-y-4">
      <Show when={error()}>
        <div class="rounded-lg border border-danger/30 bg-danger/10 px-4 py-3 text-sm text-danger">
          {error()}
        </div>
      </Show>

      <Show when={notice()}>
        <div class="rounded-lg border border-success/30 bg-success/10 px-4 py-3 text-sm text-success">
          {notice()}
        </div>
      </Show>

      <Show when={!isLoading()} fallback={<p class="text-sm text-text-3">Loading compliance settings...</p>}>
        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4 space-y-4">
          <label class="block">
            <span class="block text-sm font-medium text-text-2 mb-1.5">Message retention (days)</span>
            <input
              type="number"
              min="0"
              value={messageRetentionDays()}
              onInput={(event) => setMessageRetentionDays(Number(event.currentTarget.value))}
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
            />
          </label>

          <label class="block">
            <span class="block text-sm font-medium text-text-2 mb-1.5">File retention (days)</span>
            <input
              type="number"
              min="0"
              value={fileRetentionDays()}
              onInput={(event) => setFileRetentionDays(Number(event.currentTarget.value))}
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
            />
          </label>

          <div class="flex items-center gap-3">
            <button
              type="button"
              class="rounded-lg bg-brand px-4 py-2 text-sm font-medium text-white hover:bg-brand-hover disabled:opacity-60"
              onClick={() => {
                void saveComplianceConfig();
              }}
              disabled={isSaving()}
            >
              {isSaving() ? 'Saving...' : 'Save'}
            </button>
            <button
              type="button"
              class="rounded-lg border border-border-1 px-4 py-2 text-sm font-medium text-text-2 hover:bg-bg-surface-2"
              onClick={() => {
                void loadComplianceConfig();
              }}
              disabled={isSaving()}
            >
              Reload
            </button>
          </div>
        </div>
      </Show>
    </div>
  );
}

function AdminMembershipPoliciesSection() {
  const [policies, setPolicies] = createSignal<AdminMembershipPolicy[]>([]);
  const [summary, setSummary] = createSignal<AdminMembershipAuditSummary | null>(null);
  const [metadata, setMetadata] = createSignal<MembershipPolicyMetadata | null>(null);
  const [availableTeams, setAvailableTeams] = createSignal<AdminTeam[]>([]);
  const [availableChannels, setAvailableChannels] = createSignal<AdminChannel[]>([]);
  const [isLoading, setIsLoading] = createSignal(true);
  const [isMutatingPolicyId, setIsMutatingPolicyId] = createSignal<string | null>(null);
  const [isCreating, setIsCreating] = createSignal(false);
  const [editingPolicyId, setEditingPolicyId] = createSignal<string | null>(null);
  const [isResyncing, setIsResyncing] = createSignal(false);
  const [resyncUserId, setResyncUserId] = createSignal('');
  const [createName, setCreateName] = createSignal('');
  const [createDescription, setCreateDescription] = createSignal('');
  const [createScopeType, setCreateScopeType] = createSignal<'global' | 'team'>('global');
  const [createScopeTeamId, setCreateScopeTeamId] = createSignal('');
  const [createSourceType, setCreateSourceType] = createSignal<
    'all_users' | 'auth_service' | 'group' | 'role' | 'org'
  >('all_users');
  const [createSourceConfig, setCreateSourceConfig] = createSignal('{}');
  const [createEnabled, setCreateEnabled] = createSignal(true);
  const [createPriority, setCreatePriority] = createSignal('0');
  const [createTargets, setCreateTargets] = createSignal<MembershipPolicyTargetDraft[]>([
    {
      target_type: 'team',
      target_id: '',
      role_mode: 'member',
    },
  ]);
  const [notice, setNotice] = createSignal<string | null>(null);
  const [error, setError] = createSignal<string | null>(null);

  const sourceTypeOptions = createMemo(
    () =>
      metadata()?.source_types || [
        { value: 'all_users', label: 'All Users' },
        { value: 'auth_service', label: 'Authentication Service' },
        { value: 'group', label: 'Group Membership' },
        { value: 'role', label: 'User Role' },
        { value: 'org', label: 'Organization' },
      ]
  );

  const roleModeOptions = createMemo(
    () =>
      metadata()?.role_modes || [
        { value: 'member', label: 'Member' },
        { value: 'admin', label: 'Admin' },
      ]
  );

  const createSourceConfigPlaceholder = createMemo(() => {
    switch (createSourceType()) {
      case 'auth_service':
        return '{\n  "auth_provider": "oidc"\n}';
      case 'group':
        return '{\n  "group_names": ["engineering"]\n}';
      case 'role':
        return '{\n  "roles": ["member"]\n}';
      case 'org':
        return '{\n  "org_ids": ["00000000-0000-0000-0000-000000000000"]\n}';
      default:
        return '{}';
    }
  });

  const resetCreatePolicyForm = () => {
    setEditingPolicyId(null);
    setCreateName('');
    setCreateDescription('');
    setCreateScopeType('global');
    setCreateScopeTeamId('');
    setCreateSourceType('all_users');
    setCreateSourceConfig('{}');
    setCreateEnabled(true);
    setCreatePriority('0');
    setCreateTargets([
      {
        target_type: 'team',
        target_id: '',
        role_mode: 'member',
      },
    ]);
  };

  const loadMembershipData = async () => {
    const token = authStore.token;
    if (!token) return;

    setIsLoading(true);
    setError(null);
    setNotice(null);

    try {
      const [policiesData, summaryData, metadataData, teamsData, channelsData] = await Promise.all([
        fetchAdminJson<AdminMembershipPolicy[]>(token, '/admin/membership-policies'),
        fetchAdminJson<AdminMembershipAuditSummary>(token, '/admin/audit/membership/summary'),
        fetchAdminJson<MembershipPolicyMetadata>(token, '/admin/membership-policies/metadata').catch(
          () => null
        ),
        fetchAdminJson<AdminTeamsResponse>(token, '/admin/teams?page=1&per_page=100').catch(() => ({
          teams: [],
          total: 0,
        })),
        fetchAdminJson<AdminChannelsResponse>(token, '/admin/channels?page=1&per_page=100').catch(
          () => ({
            channels: [],
            total: 0,
          })
        ),
      ]);
      setPolicies(policiesData || []);
      setSummary(summaryData || null);
      setMetadata(metadataData);
      setAvailableTeams(teamsData.teams || []);
      setAvailableChannels(channelsData.channels || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load membership policy data');
    } finally {
      setIsLoading(false);
    }
  };

  createEffect(() => {
    if (authStore.isAuthenticated) {
      void loadMembershipData();
    }
  });

  const togglePolicyEnabled = async (policy: AdminMembershipPolicy) => {
    const token = authStore.token;
    if (!token) return;

    setIsMutatingPolicyId(policy.id);
    setError(null);
    try {
      const updatedPolicy = await fetchAdminJson<AdminMembershipPolicy>(
        token,
        `/admin/membership-policies/${policy.id}`,
        {
          method: 'PUT',
          body: JSON.stringify({ enabled: !policy.enabled }),
        }
      );

      setPolicies((current) =>
        current.map((item) => (item.id === updatedPolicy.id ? updatedPolicy : item))
      );
      setNotice(`Policy "${updatedPolicy.name}" ${updatedPolicy.enabled ? 'enabled' : 'disabled'}.`);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update policy');
    } finally {
      setIsMutatingPolicyId(null);
    }
  };

  const updateCreateTarget = (
    index: number,
    updates: Partial<MembershipPolicyTargetDraft>
  ) => {
    setCreateTargets((current) =>
      current.map((target, targetIndex) => {
        if (targetIndex !== index) return target;
        const nextTargetType = updates.target_type ?? target.target_type;
        return {
          ...target,
          ...updates,
          target_id:
            updates.target_type && updates.target_type !== target.target_type
              ? ''
              : updates.target_id ?? target.target_id,
          target_type: nextTargetType,
        };
      })
    );
  };

  const addCreateTarget = () => {
    setCreateTargets((current) => [
      ...current,
      {
        target_type: 'team',
        target_id: '',
        role_mode: 'member',
      },
    ]);
  };

  const removeCreateTarget = (index: number) => {
    setCreateTargets((current) => {
      if (current.length <= 1) return current;
      return current.filter((_, targetIndex) => targetIndex !== index);
    });
  };

  const startEditPolicy = (policy: AdminMembershipPolicy) => {
    setEditingPolicyId(policy.id);
    setCreateName(policy.name);
    setCreateDescription(policy.description || '');
    setCreateScopeType(policy.scope_type);
    setCreateScopeTeamId(policy.team_id || '');
    setCreateSourceType(policy.source_type);
    setCreateSourceConfig(JSON.stringify(policy.source_config || {}, null, 2));
    setCreateEnabled(policy.enabled);
    setCreatePriority(String(policy.priority));
    setCreateTargets(
      policy.targets.length > 0
        ? policy.targets.map((target) => ({
            target_type: target.target_type,
            target_id: target.target_id,
            role_mode: target.role_mode,
          }))
        : [
            {
              target_type: 'team',
              target_id: '',
              role_mode: 'member',
            },
          ]
    );
    setError(null);
    setNotice(null);
  };

  const createPolicy = async () => {
    const token = authStore.token;
    if (!token) return;

    const name = createName().trim();
    if (!name) {
      setError('Policy name is required.');
      return;
    }

    if (createScopeType() === 'team' && !createScopeTeamId().trim()) {
      setError('Select a scope team for team-scoped policies.');
      return;
    }

    const priority = Number(createPriority());
    if (!Number.isFinite(priority)) {
      setError('Priority must be a valid number.');
      return;
    }

    let sourceConfig: unknown = {};
    const sourceConfigValue = createSourceConfig().trim();
    if (sourceConfigValue.length > 0) {
      try {
        sourceConfig = JSON.parse(sourceConfigValue);
      } catch {
        setError('Source config must be valid JSON.');
        return;
      }
    }

    const targets = createTargets().map((target) => ({
      target_type: target.target_type,
      target_id: target.target_id.trim(),
      role_mode: target.role_mode,
    }));

    if (targets.length === 0) {
      setError('At least one target is required.');
      return;
    }

    if (targets.some((target) => target.target_id.length === 0)) {
      setError('Select a target for each target row.');
      return;
    }

    const targetKeys = new Set<string>();
    for (const target of targets) {
      const key = `${target.target_type}:${target.target_id}`;
      if (targetKeys.has(key)) {
        setError(`Duplicate target selected (${key}).`);
        return;
      }
      targetKeys.add(key);
    }

    setIsCreating(true);
    setError(null);
    setNotice(null);

    try {
      const editingId = editingPolicyId();
      if (editingId) {
        const updated = await fetchAdminJson<AdminMembershipPolicy>(
          token,
          `/admin/membership-policies/${editingId}`,
          {
            method: 'PUT',
            body: JSON.stringify({
              name,
              description: createDescription().trim() || undefined,
              enabled: createEnabled(),
              priority,
              source_config: sourceConfig,
              targets,
            }),
          }
        );
        resetCreatePolicyForm();
        await loadMembershipData();
        setNotice(`Policy "${updated.name}" updated successfully.`);
      } else {
        const created = await fetchAdminJson<AdminMembershipPolicy>(token, '/admin/membership-policies', {
          method: 'POST',
          body: JSON.stringify({
            name,
            description: createDescription().trim() || undefined,
            scope_type: createScopeType(),
            team_id: createScopeType() === 'team' ? createScopeTeamId() : undefined,
            source_type: createSourceType(),
            source_config: sourceConfig,
            enabled: createEnabled(),
            priority,
            targets,
          }),
        });

        resetCreatePolicyForm();
        await loadMembershipData();
        setNotice(`Policy "${created.name}" created successfully.`);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save policy');
    } finally {
      setIsCreating(false);
    }
  };

  const triggerUserResync = async () => {
    const token = authStore.token;
    if (!token) return;
    const userId = resyncUserId().trim();
    if (!userId) {
      setError('Enter a user ID to run membership re-sync.');
      return;
    }

    setIsResyncing(true);
    setError(null);
    setNotice(null);
    try {
      const result = await fetchAdminJson<{
        memberships_applied: number;
        memberships_failed: number;
        teams_processed: number;
      }>(token, `/admin/membership-policies/users/${userId}/resync`, {
        method: 'POST',
      });
      setNotice(
        `Re-sync completed: ${result.memberships_applied} applied, ${result.memberships_failed} failed across ${result.teams_processed} team(s).`
      );
      await loadMembershipData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to run user re-sync');
    } finally {
      setIsResyncing(false);
    }
  };

  return (
    <div class="space-y-4">
      <Show when={error()}>
        <div class="rounded-lg border border-danger/30 bg-danger/10 px-4 py-3 text-sm text-danger">
          {error()}
        </div>
      </Show>

      <Show when={notice()}>
        <div class="rounded-lg border border-success/30 bg-success/10 px-4 py-3 text-sm text-success">
          {notice()}
        </div>
      </Show>

      <Show when={!isLoading()} fallback={<p class="text-sm text-text-3">Loading membership policies...</p>}>
        <div class="grid grid-cols-1 gap-3 md:grid-cols-3">
          <OverviewCard label="Policies" value={String(policies().length)} />
          <OverviewCard label="Operations (24h)" value={String(summary()?.total_operations_24h ?? 0)} />
          <OverviewCard label="Failure Rate (24h)" value={`${(summary()?.failure_rate_24h ?? 0).toFixed(1)}%`} />
        </div>

        <Show when={metadata()}>
          <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4 space-y-3">
            <h3 class="text-sm font-semibold text-text-1">Policy Structure</h3>
            <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
              <div>
                <p class="text-xs uppercase tracking-wide text-text-3 mb-1">Source Types</p>
                <p class="text-sm text-text-2">
                  {(metadata()?.source_types || []).map((item) => item.label).join(', ') || 'N/A'}
                </p>
              </div>
              <div>
                <p class="text-xs uppercase tracking-wide text-text-3 mb-1">Scope Types</p>
                <p class="text-sm text-text-2">
                  {(metadata()?.scope_types || []).map((item) => item.label).join(', ') || 'N/A'}
                </p>
              </div>
              <div>
                <p class="text-xs uppercase tracking-wide text-text-3 mb-1">Target Types</p>
                <p class="text-sm text-text-2">
                  {(metadata()?.target_types || []).map((item) => item.label).join(', ') || 'N/A'}
                </p>
              </div>
              <div>
                <p class="text-xs uppercase tracking-wide text-text-3 mb-1">Role Modes</p>
                <p class="text-sm text-text-2">
                  {(metadata()?.role_modes || []).map((item) => item.label).join(', ') || 'N/A'}
                </p>
              </div>
            </div>
          </div>
        </Show>

        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4 space-y-4">
          <div class="flex items-center justify-between gap-3">
            <h3 class="text-sm font-semibold text-text-1">
              {editingPolicyId() ? 'Edit Policy' : 'Create Policy'}
            </h3>
            <button
              type="button"
              class="rounded-lg border border-border-1 px-3 py-1.5 text-xs font-medium text-text-2 hover:bg-bg-surface-2"
              onClick={resetCreatePolicyForm}
              disabled={isCreating()}
            >
              {editingPolicyId() ? 'Cancel Edit' : 'Reset Form'}
            </button>
          </div>
          <Show when={editingPolicyId()}>
            <p class="text-xs text-text-3">
              Scope and source type are locked for existing policies. Edit name, targets, priority, source config, and enabled state.
            </p>
          </Show>

          <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
            <label class="block">
              <span class="block text-xs uppercase tracking-wide text-text-3 mb-1">Policy Name</span>
              <input
                type="text"
                value={createName()}
                onInput={(event) => setCreateName(event.currentTarget.value)}
                placeholder="e.g. Default RustChat Membership"
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
              />
            </label>

            <label class="block">
              <span class="block text-xs uppercase tracking-wide text-text-3 mb-1">Scope Type</span>
              <select
                value={createScopeType()}
                onChange={(event) => {
                  const next = event.currentTarget.value as 'global' | 'team';
                  setCreateScopeType(next);
                  if (next === 'global') {
                    setCreateScopeTeamId('');
                  }
                }}
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                disabled={Boolean(editingPolicyId())}
              >
                <option value="global">Global</option>
                <option value="team">Team</option>
              </select>
            </label>

            <Show when={createScopeType() === 'team'}>
              <label class="block md:col-span-2">
                <span class="block text-xs uppercase tracking-wide text-text-3 mb-1">Scope Team</span>
                <select
                  value={createScopeTeamId()}
                  onChange={(event) => setCreateScopeTeamId(event.currentTarget.value)}
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                >
                  <option value="">Select team</option>
                  <For each={availableTeams()}>
                    {(team) => (
                      <option value={team.id}>{team.display_name || team.name}</option>
                    )}
                  </For>
                </select>
              </label>
            </Show>

            <label class="block">
              <span class="block text-xs uppercase tracking-wide text-text-3 mb-1">Source Type</span>
              <select
                value={createSourceType()}
                onChange={(event) =>
                  setCreateSourceType(
                    event.currentTarget.value as
                      | 'all_users'
                      | 'auth_service'
                      | 'group'
                      | 'role'
                      | 'org'
                  )
                }
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                disabled={Boolean(editingPolicyId())}
              >
                <For each={sourceTypeOptions()}>
                  {(sourceType) => (
                    <option value={sourceType.value}>{sourceType.label}</option>
                  )}
                </For>
              </select>
            </label>

            <label class="block">
              <span class="block text-xs uppercase tracking-wide text-text-3 mb-1">Priority</span>
              <input
                type="number"
                value={createPriority()}
                onInput={(event) => setCreatePriority(event.currentTarget.value)}
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
              />
            </label>

            <label class="block md:col-span-2">
              <span class="block text-xs uppercase tracking-wide text-text-3 mb-1">Description (optional)</span>
              <input
                type="text"
                value={createDescription()}
                onInput={(event) => setCreateDescription(event.currentTarget.value)}
                placeholder="What this policy does"
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
              />
            </label>

            <label class="block md:col-span-2">
              <span class="block text-xs uppercase tracking-wide text-text-3 mb-1">Source Config (JSON)</span>
              <textarea
                value={createSourceConfig()}
                onInput={(event) => setCreateSourceConfig(event.currentTarget.value)}
                placeholder={createSourceConfigPlaceholder()}
                rows={5}
                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 font-mono text-xs text-text-1"
              />
            </label>
          </div>

          <label class="inline-flex items-center gap-2 text-sm text-text-2">
            <input
              type="checkbox"
              checked={createEnabled()}
              onChange={(event) => setCreateEnabled(event.currentTarget.checked)}
              class="h-4 w-4 rounded border-border-1 bg-bg-app"
            />
            Policy enabled
          </label>

          <div class="rounded-lg border border-border-1 p-3 space-y-3">
            <div class="flex items-center justify-between">
              <p class="text-sm font-medium text-text-1">Targets</p>
              <button
                type="button"
                class="rounded-lg border border-border-1 px-2.5 py-1 text-xs font-medium text-text-2 hover:bg-bg-surface-2"
                onClick={addCreateTarget}
                disabled={isCreating()}
              >
                Add Target
              </button>
            </div>

            <For each={createTargets()}>
              {(target, index) => (
                <div class="grid grid-cols-1 gap-2 rounded-lg border border-border-1 p-3 md:grid-cols-[140px_minmax(0,1fr)_140px_auto]">
                  <select
                    value={target.target_type}
                    onChange={(event) =>
                      updateCreateTarget(index(), {
                        target_type: event.currentTarget.value as 'team' | 'channel',
                      })
                    }
                    class="rounded-lg border border-border-1 bg-bg-app px-2 py-2 text-sm text-text-1"
                  >
                    <option value="team">Team</option>
                    <option value="channel">Channel</option>
                  </select>

                  <select
                    value={target.target_id}
                    onChange={(event) =>
                      updateCreateTarget(index(), { target_id: event.currentTarget.value })
                    }
                    class="rounded-lg border border-border-1 bg-bg-app px-2 py-2 text-sm text-text-1"
                  >
                    <option value="">
                      {target.target_type === 'team' ? 'Select team target' : 'Select channel target'}
                    </option>
                    <Show when={target.target_type === 'team'} fallback={
                      <For each={availableChannels()}>
                        {(channel) => (
                          <option value={channel.id}>
                            {(channel.display_name || channel.name) + ` (${channel.name})`}
                          </option>
                        )}
                      </For>
                    }>
                      <For each={availableTeams()}>
                        {(team) => (
                          <option value={team.id}>{team.display_name || team.name}</option>
                        )}
                      </For>
                    </Show>
                  </select>

                  <select
                    value={target.role_mode}
                    onChange={(event) =>
                      updateCreateTarget(index(), {
                        role_mode: event.currentTarget.value as 'member' | 'admin',
                      })
                    }
                    class="rounded-lg border border-border-1 bg-bg-app px-2 py-2 text-sm text-text-1"
                  >
                    <For each={roleModeOptions()}>
                      {(roleMode) => (
                        <option value={roleMode.value}>{roleMode.label}</option>
                      )}
                    </For>
                  </select>

                  <button
                    type="button"
                    class="rounded-lg border border-border-1 px-2.5 py-2 text-xs font-medium text-text-2 hover:bg-bg-surface-2 disabled:opacity-60"
                    onClick={() => removeCreateTarget(index())}
                    disabled={isCreating() || createTargets().length <= 1}
                  >
                    Remove
                  </button>
                </div>
              )}
            </For>
          </div>

          <div class="flex items-center gap-2">
            <button
              type="button"
              class="rounded-lg bg-brand px-4 py-2 text-sm font-medium text-white hover:bg-brand-hover disabled:opacity-60"
              onClick={() => {
                void createPolicy();
              }}
              disabled={isCreating()}
            >
              {isCreating() ? 'Saving...' : editingPolicyId() ? 'Save Policy' : 'Create Policy'}
            </button>
          </div>
        </div>

        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4 space-y-3">
          <h3 class="text-sm font-semibold text-text-1">Manual User Re-Sync</h3>
          <p class="text-sm text-text-3">
            Re-apply auto-membership policies for a specific user across their teams.
          </p>
          <div class="grid grid-cols-1 gap-2 md:grid-cols-[minmax(0,1fr)_auto]">
            <input
              type="text"
              value={resyncUserId()}
              onInput={(event) => setResyncUserId(event.currentTarget.value)}
              placeholder="User UUID"
              class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
            />
            <button
              type="button"
              class="rounded-lg border border-border-1 px-3 py-2 text-sm font-medium text-text-2 hover:bg-bg-surface-2 disabled:opacity-60"
              onClick={() => {
                void triggerUserResync();
              }}
              disabled={isResyncing()}
            >
              {isResyncing() ? 'Running...' : 'Run Re-Sync'}
            </button>
          </div>
        </div>

        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-4 space-y-3">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-semibold text-text-1">Policy List</h3>
            <button
              type="button"
              class="rounded-lg border border-border-1 px-3 py-1.5 text-sm font-medium text-text-2 hover:bg-bg-surface-2"
              onClick={() => {
                void loadMembershipData();
              }}
            >
              Reload
            </button>
          </div>

          <Show when={policies().length > 0} fallback={<p class="text-sm text-text-3">No membership policies found.</p>}>
            <div class="space-y-2">
              <For each={policies()}>
                {(policy) => (
                  <div class="rounded-lg border border-border-1 px-3 py-3">
                    <div class="flex flex-wrap items-center justify-between gap-3">
                      <div class="space-y-1">
                        <p class="text-sm font-medium text-text-1">{policy.name}</p>
                        <p class="text-xs text-text-3">
                          {policy.scope_type} · {policy.source_type} · {policy.targets.length} targets
                        </p>
                        <Show when={policy.description}>
                          <p class="text-xs text-text-3">{policy.description}</p>
                        </Show>
                        <Show when={policy.targets.length > 0}>
                          <p class="text-xs text-text-3">
                            Targets:{' '}
                            {policy.targets
                              .map((target) => `${target.target_type}:${target.target_id}`)
                              .join(', ')}
                          </p>
                        </Show>
                      </div>

                      <div class="flex items-center gap-2">
                        <span
                          class={`rounded-full px-2 py-1 text-xs font-medium ${
                            policy.enabled ? 'bg-success/15 text-success' : 'bg-warning/20 text-warning'
                          }`}
                        >
                          {policy.enabled ? 'Enabled' : 'Disabled'}
                        </span>
                        <button
                          type="button"
                          class="rounded-lg border border-border-1 px-3 py-1.5 text-xs font-medium text-text-2 hover:bg-bg-surface-2"
                          onClick={() => {
                            startEditPolicy(policy);
                          }}
                        >
                          Edit
                        </button>
                        <button
                          type="button"
                          class="rounded-lg border border-border-1 px-3 py-1.5 text-xs font-medium text-text-2 hover:bg-bg-surface-2 disabled:opacity-60"
                          onClick={() => {
                            void togglePolicyEnabled(policy);
                          }}
                          disabled={isMutatingPolicyId() === policy.id}
                        >
                          {isMutatingPolicyId() === policy.id
                            ? 'Updating...'
                            : policy.enabled
                              ? 'Disable'
                              : 'Enable'}
                        </button>
                      </div>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
}

function AdminAuditSection() {
  const [logs, setLogs] = createSignal<AdminAuditLog[]>([]);
  const [isLoading, setIsLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);

  const loadAuditLogs = async () => {
    const token = authStore.token;
    if (!token) return;

    setIsLoading(true);
    setError(null);

    try {
      const data = await fetchAdminJson<AdminAuditLog[]>(token, '/admin/audit?page=1&per_page=50');
      setLogs(data || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load audit logs');
    } finally {
      setIsLoading(false);
    }
  };

  createEffect(() => {
    if (authStore.isAuthenticated) {
      void loadAuditLogs();
    }
  });

  return (
    <div class="space-y-4">
      <Show when={error()}>
        <div class="rounded-lg border border-danger/30 bg-danger/10 px-4 py-3 text-sm text-danger">
          {error()}
        </div>
      </Show>

      <div class="overflow-x-auto rounded-xl border border-border-1 bg-bg-surface-1">
        <table class="w-full text-left text-sm">
          <thead class="border-b border-border-1 bg-bg-surface-2 text-text-3">
            <tr>
              <th class="px-4 py-3 font-medium">Action</th>
              <th class="px-4 py-3 font-medium">Target</th>
              <th class="px-4 py-3 font-medium">Actor</th>
              <th class="px-4 py-3 font-medium">Timestamp</th>
            </tr>
          </thead>
          <tbody>
            <Show when={!isLoading()} fallback={<tr><td colSpan={4} class="px-4 py-4 text-text-3">Loading audit logs...</td></tr>}>
              <Show when={logs().length > 0} fallback={<tr><td colSpan={4} class="px-4 py-4 text-text-3">No audit logs found.</td></tr>}>
                <For each={logs()}>
                  {(log) => (
                    <tr class="border-b border-border-1/60 last:border-b-0">
                      <td class="px-4 py-3 text-text-1">{log.action}</td>
                      <td class="px-4 py-3 text-text-2">
                        {log.target_type}
                        <Show when={log.target_id}>
                          <span class="ml-1 text-text-3">({log.target_id})</span>
                        </Show>
                      </td>
                      <td class="px-4 py-3 text-text-2">{log.actor_user_id || 'system'}</td>
                      <td class="px-4 py-3 text-text-2">{formatDateTime(log.created_at)}</td>
                    </tr>
                  )}
                </For>
              </Show>
            </Show>
          </tbody>
        </table>
      </div>
    </div>
  );
}

export default function Admin() {
  const location = useLocation();
  const navigate = useNavigate();
  const adminAccessState = createMemo<'loading' | 'allowed' | 'denied'>(() => {
    if (!authStore.isAuthenticated) return 'denied';
    const currentUser = authStore.user();
    if (!currentUser) return 'loading';
    return isAdminRole(currentUser.role) ? 'allowed' : 'denied';
  });

  createEffect(() => {
    if (adminAccessState() === 'denied') {
      navigate('/', { replace: true });
    }
  });

  const activeSectionId = createMemo(() => {
    const suffix = location.pathname.replace(/^\/admin\/?/, '');
    return suffix.split('/')[0] || '';
  });

  const activeSection = createMemo(
    () => sections.find((section) => section.id === activeSectionId()) || sections[0]
  );

  return (
    <Show
      when={adminAccessState() === 'allowed'}
      fallback={
        <div class="p-6 lg:p-8">
          <p class="text-sm text-text-3">Checking admin access...</p>
        </div>
      }
    >
      <div class="p-6 lg:p-8 space-y-6" data-testid="admin-page">
        <div class="grid grid-cols-1 gap-6 xl:grid-cols-[260px_minmax(0,1fr)]">
          <aside class="rounded-xl border border-border-1 bg-bg-surface-1 p-3 space-y-3 h-fit">
            <A
              href="/"
              class="block rounded-lg border border-border-1 px-3 py-2 text-sm font-medium text-text-2 hover:bg-bg-surface-2 hover:text-text-1"
            >
              Back to Chat
            </A>
            <div class="space-y-1">
              <h1 class="text-lg font-bold text-text-1">Admin Console</h1>
              <p class="text-xs text-text-3">System administration sections</p>
            </div>
            <div class="space-y-1">
              <For each={sections}>
                {(section) => {
                  const href = section.id ? `/admin/${section.id}` : '/admin';
                  return (
                    <A
                      href={href}
                      class={`block rounded-lg px-3 py-2 text-sm transition-colors ${
                        activeSectionId() === section.id
                          ? 'bg-brand/10 text-brand font-medium'
                          : 'text-text-2 hover:bg-bg-surface-2 hover:text-text-1'
                      }`}
                    >
                      {section.label}
                    </A>
                  );
                }}
              </For>
            </div>
          </aside>

          <div class="space-y-4">
            <div class="space-y-1">
              <h2 class="text-2xl font-bold text-text-1">{activeSection().label}</h2>
              <p class="text-text-3">{activeSection().description}</p>
            </div>

            <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-6 space-y-4">
              <Show when={activeSectionId() === ''}>
                <AdminOverviewSection />
              </Show>

              <Show when={activeSectionId() === 'audit-dashboard'}>
                <AdminOverviewSection />
              </Show>

              <Show when={activeSectionId() === 'users'}>
                <AdminUsersSection />
              </Show>

              <Show when={activeSectionId() === 'teams'}>
                <AdminTeamsSection />
              </Show>

              <Show when={activeSectionId() === 'settings'}>
                <AdminSettingsSection />
              </Show>

              <Show when={activeSectionId() === 'security'}>
                <AdminSecuritySection />
              </Show>

              <Show when={activeSectionId() === 'permissions'}>
                <AdminSecuritySection />
              </Show>

              <Show when={activeSectionId() === 'sso'}>
                <AdminSecuritySection />
              </Show>

              <Show when={activeSectionId() === 'integrations'}>
                <AdminSettingsSection />
              </Show>

              <Show when={activeSectionId() === 'email'}>
                <AdminEmailSection />
              </Show>

              <Show when={activeSectionId() === 'membership-policies'}>
                <AdminMembershipPoliciesSection />
              </Show>

              <Show when={activeSectionId() === 'compliance'}>
                <AdminComplianceSection />
              </Show>

              <Show when={activeSectionId() === 'audit'}>
                <AdminAuditSection />
              </Show>

              <Show when={activeSectionId() === 'health'}>
                <AdminOverviewSection />
              </Show>
            </div>

            <Show when={activeSectionId() !== ''}>
              <div class="text-sm text-text-3">
                Current route: <code class="text-text-2">{location.pathname}</code>
              </div>
            </Show>
          </div>
        </div>
      </div>
    </Show>
  );
}
