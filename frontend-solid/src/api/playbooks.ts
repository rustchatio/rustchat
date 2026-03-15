// ============================================
// Playbooks API Methods
// ============================================

import { client } from './client';

export interface Playbook {
  id: string;
  team_id: string;
  created_by: string;
  name: string;
  description: string | null;
  icon: string | null;
  is_public: boolean;
  member_ids: string[] | null;
  create_channel_on_run: boolean;
  channel_name_template: string | null;
  default_owner_id: string | null;
  webhook_enabled: boolean;
  webhook_secret: string | null;
  keyword_triggers: string[] | null;
  is_archived: boolean;
  version: number;
  created_at: string;
  updated_at: string;
}

export interface PlaybookTask {
  id: string;
  checklist_id: string;
  title: string;
  description: string | null;
  default_assignee_id: string | null;
  assignee_role: string | null;
  due_after_minutes: number | null;
  slash_command: string | null;
  webhook_url: string | null;
  condition_attribute: string | null;
  condition_value: string | null;
  sort_order: number;
  created_at: string;
}

export interface ChecklistWithTasks {
  id: string;
  playbook_id: string;
  name: string;
  sort_order: number;
  created_at: string;
  tasks: PlaybookTask[];
}

export interface PlaybookFull extends Playbook {
  checklists: ChecklistWithTasks[];
}

export interface CreatePlaybookRequest {
  name: string;
  description?: string;
  icon?: string;
  is_public?: boolean;
  create_channel_on_run?: boolean;
  channel_name_template?: string;
}

export interface UpdatePlaybookRequest {
  name?: string;
  description?: string;
  icon?: string;
  is_public?: boolean;
  create_channel_on_run?: boolean;
  channel_name_template?: string;
  keyword_triggers?: string[];
}

export interface PlaybookChecklist {
  id: string;
  playbook_id: string;
  name: string;
  sort_order: number;
  created_at: string;
}

export interface CreateChecklistRequest {
  name: string;
  sort_order?: number;
}

export interface CreateTaskRequest {
  title: string;
  description?: string | null;
  default_assignee_id?: string | null;
  due_after_minutes?: number | null;
  slash_command?: string | null;
  sort_order?: number;
}

