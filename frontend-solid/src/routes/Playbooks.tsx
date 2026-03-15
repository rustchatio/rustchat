import { For, Show, createSignal, onMount } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { getErrorMessage } from '@/api/client';
import { playbooksApi, type Playbook, type PlaybookRun } from '@/api/playbooks';
import { Button } from '@/components/ui/Button';
import { toast } from '@/hooks/useToast';
import { resolveActiveTeamId } from '@/utils/teamContext';

type PlaybooksTab = 'playbooks' | 'runs';

function formatDate(value: string): string {
  return new Date(value).toLocaleDateString();
}

export default function PlaybooksRoute() {
  const navigate = useNavigate();
  const [activeTab, setActiveTab] = createSignal<PlaybooksTab>('playbooks');
  const [teamId, setTeamId] = createSignal<string | null>(null);
  const [playbooks, setPlaybooks] = createSignal<Playbook[]>([]);
  const [runs, setRuns] = createSignal<PlaybookRun[]>([]);
  const [isLoading, setIsLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [startingPlaybookId, setStartingPlaybookId] = createSignal<string | null>(null);

  const load = async () => {
    setIsLoading(true);
    setError(null);

    try {
      const resolvedTeamId = await resolveActiveTeamId();
      if (!resolvedTeamId) {
        setError('No team is available yet. Join or create a team first.');
        return;
      }

      setTeamId(resolvedTeamId);
      const [playbookList, runList] = await Promise.all([
        playbooksApi.list(resolvedTeamId),
        playbooksApi.listRuns(resolvedTeamId),
      ]);
      setPlaybooks(playbookList);
      setRuns(runList);
    } catch (err) {
      setError(getErrorMessage(err) || 'Failed to load playbooks.');
    } finally {
      setIsLoading(false);
    }
  };

  const handleStartRun = async (playbook: Playbook) => {
    const currentTeamId = teamId();
    if (!currentTeamId) {
      toast.error('Team not available', 'Please reload and try again.');
      return;
    }

    const defaultName = `${playbook.name} Run`;
    const runName = window
      .prompt(`Name for this run of ${playbook.name}:`, defaultName)
      ?.trim();
    if (!runName) {
      return;
    }

    setStartingPlaybookId(playbook.id);
    try {
      const started = await playbooksApi.startRun(currentTeamId, {
        playbook_id: playbook.id,
        name: runName,
      });
      setRuns((prev) => [started.run, ...prev.filter((run) => run.id !== started.run.id)]);
      toast.success('Run started', `${runName} is now active.`);
      navigate(`/runs/${started.run.id}`);
    } catch (err) {
      toast.error('Unable to start run', getErrorMessage(err) || 'Please try again.');
    } finally {
      setStartingPlaybookId(null);
    }
  };

  onMount(() => {
    void load();
  });

  return (
    <div class="h-full flex flex-col bg-bg-app">
      <header class="border-b border-border-1 bg-bg-surface-1 px-6 py-4 flex items-center justify-between">
        <div>
          <h1 class="text-2xl font-semibold text-text-1">Playbooks</h1>
          <p class="text-sm text-text-3">Automate operational workflows and incident runs.</p>
        </div>
        <Button
          variant="primary"
          onClick={() => navigate('/playbooks/new')}
          disabled={!teamId()}
        >
          Create Playbook
        </Button>
      </header>

      <div class="px-6 border-b border-border-1 bg-bg-surface-1">
        <nav class="flex gap-6">
          <button
            type="button"
            class={`py-3 text-sm font-medium border-b-2 transition-colors ${
              activeTab() === 'playbooks'
                ? 'border-brand text-brand'
                : 'border-transparent text-text-3 hover:text-text-1'
            }`}
            onClick={() => setActiveTab('playbooks')}
          >
            Library
          </button>
          <button
            type="button"
            class={`py-3 text-sm font-medium border-b-2 transition-colors ${
              activeTab() === 'runs'
                ? 'border-brand text-brand'
                : 'border-transparent text-text-3 hover:text-text-1'
            }`}
            onClick={() => setActiveTab('runs')}
          >
            Runs
          </button>
        </nav>
      </div>

      <main class="flex-1 overflow-y-auto p-6">
        <Show
          when={!isLoading()}
          fallback={
            <div class="h-full flex items-center justify-center">
              <div class="w-10 h-10 rounded-full border-4 border-brand/20 border-t-brand animate-spin" />
            </div>
          }
        >
          <Show
            when={!error()}
            fallback={
              <div class="max-w-lg rounded-xl border border-danger/30 bg-danger/10 p-4 space-y-3">
                <p class="text-sm text-danger">{error()}</p>
                <div class="flex gap-2">
                  <Button variant="secondary" onClick={() => void load()}>
                    Retry
                  </Button>
                  <Button variant="ghost" onClick={() => navigate('/')}>
                    Open Workspace
                  </Button>
                </div>
              </div>
            }
          >
            <Show when={activeTab() === 'playbooks'}>
              <Show
                when={playbooks().length > 0}
                fallback={
                  <div class="rounded-xl border border-dashed border-border-2 bg-bg-surface-1 p-8 text-center">
                    <h2 class="text-lg font-semibold text-text-1">No playbooks yet</h2>
                    <p class="mt-2 text-sm text-text-3">
                      Create your first playbook to standardize runbooks for your team.
                    </p>
                  </div>
                }
              >
                <div class="grid gap-4 lg:grid-cols-2 2xl:grid-cols-3">
                  <For each={playbooks()}>
                    {(playbook) => (
                      <article class="rounded-xl border border-border-1 bg-bg-surface-1 p-5 flex flex-col gap-4">
                        <div class="flex items-start gap-3">
                          <span class="text-3xl leading-none">{playbook.icon || '📘'}</span>
                          <div class="min-w-0">
                            <h3 class="text-base font-semibold text-text-1 truncate">
                              {playbook.name}
                            </h3>
                            <p class="text-xs text-text-3 mt-1">
                              Updated {formatDate(playbook.updated_at)}
                            </p>
                          </div>
                        </div>

                        <p class="text-sm text-text-2 min-h-[40px]">
                          {playbook.description || 'No description provided.'}
                        </p>

                        <div class="mt-auto flex gap-2">
                          <Button
                            variant="primary"
                            size="sm"
                            class="flex-1"
                            loading={startingPlaybookId() === playbook.id}
                            onClick={() => void handleStartRun(playbook)}
                          >
                            Run
                          </Button>
                          <Button
                            variant="secondary"
                            size="sm"
                            class="flex-1"
                            onClick={() => navigate(`/playbooks/${playbook.id}/edit`)}
                          >
                            Edit
                          </Button>
                        </div>
                      </article>
                    )}
                  </For>
                </div>
              </Show>
            </Show>

            <Show when={activeTab() === 'runs'}>
              <Show
                when={runs().length > 0}
                fallback={
                  <div class="rounded-xl border border-dashed border-border-2 bg-bg-surface-1 p-8 text-center">
                    <h2 class="text-lg font-semibold text-text-1">No active runs</h2>
                    <p class="mt-2 text-sm text-text-3">
                      Start a run from the library tab to track checklist execution.
                    </p>
                  </div>
                }
              >
                <div class="space-y-3">
                  <For each={runs()}>
                    {(run) => (
                      <button
                        type="button"
                        class="w-full rounded-xl border border-border-1 bg-bg-surface-1 px-4 py-3 text-left hover:border-border-2 transition-colors"
                        onClick={() => navigate(`/runs/${run.id}`)}
                      >
                        <div class="flex items-center justify-between gap-4">
                          <div class="min-w-0">
                            <h3 class="text-sm font-semibold text-text-1 truncate">{run.name}</h3>
                            <p class="text-xs text-text-3 mt-1">
                              Started {new Date(run.started_at).toLocaleString()}
                            </p>
                          </div>
                          <span class="rounded-full border border-border-1 px-2 py-0.5 text-xs font-medium text-text-2 capitalize">
                            {run.status.replace('_', ' ')}
                          </span>
                        </div>
                      </button>
                    )}
                  </For>
                </div>
              </Show>
            </Show>
          </Show>
        </Show>
      </main>
    </div>
  );
}
