import { invoke } from './native-invoke';
import type { SavedView } from '../types/view';

export function loadSavedViews() {
  return invoke<SavedView[]>('storage_list_views');
}

export function createSavedView(view: SavedView, mutationId = crypto.randomUUID()) {
  return invoke<SavedView>('storage_create_view', { view, mutation: { mutationId } });
}

export function updateSavedView(view: SavedView, mutationId = crypto.randomUUID()) {
  return invoke<SavedView>('storage_update_view', {
    view,
    mutation: { mutationId, expectedVersion: view.version },
  });
}

export function deleteSavedView(
  id: string,
  expectedVersion: number,
  mutationId = crypto.randomUUID(),
) {
  return invoke<void>('storage_delete_view', {
    id,
    mutation: { mutationId, expectedVersion },
  });
}