export interface PlaybookRun {
  id: string;
  playbook_id: string;
  team_id: string;
  channel_id: string | null;
  name: string;
  owner_id: string;
  status: 'in_progress' | 'finished' | 'archived' | string;
  attributes: Record<string, unknown> | null;
  summary: string | null;
  started_at: string;
  finished_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface RunTask {
  id: string;
  run_id: string;
  task_id: string;
  status: 'pending' | 'in_progress' | 'done' | 'skipped' | string;
  assignee_id: string | null;
  completed_at: string | null;
  completed_by: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

export interface RunProgress {
  total: number;
  completed: number;
  in_progress: number;
  pending: number;
}

export interface RunWithTasks {
  run: PlaybookRun;
  tasks: RunTask[];
  progress: RunProgress;
}

type RunWithTasksRaw = RunWithTasks | (PlaybookRun & { tasks: RunTask[]; progress: RunProgress });

export interface StartRunRequest {
  playbook_id: string;
  name: string;
  owner_id?: string;
  channel_id?: string;
  attributes?: Record<string, unknown>;
}

export interface UpdateRunTaskRequest {
  status?: string;
  assignee_id?: string;
  notes?: string;
}

export interface RunStatusUpdate {
  id: string;
  run_id: string;
  author_id: string;
  message: string;
  is_broadcast: boolean;
  created_at: string;
}

function normalizeRunWithTasks(payload: RunWithTasksRaw): RunWithTasks {
  if ('run' in payload) {
    return payload;
  }

  const { tasks, progress, ...run } = payload;
  return {
    run: run as PlaybookRun,
    tasks,
    progress,
  };
}

export async function listPlaybooks(teamId: string): Promise<Playbook[]> {
  const response = await client.get<Playbook[]>('/playbooks', {
    params: { team_id: teamId },
  });
  return response.data;
}

export async function createPlaybook(
  teamId: string,
  data: CreatePlaybookRequest
): Promise<Playbook> {
  const response = await client.post<Playbook>('/playbooks', data, {
    params: { team_id: teamId },
  });
  return response.data;
}

export async function getPlaybook(id: string): Promise<PlaybookFull> {
  const response = await client.get<PlaybookFull>(`/playbooks/${id}`);
  return response.data;
}

export async function updatePlaybook(
  id: string,
  data: UpdatePlaybookRequest
): Promise<Playbook> {
  const response = await client.put<Playbook>(`/playbooks/${id}`, data);
  return response.data;
}

export async function deletePlaybook(id: string): Promise<void> {
  await client.delete(`/playbooks/${id}`);
}

export async function createChecklist(
  playbookId: string,
  data: CreateChecklistRequest
): Promise<PlaybookChecklist> {
  const response = await client.post<PlaybookChecklist>(
    `/playbooks/${playbookId}/checklists`,
    data
  );
  return response.data;
}

export async function deleteChecklist(playbookId: string, id: string): Promise<void> {
  await client.delete(`/playbooks/${playbookId}/checklists/${id}`);
}

export async function createTask(
  checklistId: string,
  data: CreateTaskRequest
): Promise<PlaybookTask> {
  const response = await client.post<PlaybookTask>(
    `/checklists/${checklistId}/tasks`,
    data
  );
  return response.data;
}

export async function updateTask(id: string, data: CreateTaskRequest): Promise<PlaybookTask> {
  const response = await client.put<PlaybookTask>(`/tasks/${id}`, data);
  return response.data;
}

export async function deleteTask(id: string): Promise<void> {
  await client.delete(`/tasks/${id}`);
}

export async function listRuns(teamId: string): Promise<PlaybookRun[]> {
  const response = await client.get<PlaybookRun[]>('/runs', {
    params: { team_id: teamId },
  });
  return response.data;
}

export async function startRun(
  teamId: string,
  data: StartRunRequest
): Promise<RunWithTasks> {
  const response = await client.post<RunWithTasksRaw>('/runs', data, {
    params: { team_id: teamId },
  });
  return normalizeRunWithTasks(response.data);
}

export async function getRun(id: string): Promise<RunWithTasks> {
  const response = await client.get<RunWithTasksRaw>(`/runs/${id}`);
  return normalizeRunWithTasks(response.data);
}

export async function finishRun(id: string): Promise<PlaybookRun> {
  const response = await client.post<PlaybookRun>(`/runs/${id}/finish`);
  return response.data;
}

export async function updateRunTask(
  runId: string,
  taskId: string,
  data: UpdateRunTaskRequest
): Promise<RunTask> {
  const response = await client.put<RunTask>(`/runs/${runId}/tasks/${taskId}`, data);
  return response.data;
}

export async function listStatusUpdates(runId: string): Promise<RunStatusUpdate[]> {
  const response = await client.get<RunStatusUpdate[]>(`/runs/${runId}/updates`);
  return response.data;
}

export async function createStatusUpdate(
  runId: string,
  data: { message: string; is_broadcast?: boolean }
): Promise<RunStatusUpdate> {
  const response = await client.post<RunStatusUpdate>(`/runs/${runId}/updates`, data);
  return response.data;
}

export const playbooksApi = {
  list: listPlaybooks,
  create: createPlaybook,
  get: getPlaybook,
  update: updatePlaybook,
  delete: deletePlaybook,
  createChecklist,
  deleteChecklist,
  createTask,
  updateTask,
  deleteTask,
  listRuns,
  startRun,
  getRun,
  finishRun,
  updateRunTask,
  listStatusUpdates,
  createStatusUpdate,
};

export default playbooksApi;
