import { invoke } from './native-invoke';

export interface Cycle {
  id: string;
  version?: number;
  title: string;
  description?: string;
  startDate: string;
  endDate: string;
  status: 'planned' | 'active' | 'completed';
  scheduled?: boolean;
  seriesId?: string | null;
  successorId?: string | null;
  scheduleIndex?: number;
  issueIds: string[];
  projectIds: string[];
  completedCount?: number | null;
  totalCount?: number | null;
  rolloverPolicy?: 'automatic' | 'leave' | 'ask';
  transitionResult?: 'pending' | 'completed' | null;
  rolloverAttemptOrder?: number;
  timeZone?: string;
  startLocalTime?: string;
  endLocalTime?: string;
  startAt?: string;
  endAt?: string;
  preciseEnd?: boolean;
}

export interface CycleSettings {
  id: string;
  version?: number;
  name: string;
  durationDays: number;
  intervalDays: number;
  enabled: boolean;
  startDate: string;
  rolloverPolicy: 'automatic' | 'leave' | 'ask';
  projectIds: string[];
  timeZone?: string;
  startLocalTime?: string;
}

export interface CycleReconcileProgress {
  seriesId: string;
  total: number;
  processed: number;
}

export interface CycleTiming {
  timeZone: string;
  startLocalTime: string;
  endLocalTime: string;
  rolloverPolicy: 'automatic' | 'leave' | 'ask';
}

export function listCycleSeries() {
  return invoke<CycleSettings[]>('storage_list_cycle_series');
}

export function setCycleSeriesEnabled(seriesId: string, enabled: boolean, version: number) {
  return invoke<void>('storage_set_cycle_series_enabled', {
    seriesId,
    enabled,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: version },
  });
}

export function listCycleReconcileProgress() {
  return invoke<CycleReconcileProgress[]>('storage_list_cycle_reconcile_progress');
}

export function configureCycles(settings: CycleSettings) {
  return invoke<void>('storage_configure_cycles', {
    settings,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: null },
  });
}

export function updateCycleSchedule(settings: CycleSettings) {
  return invoke<void>('storage_update_cycle_schedule', {
    settings,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: settings.version ?? 0 },
  });
}

export function setCycleSuccessor(cycleId: string, version: number, successorId?: string | null) {
  return invoke<void>('storage_set_cycle_successor', {
    cycleId,
    successorId,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: version },
  });
}

export function listCycles() {
  return invoke<Cycle[]>('storage_list_cycles');
}

export function createCycle(
  title: string,
  startDate: string,
  endDate: string,
  projectIds: string[],
  timing?: CycleTiming,
) {
  return invoke<Cycle>('storage_create_cycle', {
    title,
    startDate,
    endDate,
    projectIds,
    ...timing,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: null },
  });
}

export function updateCycle(cycle: Cycle) {
  return invoke<Cycle>('storage_update_cycle', {
    cycleId: cycle.id,
    title: cycle.title,
    description: cycle.description ?? '',
    startDate: cycle.startDate,
    endDate: cycle.endDate,
    projectIds: cycle.projectIds,
    timeZone: cycle.timeZone,
    startLocalTime: cycle.startLocalTime,
    endLocalTime: cycle.endLocalTime,
    rolloverPolicy: cycle.rolloverPolicy,
    preciseEnd: cycle.preciseEnd,
    successorId: cycle.successorId,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: cycle.version ?? 0 },
  });
}

export function setCycleIssue(
  cycleId: string,
  identifier: string,
  assigned: boolean,
  version: number,
) {
  return invoke<void>('storage_set_cycle_issue', {
    cycleId,
    identifier,
    assigned,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: version },
  });
}

export function completeCycle(
  cycleId: string,
  nextCycleId?: string,
  selectedIdentifiers?: string[],
  version?: number,
  nextCycleVersion?: number,
) {
  return invoke<void>('storage_complete_cycle', {
    cycleId,
    nextCycleId,
    selectedIdentifiers,
    nextCycleVersion,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: version ?? 0 },
  });
}
