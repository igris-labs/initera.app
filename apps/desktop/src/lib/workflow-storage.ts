import { invoke } from './native-invoke';
import type { WorkflowStatus } from '../types/workflow';

export interface WorkflowSnapshot {
  statuses: WorkflowStatus[];
  version: number;
}

export function listWorkflow() {
  return invoke<WorkflowSnapshot>('storage_list_workflow');
}

export function saveWorkflow(
  statuses: WorkflowStatus[],
  version: number,
  renameFrom?: string,
  renameTo?: string,
  mutationId = crypto.randomUUID(),
) {
  return invoke<WorkflowSnapshot>('storage_save_workflow', {
    statuses,
    renameFrom: renameFrom ?? null,
    renameTo: renameTo ?? null,
    mutation: { mutationId, expectedVersion: version },
  });
}
