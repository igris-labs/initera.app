import { invoke } from './native-invoke';
import { usesNativeStorage } from './workspace-storage';

export interface DraftRecord<T> {
  key: string;
  payload: T;
  version: number;
  updatedAt: string;
}

export async function loadDraft<T>(key: string): Promise<DraftRecord<T> | null> {
  if (!usesNativeStorage) return null;
  return invoke<DraftRecord<T> | null>('storage_load_draft', { key });
}

export async function saveDraft<T>(
  key: string,
  payload: T,
  expectedVersion: number | null,
): Promise<DraftRecord<T>> {
  return invoke<DraftRecord<T>>('storage_save_draft', {
    key,
    payload,
    expectedVersion,
    mutation: { mutationId: crypto.randomUUID() },
  });
}

export async function discardDraft(key: string, expectedVersion: number | null): Promise<void> {
  await invoke('storage_discard_draft', {
    key,
    expectedVersion,
    mutation: { mutationId: crypto.randomUUID() },
  });
}
