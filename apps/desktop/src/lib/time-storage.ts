import { invoke } from './native-invoke';

export interface TimeEntry {
  id: string;
  version: number;
  issueId: string;
  issueIdentifier: string;
  startedAt: number;
  stoppedAt: number | null;
  durationSeconds: number;
  elapsedMs: number;
  startedAtMs: number;
  stoppedAtMs: number | null;
  source: 'timer' | 'manual';
  startAt: number;
  endAt: number | null;
  running: boolean;
  note: string;
  timeZone: string | null;
}

export interface ManualTimeInput {
  date: string;
  startTime: string;
  timeZone: string;
  ambiguous?: 'earlier' | 'later';
  allowOverlap?: boolean;
}

export interface TimeSummary {
  entries: TimeEntry[];
  running: TimeEntry | null;
  totalSeconds: number;
  totalMs: number;
}

export interface WorkspaceTimeIssue {
  identifier: string;
  title: string;
  project: string;
  completedSeconds: number;
  entryCount: number;
  runningStartedAt: number | null;
  archived: boolean;
}

export interface WorkspaceTimeEntry {
  id: string;
  issueId: string;
  issueIdentifier: string;
  issueTitle: string;
  project: string;
  startedAt: number;
  stoppedAt: number | null;
  durationSeconds: number;
  elapsedMs: number;
  startedAtMs: number;
  stoppedAtMs: number | null;
  source: 'timer' | 'manual';
  startAt: number;
  endAt: number | null;
  running: boolean;
  note: string;
  timeZone: string | null;
}

export interface WorkspaceTimePage {
  items: WorkspaceTimeEntry[];
  nextCursor: string | null;
  totalSeconds: number;
  totalMs: number;
  totalCount: number;
}

export function loadWorkspaceTimePage(filters: {
  issue?: string;
  project?: string;
  start?: number;
  end?: number;
  cursor?: string;
}) {
  return invoke<WorkspaceTimePage>('storage_list_workspace_time_page', { ...filters, limit: 50 });
}

export function loadWorkspaceTime() {
  return invoke<WorkspaceTimeIssue[]>('storage_list_workspace_time');
}

export function loadTime(identifier: string) {
  return invoke<TimeSummary>('storage_list_time', { identifier });
}

export function loadRunningTimer() {
  return invoke<TimeEntry | null>('storage_running_timer');
}

export function loadRunningTimers() {
  return invoke<TimeEntry[]>('storage_running_timers');
}

async function mutateTimer(command: string, identifier: string, switchCurrent = false) {
  const version = await invoke<number>('storage_timer_state_version');
  const summary = await invoke<TimeSummary>(command, {
    identifier,
    switchCurrent,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: version },
  });
  window.dispatchEvent(new Event('timer-changed'));
  return summary;
}

export function startTimer(identifier: string, switchCurrent = false) {
  return mutateTimer('storage_start_timer', identifier, switchCurrent);
}

export function stopTimer(identifier: string) {
  return mutateTimer('storage_stop_timer', identifier);
}

export function addTime(identifier: string, seconds: number, note: string, when: ManualTimeInput) {
  return mutateEntry('storage_add_time', {
    identifier,
    seconds,
    note,
    ...when,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: null },
  });
}

export function updateTime(
  identifier: string,
  entryId: string,
  seconds: number,
  note: string,
  version: number,
  when: ManualTimeInput,
) {
  return mutateEntry('storage_update_time', {
    identifier,
    entryId,
    seconds,
    note,
    ...when,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: version },
  });
}

export function deleteTime(identifier: string, entryId: string, version: number) {
  return mutateEntry('storage_delete_time', {
    identifier,
    entryId,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: version },
  });
}

async function mutateEntry(command: string, input: Record<string, unknown>) {
  const summary = await invoke<TimeSummary>(command, input);
  window.dispatchEvent(new Event('timer-changed'));
  return summary;
}
