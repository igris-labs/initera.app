import { defineStore } from 'pinia';
import {
  createProjectUpdate,
  loadProjectUpdates,
  usesNativeStorage,
} from '../lib/workspace-storage';
import type { ProjectUpdate } from '../lib/workspace-storage';
import { useWorkspaceStore } from './workspace';

export const useProjectUpdatesStore = defineStore('project-updates', {
  state: (): {
    byProject: Record<string, ProjectUpdate[]>;
    loaded: string[];
    hasMore: Record<string, boolean>;
    cursorByProject: Record<string, string | null>;
    loading: Record<string, boolean>;
    errors: Record<string, string>;
  } => ({
    byProject: {},
    loaded: [],
    hasMore: {},
    cursorByProject: {},
    loading: {},
    errors: {},
  }),
  actions: {
    async load(projectId: string, force = false) {
      if (
        !usesNativeStorage ||
        this.loading[projectId] ||
        (!force && this.loaded.includes(projectId))
      )
        return;
      this.loading[projectId] = true;
      try {
        const page = await loadProjectUpdates(projectId);
        this.byProject[projectId] = page.items;
        this.cursorByProject[projectId] = page.nextCursor;
        this.hasMore[projectId] = Boolean(page.nextCursor);
        if (!this.loaded.includes(projectId)) this.loaded.push(projectId);
        delete this.errors[projectId];
      } catch (cause) {
        this.loaded = this.loaded.filter((id) => id !== projectId);
        this.errors[projectId] = String(cause);
        throw cause;
      } finally {
        this.loading[projectId] = false;
      }
    },
    async loadMore(projectId: string) {
      if (!usesNativeStorage || this.loading[projectId] || !this.hasMore[projectId]) return;
      this.loading[projectId] = true;
      try {
        const previous = this.byProject[projectId] ?? [];
        const page = await loadProjectUpdates(
          projectId,
          this.cursorByProject[projectId] ?? undefined,
        );
        this.byProject[projectId] = [...previous, ...page.items];
        this.cursorByProject[projectId] = page.nextCursor;
        this.hasMore[projectId] = Boolean(page.nextCursor);
        delete this.errors[projectId];
      } catch (cause) {
        this.errors[projectId] = String(cause);
        throw cause;
      } finally {
        this.loading[projectId] = false;
      }
    },
    recordChange(projectId: string, body: string) {
      if (!body) return;
      this.byProject[projectId] = [
        {
          id: crypto.randomUUID(),
          projectId,
          actorName: useWorkspaceStore().userName || 'You',
          body,
          kind: 'change',
          createdAt: new Date().toISOString(),
        },
        ...(this.byProject[projectId] ?? []),
      ];
    },
    async create(projectId: string, body: string) {
      const trimmed = body.trim();
      if (!trimmed || trimmed.length > 100_000)
        throw new Error('Write an update of up to 100,000 characters.');
      const update = usesNativeStorage
        ? await createProjectUpdate(projectId, trimmed)
        : {
            id: crypto.randomUUID(),
            projectId,
            actorName: useWorkspaceStore().userName || 'You',
            body: trimmed,
            kind: 'update' as const,
            createdAt: new Date().toISOString(),
          };
      this.byProject[projectId] = [update, ...(this.byProject[projectId] ?? [])];
      return update;
    },
  },
  persist: usesNativeStorage ? false : { pick: ['byProject'] },
});
