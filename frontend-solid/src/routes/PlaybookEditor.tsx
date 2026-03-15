import { For, Show, createMemo, createSignal, onMount } from 'solid-js';
import { useNavigate, useParams } from '@solidjs/router';
import { getErrorMessage } from '@/api/client';
import { playbooksApi, type PlaybookFull } from '@/api/playbooks';
import { Button } from '@/components/ui/Button';
import { toast } from '@/hooks/useToast';
import { resolveActiveTeamId } from '@/utils/teamContext';
import { generateUUID } from '@/utils/uuid';

interface PlaybookFormState {
  name: string;
  description: string;
  icon: string;
  isPublic: boolean;
  createChannelOnRun: boolean;
  channelNameTemplate: string;
  keywordTriggers: string;
}

interface TaskDraft {
  id: string;
  title: string;
  description: string;
  isTemp: boolean;
}

interface ChecklistDraft {
  id: string;
  name: string;
  isTemp: boolean;
  tasks: TaskDraft[];
}

interface OriginalTaskSnapshot {
  checklistId: string;
  taskId: string;
}

function createTempChecklist(name: string): ChecklistDraft {
  return {
    id: `temp-checklist-${generateUUID()}`,
    name,
    isTemp: true,
    tasks: [],
  };
}

function createTempTask(): TaskDraft {
  return {
    id: `temp-task-${generateUUID()}`,
    title: '',
    description: '',
    isTemp: true,
  };
}

function mapPlaybookToDraft(playbook: PlaybookFull): ChecklistDraft[] {
  return playbook.checklists.map((checklist) => ({
    id: checklist.id,
    name: checklist.name,
    isTemp: false,
    tasks: checklist.tasks.map((task) => ({
      id: task.id,
      title: task.title,
      description: task.description || '',
      isTemp: false,
    })),
  }));
}

