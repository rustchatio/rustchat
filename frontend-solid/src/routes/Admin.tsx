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

const sections: AdminSection[] = [
  { id: '', label: 'Overview', description: 'System summary and quick links' },
  { id: 'users', label: 'Users', description: 'Manage users and account lifecycle' },
  { id: 'teams', label: 'Teams', description: 'Manage teams and channels' },
  { id: 'settings', label: 'Server Settings', description: 'Server and platform configuration' },
  { id: 'security', label: 'Security', description: 'Authentication and access controls' },
  { id: 'compliance', label: 'Compliance', description: 'Retention, exports, and compliance controls' },
  { id: 'audit', label: 'Audit Logs', description: 'Administrative events and audit trail' },
];

function formatDateTime(dateValue?: string | null): string {
  if (!dateValue) return 'Never';
  const parsed = new Date(dateValue);
  if (Number.isNaN(parsed.getTime())) return 'Never';
  return parsed.toLocaleString();
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
  const headers = new Headers(init.headers);
  headers.set('Authorization', `Bearer ${token}`);
  if (!(init.body instanceof FormData)) {
    headers.set('Content-Type', 'application/json');
  }

  const response = await fetch(`/api/v1${path}`, {
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
  const [error, setError] = createSignal<string | null>(null);
  const [search, setSearch] = createSignal('');
  const [status, setStatus] = createSignal<'all' | 'active' | 'inactive'>('all');

  const loadUsers = async () => {
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
      if (status() !== 'all') {
        params.set('status', status());
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

    try {
      const endpoint = user.is_active
        ? `/admin/users/${user.id}/deactivate`
        : `/admin/users/${user.id}/reactivate`;
      await fetchAdminJson<{ status: string }>(token, endpoint, { method: 'POST' });
      await loadUsers();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update user status');
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
                        <span class={`rounded-full px-2 py-1 text-xs font-medium ${user.is_active ? 'bg-success/15 text-success' : 'bg-warning/20 text-warning'}`}>
                          {user.is_active ? 'Active' : 'Inactive'}
                        </span>
                      </td>
                      <td class="px-4 py-3 text-text-2">{formatDateTime(user.last_login_at)}</td>
                      <td class="px-4 py-3 text-right">
                        <button
                          type="button"
                          class={`rounded-lg px-3 py-1.5 text-xs font-medium ${user.is_active ? 'bg-warning/20 text-warning hover:bg-warning/30' : 'bg-success/20 text-success hover:bg-success/30'}`}
                          onClick={() => {
                            void toggleUserActive(user);
                          }}
                        >
                          {user.is_active ? 'Deactivate' : 'Reactivate'}
                        </button>
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
      const data = await fetchAdminJson<AdminConfig>(token, '/admin/config');
      setConfig(data);
      setSiteName(String(data.site?.site_name || ''));
      setSiteUrl(String(data.site?.site_url || ''));
      setSupportEmail(String(data.site?.support_email || ''));
      setAllowRegistration(Boolean(data.authentication?.allow_registration));
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
  const [isLoading, setIsLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);

  const permissionsByCategory = createMemo(() => {
    const categories = new Map<string, number>();
    for (const permission of permissions()) {
      const category = permission.category || 'uncategorized';
      categories.set(category, (categories.get(category) || 0) + 1);
    }
    return Array.from(categories.entries()).sort((a, b) => b[1] - a[1]);
  });

  const loadSecurityData = async () => {
    const token = authStore.token;
    if (!token) return;

    setIsLoading(true);
    setError(null);

    try {
      const [ssoData, permissionsData, systemAdminData] = await Promise.all([
        fetchAdminJson<AdminSsoConfig[]>(token, '/admin/sso'),
        fetchAdminJson<AdminPermission[]>(token, '/admin/permissions'),
        fetchAdminJson<string[]>(token, '/admin/roles/system_admin/permissions'),
      ]);
      setSsoConfigs(ssoData || []);
      setPermissions(permissionsData || []);
      setSystemAdminPermissions(systemAdminData || []);
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

  return (
    <div class="space-y-4">
      <Show when={error()}>
        <div class="rounded-lg border border-danger/30 bg-danger/10 px-4 py-3 text-sm text-danger">
          {error()}
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
        <div class="space-y-1">
          <h1 class="text-2xl font-bold text-text-1">Admin Console</h1>
          <p class="text-text-3">
            Centralized administration for users, teams, security, and system configuration.
          </p>
        </div>

        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-2 flex flex-wrap gap-2">
          <For each={sections}>
            {(section) => {
              const href = section.id ? `/admin/${section.id}` : '/admin';
              return (
                <A
                  href={href}
                  class={`px-3 py-2 rounded-lg text-sm transition-colors ${
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

        <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-6 space-y-4">
          <div>
            <h2 class="text-lg font-semibold text-text-1">{activeSection().label}</h2>
            <p class="text-sm text-text-3 mt-1">{activeSection().description}</p>
          </div>

          <Show when={activeSectionId() === ''}>
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

          <Show when={activeSectionId() === 'compliance'}>
            <AdminComplianceSection />
          </Show>

          <Show when={activeSectionId() === 'audit'}>
            <AdminAuditSection />
          </Show>
        </div>

        <Show when={activeSectionId() !== ''}>
          <div class="text-sm text-text-3">
            Current route: <code class="text-text-2">{location.pathname}</code>
          </div>
        </Show>
      </div>
    </Show>
  );
}
