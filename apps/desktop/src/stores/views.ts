import { defineStore } from 'pinia';
import { notifications } from '../lib/notifications';
import { usesNativeStorage } from '../lib/workspace-storage';
import {
  createSavedView,
  deleteSavedView,
  loadSavedViews,
  updateSavedView,
} from '../lib/view-storage';
import type { SavedView } from '../types/view';

let pendingSave: Promise<unknown> = Promise.resolve();
const revisions = new Map<string, number>();

export const useViewsStore = defineStore('views', {
  state: (): { items: SavedView[]; ready: boolean; loadError: string } => ({
    items: usesNativeStorage
      ? []
      : [
          {
            id: 'board-view',
            name: 'Board view',
            description: 'Work grouped by status',
            layout: 'board',
            groupBy: 'status',
            scope: 'active',
            status: 'All statuses',
            favorite: false,
          },
        ],
    ready: !usesNativeStorage,
    loadError: '',
  }),
  actions: {
    async bootstrap() {
      if (!usesNativeStorage || this.ready) return;

      this.loadError = '';
      try {
        this.items = await loadSavedViews();
        this.ready = true;
      } catch (error) {
        this.loadError = String(error);
      }
    },
    async flushSaves() {
      await pendingSave;
    },
    async create(name: string): Promise<SavedView> {
      const trimmed = name.trim();
      if (!trimmed || trimmed.length > 80)
        throw new Error('Enter a view name of up to 80 characters.');

      const view: SavedView = {
        id: crypto.randomUUID(),
        name: trimmed,
        description: '',
        layout: 'list',
        groupBy: 'status',
        scope: 'active',
        status: 'All statuses',
        favorite: false,
      };
      if (usesNativeStorage) {
        await pendingSave;
        const saved = await createSavedView(view);
        this.items.push(saved);
        return saved;
      } else {
        this.items.push(view);
      }

      return view;
    },
    update(id: string, patch: Partial<SavedView>): Promise<boolean> {
      const view = this.items.find((item) => item.id === id);
      if (!view) return Promise.resolve(false);

      Object.assign(view, patch);
      if (!usesNativeStorage) return Promise.resolve(true);

      const revision = (revisions.get(id) ?? 0) + 1;
      revisions.set(id, revision);

      // Serialize snapshots so rapid filter edits cannot reach SQLite out of order.
      const save = pendingSave.then(async () => {
        const saved = await updateSavedView({ ...view });
        view.version = saved.version;
        return saved;
      });
      const result = save.then(
        () => true,
        (error) => {
          notifications.add({
            title: 'Could not save view',
            description: `${String(error)} Your edits are still here. Reload the view before retrying.`,
            type: 'error',
          });
          return false;
        },
      );
      pendingSave = result.then(() => {});
      return result;
    },
    async remove(id: string) {
      if (usesNativeStorage) {
        await pendingSave;
        const view = this.items.find((item) => item.id === id);
        if (!view?.version) throw new Error('Saved view needs to be reloaded before deletion.');
        await deleteSavedView(id, view.version);
      }

      this.items = this.items.filter((item) => item.id !== id);
      revisions.delete(id);
    },
  },
  persist: usesNativeStorage ? false : { pick: ['items'] },
});
