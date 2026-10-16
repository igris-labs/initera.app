import { invoke } from './native-invoke';

export interface LabelRecord {
  id: string;
  version: number;
  name: string;
  color: string;
  archivedAt: string | null;
  deletedAt: string | null;
  issueCount: number;
  projectCount: number;
  detached?: { issueIds: string[]; projectIds: string[] };
}

export interface LabelRestoreReport {
  restored: number;
  skipped: number;
}

export const listLabels = () => invoke<LabelRecord[]>('storage_list_labels');
export function createLabel(name: string, color: string) {
  return invoke<LabelRecord[]>('storage_create_label', {
    name,
    color,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: null },
  });
}

export function updateLabel(id: string, name: string, color: string, version: number) {
  return invoke<LabelRecord[]>('storage_update_label', {
    id,
    name,
    color,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: version },
  });
}

export function archiveLabel(id: string, version: number) {
  return invoke<LabelRecord[]>('storage_archive_label', {
    id,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: version },
  });
}

export function restoreArchivedLabel(id: string, version: number) {
  return invoke<LabelRecord[]>('storage_restore_archived_label', {
    id,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: version },
  });
}

export function trashLabel(id: string, version: number) {
  return invoke<LabelRecord[]>('storage_trash_label', {
    id,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: version },
  });
}

export function restoreTrashedLabel(id: string, version: number) {
  return invoke<LabelRestoreReport>('storage_restore_trashed_label', {
    id,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: version },
  });
}
