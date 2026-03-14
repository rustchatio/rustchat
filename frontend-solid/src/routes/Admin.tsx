import { For, Show, createEffect, createMemo } from 'solid-js';
import { A, useLocation, useNavigate } from '@solidjs/router';
import { authStore } from '@/stores/auth';
import { isAdminRole } from '@/utils/roles';

interface AdminSection {
  id: string;
  label: string;
  description: string;
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

export default function Admin() {
  const location = useLocation();
  const navigate = useNavigate();

  createEffect(() => {
    const currentUser = authStore.user();
    if (!currentUser) return;
    if (!isAdminRole(currentUser.role)) {
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

      <div class="rounded-xl border border-border-1 bg-bg-surface-1 p-6">
        <h2 class="text-lg font-semibold text-text-1">{activeSection().label}</h2>
        <p class="text-sm text-text-3 mt-1">{activeSection().description}</p>
        <p class="text-sm text-text-2 mt-4">
          This section is available and discoverable. Detailed admin workflows can be migrated in
          iterative parity slices.
        </p>
      </div>

      <Show when={activeSectionId() !== ''}>
        <div class="text-sm text-text-3">
          Current route: <code class="text-text-2">{location.pathname}</code>
        </div>
      </Show>
    </div>
  );
}