export default function PlaybookEditorRoute() {
  const navigate = useNavigate();
  const params = useParams<{ id?: string }>();
  const playbookId = () => params.id;
  const isEditing = createMemo(() => Boolean(playbookId()));

  const [teamId, setTeamId] = createSignal<string | null>(null);
  const [isLoading, setIsLoading] = createSignal(true);
  const [isSaving, setIsSaving] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [warning, setWarning] = createSignal<string | null>(null);

  const [form, setForm] = createSignal<PlaybookFormState>({
    name: '',
    description: '',
    icon: '📘',
    isPublic: false,
    createChannelOnRun: true,
    channelNameTemplate: 'run-{{date}}',
    keywordTriggers: '',
  });

  const [checklists, setChecklists] = createSignal<ChecklistDraft[]>([]);
  const [originalChecklistIds, setOriginalChecklistIds] = createSignal<string[]>([]);
  const [originalTasks, setOriginalTasks] = createSignal<OriginalTaskSnapshot[]>([]);

  const updateForm = <K extends keyof PlaybookFormState>(field: K, value: PlaybookFormState[K]) => {
    setForm((prev) => ({ ...prev, [field]: value }));
  };

  const updateChecklistName = (index: number, name: string) => {
    setChecklists((prev) =>
      prev.map((checklist, currentIndex) =>
        currentIndex === index ? { ...checklist, name } : checklist
      )
    );
  };

  const removeChecklist = (index: number) => {
    setChecklists((prev) => prev.filter((_, currentIndex) => currentIndex !== index));
  };

  const addChecklist = () => {
    setChecklists((prev) => [...prev, createTempChecklist('New Checklist')]);
  };

  const addTask = (checklistIndex: number) => {
    setChecklists((prev) =>
      prev.map((checklist, currentIndex) =>
        currentIndex === checklistIndex
          ? { ...checklist, tasks: [...checklist.tasks, createTempTask()] }
          : checklist
      )
    );
  };

  const updateTask = (
    checklistIndex: number,
    taskIndex: number,
    field: keyof Pick<TaskDraft, 'title' | 'description'>,
    value: string
  ) => {
    setChecklists((prev) =>
      prev.map((checklist, currentChecklistIndex) => {
        if (currentChecklistIndex !== checklistIndex) {
          return checklist;
        }

        return {
          ...checklist,
          tasks: checklist.tasks.map((task, currentTaskIndex) =>
            currentTaskIndex === taskIndex ? { ...task, [field]: value } : task
          ),
        };
      })
    );
  };

  const removeTask = (checklistIndex: number, taskIndex: number) => {
    setChecklists((prev) =>
      prev.map((checklist, currentIndex) =>
        currentIndex === checklistIndex
          ? {
              ...checklist,
              tasks: checklist.tasks.filter((_, currentTaskIndex) => currentTaskIndex !== taskIndex),
            }
          : checklist
      )
    );
  };

  const load = async () => {
    setIsLoading(true);
    setError(null);
    setWarning(null);

    try {
      const resolvedTeamId = await resolveActiveTeamId();
      if (!resolvedTeamId) {
        setError('No team is available yet. Join or create a team first.');
        return;
      }
      setTeamId(resolvedTeamId);

      if (!isEditing()) {
        setChecklists([createTempChecklist('Default Checklist')]);
        return;
      }

      const currentPlaybookId = playbookId();
      if (!currentPlaybookId) {
        setError('Invalid playbook id.');
        return;
      }

      const playbook = await playbooksApi.get(currentPlaybookId);

      setForm({
        name: playbook.name,
        description: playbook.description || '',
        icon: playbook.icon || '📘',
        isPublic: playbook.is_public,
        createChannelOnRun: playbook.create_channel_on_run,
        channelNameTemplate: playbook.channel_name_template || 'run-{{date}}',
        keywordTriggers: playbook.keyword_triggers?.join(', ') || '',
      });

      const checklistDrafts = mapPlaybookToDraft(playbook);
      setChecklists(checklistDrafts.length > 0 ? checklistDrafts : [createTempChecklist('Default Checklist')]);
      setOriginalChecklistIds(playbook.checklists.map((checklist) => checklist.id));
      setOriginalTasks(
        playbook.checklists.flatMap((checklist) =>
          checklist.tasks.map((task) => ({
            checklistId: checklist.id,
            taskId: task.id,
          }))
        )
      );
    } catch (err) {
      setError(getErrorMessage(err) || 'Failed to load playbook.');
    } finally {
      setIsLoading(false);
    }
  };

  const syncChecklists = async (savedPlaybookId: string) => {
    const drafts = checklists();
    const keptChecklistIds = new Set<string>();
    const keptTaskIds = new Set<string>();
    const checklistIdMap = new Map<string, string>();
    let checklistRenameSkipped = false;

    for (let checklistIndex = 0; checklistIndex < drafts.length; checklistIndex += 1) {
      const checklist = drafts[checklistIndex];
      const checklistName = checklist.name.trim() || `Checklist ${checklistIndex + 1}`;
      const sortOrder = checklistIndex + 1;

      let persistedChecklistId = checklist.id;
      if (checklist.isTemp) {
        const createdChecklist = await playbooksApi.createChecklist(savedPlaybookId, {
          name: checklistName,
          sort_order: sortOrder,
        });
        persistedChecklistId = createdChecklist.id;
        checklistIdMap.set(checklist.id, persistedChecklistId);
      } else {
        keptChecklistIds.add(checklist.id);
        if (checklist.name.trim() !== checklistName) {
          checklistRenameSkipped = true;
        }
      }

      for (let taskIndex = 0; taskIndex < checklist.tasks.length; taskIndex += 1) {
        const task = checklist.tasks[taskIndex];
        const title = task.title.trim();
        if (!title) {
          continue;
        }

        const description = task.description.trim();
        const payload = {
          title,
          description: description || null,
          sort_order: taskIndex + 1,
        };

        if (task.isTemp) {
          const createdTask = await playbooksApi.createTask(persistedChecklistId, payload);
          keptTaskIds.add(createdTask.id);
        } else {
          const updatedTask = await playbooksApi.updateTask(task.id, payload);
          keptTaskIds.add(updatedTask.id);
        }
      }
    }

    const tasksGroupedByChecklist = originalTasks().reduce<Record<string, string[]>>((acc, item) => {
      acc[item.checklistId] = acc[item.checklistId] || [];
      acc[item.checklistId].push(item.taskId);
      return acc;
    }, {});

    for (const checklistId of originalChecklistIds()) {
      if (!keptChecklistIds.has(checklistId)) {
        await playbooksApi.deleteChecklist(savedPlaybookId, checklistId);
        continue;
      }

      const checklistTaskIds = tasksGroupedByChecklist[checklistId] || [];
      for (const taskId of checklistTaskIds) {
        if (!keptTaskIds.has(taskId)) {
          await playbooksApi.deleteTask(taskId);
        }
      }
    }

    if (checklistRenameSkipped || checklistIdMap.size > 0) {
      setWarning(
        checklistRenameSkipped
          ? 'Existing checklist names cannot be updated by current API. New checklists and tasks were saved.'
          : null
      );
    }
  };

  const save = async () => {
    const currentTeamId = teamId();
    if (!currentTeamId) {
      setError('No team is available yet.');
      return;
    }

    const payload = form();
    if (!payload.name.trim()) {
      setError('Playbook name is required.');
      return;
    }

    if (checklists().length === 0) {
      setError('Add at least one checklist before saving.');
      return;
    }

    setIsSaving(true);
    setError(null);
    setWarning(null);

    try {
      const keywordTriggers = payload.keywordTriggers
        .split(',')
        .map((keyword) => keyword.trim())
        .filter(Boolean);

      const playbookPayload = {
        name: payload.name.trim(),
        description: payload.description.trim() || undefined,
        icon: payload.icon.trim() || undefined,
        is_public: payload.isPublic,
        create_channel_on_run: payload.createChannelOnRun,
        channel_name_template: payload.createChannelOnRun
          ? payload.channelNameTemplate.trim() || 'run-{{date}}'
          : undefined,
        keyword_triggers: keywordTriggers,
      };

      let savedPlaybookId = playbookId();
      if (savedPlaybookId) {
        await playbooksApi.update(savedPlaybookId, playbookPayload);
      } else {
        const created = await playbooksApi.create(currentTeamId, playbookPayload);
        savedPlaybookId = created.id;
      }

      if (!savedPlaybookId) {
        throw new Error('Playbook id is missing after save.');
      }

      await syncChecklists(savedPlaybookId);
      toast.success('Playbook saved', 'Your changes were stored successfully.');
      navigate('/playbooks');
    } catch (err) {
      const message = getErrorMessage(err) || 'Failed to save playbook.';
      setError(message);
      toast.error('Unable to save playbook', message);
    } finally {
      setIsSaving(false);
    }
  };

  onMount(() => {
    void load();
  });

  return (
    <div class="h-full flex flex-col bg-bg-app">
      <header class="border-b border-border-1 bg-bg-surface-1 px-6 py-4 flex items-center justify-between">
        <div class="flex items-center gap-3">
          <Button variant="ghost" size="sm" onClick={() => navigate('/playbooks')}>
            Back
          </Button>
          <div>
            <h1 class="text-xl font-semibold text-text-1">
              {isEditing() ? 'Edit Playbook' : 'Create Playbook'}
            </h1>
            <p class="text-sm text-text-3">
              Configure automation settings, checklists, and run tasks.
            </p>
          </div>
        </div>
        <Button variant="primary" loading={isSaving()} onClick={() => void save()}>
          Save Playbook
        </Button>
      </header>

      <main class="flex-1 overflow-y-auto p-6">
        <Show
          when={!isLoading()}
          fallback={
            <div class="h-full flex items-center justify-center">
              <div class="w-10 h-10 rounded-full border-4 border-brand/20 border-t-brand animate-spin" />
            </div>
          }
        >
          <div class="mx-auto w-full max-w-4xl space-y-6">
            <Show when={error()}>
              <div class="rounded-xl border border-danger/30 bg-danger/10 px-4 py-3 text-sm text-danger">
                {error()}
              </div>
            </Show>

            <Show when={warning()}>
              <div class="rounded-xl border border-warning/30 bg-warning/10 px-4 py-3 text-sm text-warning">
                {warning()}
              </div>
            </Show>

            <section class="rounded-xl border border-border-1 bg-bg-surface-1 p-5 space-y-4">
              <h2 class="text-lg font-semibold text-text-1">General Information</h2>
              <div class="grid grid-cols-1 md:grid-cols-[84px_1fr] gap-4">
                <label class="space-y-1">
                  <span class="text-sm font-medium text-text-2">Icon</span>
                  <input
                    type="text"
                    value={form().icon}
                    onInput={(event) => updateForm('icon', event.currentTarget.value)}
                    class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-center text-2xl text-text-1"
                  />
                </label>
                <label class="space-y-1">
                  <span class="text-sm font-medium text-text-2">Name</span>
                  <input
                    type="text"
                    value={form().name}
                    onInput={(event) => updateForm('name', event.currentTarget.value)}
                    placeholder="Incident Response"
                    class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                  />
                </label>
              </div>
              <label class="space-y-1 block">
                <span class="text-sm font-medium text-text-2">Description</span>
                <textarea
                  value={form().description}
                  onInput={(event) => updateForm('description', event.currentTarget.value)}
                  rows={3}
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1 resize-y"
                />
              </label>
            </section>

            <section class="rounded-xl border border-border-1 bg-bg-surface-1 p-5 space-y-4">
              <h2 class="text-lg font-semibold text-text-1">Automation</h2>

              <label class="inline-flex items-center gap-2 text-sm text-text-2">
                <input
                  type="checkbox"
                  checked={form().isPublic}
                  onChange={(event) => updateForm('isPublic', event.currentTarget.checked)}
                  class="h-4 w-4 rounded border-border-2"
                />
                Public in team
              </label>

              <label class="inline-flex items-center gap-2 text-sm text-text-2">
                <input
                  type="checkbox"
                  checked={form().createChannelOnRun}
                  onChange={(event) =>
                    updateForm('createChannelOnRun', event.currentTarget.checked)
                  }
                  class="h-4 w-4 rounded border-border-2"
                />
                Create channel when run starts
              </label>

              <Show when={form().createChannelOnRun}>
                <label class="space-y-1 block">
                  <span class="text-sm font-medium text-text-2">Channel Name Template</span>
                  <input
                    type="text"
                    value={form().channelNameTemplate}
                    onInput={(event) =>
                      updateForm('channelNameTemplate', event.currentTarget.value)
                    }
                    placeholder="run-{{date}}"
                    class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                  />
                  <p class="text-xs text-text-3">Supported variables: {'{{date}}, {{playbook_name}}'}</p>
                </label>
              </Show>

              <label class="space-y-1 block">
                <span class="text-sm font-medium text-text-2">Keyword Triggers</span>
                <input
                  type="text"
                  value={form().keywordTriggers}
                  onInput={(event) => updateForm('keywordTriggers', event.currentTarget.value)}
                  placeholder="incident, outage, escalation"
                  class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                />
                <p class="text-xs text-text-3">
                  Comma-separated keywords that can trigger this playbook.
                </p>
              </label>
            </section>

            <section class="rounded-xl border border-border-1 bg-bg-surface-1 p-5 space-y-4">
              <div class="flex items-center justify-between gap-3">
                <h2 class="text-lg font-semibold text-text-1">Checklists & Tasks</h2>
                <Button variant="secondary" size="sm" onClick={addChecklist}>
                  Add Checklist
                </Button>
              </div>

              <div class="space-y-4">
                <For each={checklists()}>
                  {(checklist, checklistIndex) => (
                    <article class="rounded-xl border border-border-1 bg-bg-app p-4 space-y-3">
                      <div class="flex items-center gap-2">
                        <input
                          type="text"
                          value={checklist.name}
                          onInput={(event) =>
                            updateChecklistName(checklistIndex(), event.currentTarget.value)
                          }
                          disabled={!checklist.isTemp}
                          class="flex-1 rounded-lg border border-border-1 bg-bg-surface-1 px-3 py-2 text-sm text-text-1 disabled:opacity-70 disabled:cursor-not-allowed"
                          placeholder="Checklist name"
                        />
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => removeChecklist(checklistIndex())}
                        >
                          Remove
                        </Button>
                      </div>

                      <Show when={!checklist.isTemp}>
                        <p class="text-xs text-text-3">
                          Checklist rename is not available in the current API; add a new checklist
                          if you need a different name.
                        </p>
                      </Show>

                      <div class="space-y-2">
                        <For each={checklist.tasks}>
                          {(task, taskIndex) => (
                            <div class="rounded-lg border border-border-1 bg-bg-surface-1 p-3 space-y-2">
                              <div class="flex gap-2">
                                <input
                                  type="text"
                                  value={task.title}
                                  onInput={(event) =>
                                    updateTask(
                                      checklistIndex(),
                                      taskIndex(),
                                      'title',
                                      event.currentTarget.value
                                    )
                                  }
                                  placeholder="Task title"
                                  class="flex-1 rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                                />
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() => removeTask(checklistIndex(), taskIndex())}
                                >
                                  Delete
                                </Button>
                              </div>
                              <input
                                type="text"
                                value={task.description}
                                onInput={(event) =>
                                  updateTask(
                                    checklistIndex(),
                                    taskIndex(),
                                    'description',
                                    event.currentTarget.value
                                  )
                                }
                                placeholder="Description (optional)"
                                class="w-full rounded-lg border border-border-1 bg-bg-app px-3 py-2 text-sm text-text-1"
                              />
                            </div>
                          )}
                        </For>
                      </div>

                      <Button
                        variant="secondary"
                        size="sm"
                        onClick={() => addTask(checklistIndex())}
                      >
                        Add Task
                      </Button>
                    </article>
                  )}
                </For>
              </div>
            </section>
          </div>
        </Show>
      </main>
    </div>
  );
}
