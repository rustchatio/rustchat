import { For, Show, createMemo, createSignal, onMount } from 'solid-js';
import { useNavigate, useParams } from '@solidjs/router';
import { getErrorMessage } from '@/api/client';
import { playbooksApi, type RunStatusUpdate, type RunTask, type RunWithTasks } from '@/api/playbooks';
import { Button } from '@/components/ui/Button';
import { toast } from '@/hooks/useToast';

interface TaskReference {
  title: string;
  description: string | null;
}

function formatDate(value: string): string {
  return new Date(value).toLocaleString();
}

export default function PlaybookRunRoute() {
  const navigate = useNavigate();
  const params = useParams<{ id: string }>();
  const runId = () => params.id;

  const [run, setRun] = createSignal<RunWithTasks | null>(null);
  const [updates, setUpdates] = createSignal<RunStatusUpdate[]>([]);
  const [taskMap, setTaskMap] = createSignal<Record<string, TaskReference>>({});
  const [newUpdate, setNewUpdate] = createSignal('');
  const [isLoading, setIsLoading] = createSignal(true);
  const [isPosting, setIsPosting] = createSignal(false);
  const [updatingTaskId, setUpdatingTaskId] = createSignal<string | null>(null);
  const [finishingRun, setFinishingRun] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const progressPercent = createMemo(() => {
    const current = run();
    if (!current || current.progress.total <= 0) return 0;
    return Math.round((current.progress.completed / current.progress.total) * 100);
  });

  const loadRun = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const currentRun = await playbooksApi.getRun(runId());
      setRun(currentRun);

      const playbook = await playbooksApi.get(currentRun.run.playbook_id);
      const mapping: Record<string, TaskReference> = {};
      for (const checklist of playbook.checklists) {
        for (const task of checklist.tasks) {
          mapping[task.id] = {
            title: task.title,
            description: task.description,
          };
        }
      }
      setTaskMap(mapping);

      const statusUpdates = await playbooksApi.listStatusUpdates(currentRun.run.id);
      setUpdates(statusUpdates);
    } catch (err) {
      setError(getErrorMessage(err) || 'Failed to load playbook run.');
    } finally {
      setIsLoading(false);
    }
  };

  const toggleTask = async (task: RunTask) => {
    const current = run();
    if (!current) return;

    const nextStatus = task.status === 'done' ? 'pending' : 'done';
    setUpdatingTaskId(task.id);
    try {
      await playbooksApi.updateRunTask(current.run.id, task.task_id, { status: nextStatus });
      const refreshed = await playbooksApi.getRun(current.run.id);
      setRun(refreshed);
    } catch (err) {
      toast.error('Unable to update task', getErrorMessage(err) || 'Please try again.');
    } finally {
      setUpdatingTaskId(null);
    }
  };

  const postUpdate = async () => {
    const current = run();
    const message = newUpdate().trim();
    if (!current || !message) {
      return;
    }

    setIsPosting(true);
    try {
      const created = await playbooksApi.createStatusUpdate(current.run.id, {
        message,
        is_broadcast: false,
      });
      setUpdates((prev) => [created, ...prev]);
      setNewUpdate('');
    } catch (err) {
      toast.error('Unable to post update', getErrorMessage(err) || 'Please try again.');
    } finally {
      setIsPosting(false);
    }
  };

  const finishRun = async () => {
    const current = run();
    if (!current || current.run.status === 'finished') {
      return;
    }

    setFinishingRun(true);
    try {
      await playbooksApi.finishRun(current.run.id);
      const refreshed = await playbooksApi.getRun(current.run.id);
      setRun(refreshed);
      toast.success('Run finished', 'This run has been marked as finished.');
    } catch (err) {
      toast.error('Unable to finish run', getErrorMessage(err) || 'Please try again.');
    } finally {
      setFinishingRun(false);
    }
  };

  onMount(() => {
    void loadRun();
  });

  return (
    <div class="h-full flex flex-col bg-bg-app">
      <Show
        when={!isLoading()}
        fallback={
          <div class="h-full flex items-center justify-center">
            <div class="w-10 h-10 rounded-full border-4 border-brand/20 border-t-brand animate-spin" />
          </div>
        }
      >
        <Show
          when={run()}
          fallback={
            <div class="p-6">
              <div class="max-w-xl rounded-xl border border-danger/30 bg-danger/10 px-4 py-3 space-y-3">
                <p class="text-sm text-danger">{error() || 'Run not found.'}</p>
                <div class="flex gap-2">
                  <Button variant="secondary" onClick={() => void loadRun()}>
                    Retry
                  </Button>
                  <Button variant="ghost" onClick={() => navigate('/playbooks')}>
                    Back to Playbooks
                  </Button>
                </div>
              </div>
            </div>
          }
        >
          {(currentRun) => (
            <>
              <header class="border-b border-border-1 bg-bg-surface-1 px-6 py-4 space-y-4">
                <div class="flex items-center justify-between gap-3">
                  <div class="flex items-center gap-3">
                    <Button variant="ghost" size="sm" onClick={() => navigate('/playbooks')}>
                      Back
                    </Button>
                    <div>
                      <div class="flex items-center gap-2">
                        <h1 class="text-xl font-semibold text-text-1">{currentRun().run.name}</h1>
                        <span class="rounded-full border border-border-1 px-2 py-0.5 text-xs font-medium text-text-2 capitalize">
                          {currentRun().run.status.replace('_', ' ')}
                        </span>
                      </div>
                      <p class="text-sm text-text-3">
                        Started {formatDate(currentRun().run.started_at)}
                      </p>
                    </div>
                  </div>
                  <Button
                    variant="secondary"
                    loading={finishingRun()}
                    disabled={currentRun().run.status === 'finished'}
                    onClick={() => void finishRun()}
                  >
                    Finish Run
                  </Button>
                </div>

                <div>
                  <div class="h-2.5 w-full rounded-full bg-bg-app">
                    <div
                      class="h-2.5 rounded-full bg-brand transition-all"
                      style={{ width: `${progressPercent()}%` }}
                    />
                  </div>
                  <div class="mt-1 flex justify-between text-xs text-text-3">
                    <span>{progressPercent()}% complete</span>
                    <span>
                      {currentRun().progress.completed}/{currentRun().progress.total} tasks
                    </span>
                  </div>
                </div>
              </header>

              <div class="flex-1 min-h-0 grid grid-cols-1 lg:grid-cols-[1fr_340px]">
                <section class="overflow-y-auto p-6 space-y-3">
                  <For each={currentRun().tasks}>
                    {(task) => (
                      <article class="rounded-xl border border-border-1 bg-bg-surface-1 p-4 flex items-start gap-3">
                        <button
                          type="button"
                          class={`mt-0.5 h-5 w-5 rounded-full border text-xs font-semibold transition-colors ${
                            task.status === 'done'
                              ? 'border-success bg-success text-white'
                              : 'border-border-2 text-text-3 hover:border-brand hover:text-brand'
                          }`}
                          onClick={() => void toggleTask(task)}
                          disabled={updatingTaskId() === task.id}
                          aria-label={task.status === 'done' ? 'Mark as pending' : 'Mark as done'}
                        >
                          {task.status === 'done' ? '✓' : ''}
                        </button>

                        <div class="min-w-0 flex-1">
                          <p
                            class={`text-sm font-medium ${
                              task.status === 'done'
                                ? 'text-text-3 line-through'
                                : 'text-text-1'
                            }`}
                          >
                            {taskMap()[task.task_id]?.title || 'Task'}
                          </p>
                          <Show when={taskMap()[task.task_id]?.description}>
                            <p class="text-sm text-text-3 mt-1">
                              {taskMap()[task.task_id]?.description}
                            </p>
                          </Show>
                          <Show when={task.notes}>
                            <p class="text-xs text-text-3 mt-2 italic">{task.notes}</p>
                          </Show>
                        </div>
                      </article>
                    )}
                  </For>
                </section>

                <aside class="border-l border-border-1 bg-bg-surface-1 flex flex-col min-h-0">
                  <div class="px-4 py-3 border-b border-border-1">
                    <h2 class="text-base font-semibold text-text-1">Timeline</h2>
                  </div>

                  <div class="flex-1 overflow-y-auto p-4 space-y-3">
                    <Show
                      when={updates().length > 0}
                      fallback={<p class="text-sm text-text-3">No updates posted yet.</p>}
                    >
                      <For each={updates()}>
                        {(update) => (
                          <article class="rounded-lg border border-border-1 bg-bg-app px-3 py-2">
                            <p class="text-sm text-text-1">{update.message}</p>
                            <p class="text-xs text-text-3 mt-1">
                              {formatDate(update.created_at)}
                            </p>
                          </article>
                        )}
                      </For>
                    </Show>
                  </div>

                  <form
                    class="border-t border-border-1 p-4 flex gap-2"
                    onSubmit={(event) => {
                      event.preventDefault();
                      void postUpdate();
                    }}
                  >
                    <input
                      type="text"
                      value={newUpdate()}
                      onInput={(event) => setNewUpdate(event.currentTarget.value)}
                      placeholder="Post an update..."
                      class="flex-1 rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                    />
                    <Button
                      variant="primary"
                      size="sm"
                      loading={isPosting()}
                      disabled={!newUpdate().trim()}
                      type="submit"
                    >
                      Send
                    </Button>
                  </form>
                </aside>
              </div>
            </>
          )}
        </Show>
      </Show>
    </div>
  );
}
