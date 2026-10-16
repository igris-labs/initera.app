import { defineStore } from 'pinia';
import {
  createProject,
  loadInactiveProjects,
  loadProject,
  loadProjectByName,
  loadProjectPage,
  setProjectLifecycle,
  updateProject,
  usesNativeStorage,
} from '../lib/workspace-storage';
import type {
  ProjectIndexQuery,
  ProjectInput,
  ProjectLifecycleEntry,
  ProjectRecord,
} from '../lib/workspace-storage';
import { notifySaved } from '../lib/notifications';
import { projectChanges } from '../lib/project-activity';
import { useProjectUpdatesStore } from './project-updates';
import { useIssuesStore } from './issues';

export const useProjectsStore = defineStore('projects', {
  state: (): {
    items: ProjectRecord[];
    inactive: ProjectLifecycleEntry[];
    inactiveRecords: (ProjectRecord & { lifecycleState: 'archived' | 'trash' })[];
    ready: boolean;
    loadError: string;
    nextCursor: string | null;
    totalCount: number;
    indexQuery: ProjectIndexQuery;
  } => ({
    items: [],
    ready: !usesNativeStorage,
    inactive: [],
    inactiveRecords: [],
    loadError: '',
    nextCursor: null,
    totalCount: 0,
    indexQuery: { sort: 'updated', status: null, priority: null, search: '' },
  }),
  getters: {
    names: (state) => [...state.items.map((item) => item.name), 'Unassigned'],
  },
  actions: {
    async refresh() {
      if (!usesNativeStorage) return;
      this.items = [];
      this.nextCursor = null;
      await this.loadMore();
    },
    async setIndexQuery(query: ProjectIndexQuery) {
      this.indexQuery = query;
      await this.refresh();
    },
    async refreshOne(id: string) {
      if (!usesNativeStorage) return;
      const record = await loadProject(id);
      const index = this.items.findIndex((item) => item.id === id);
      if (index >= 0) this.items.splice(index, 1, record);
      else this.items.push(record);
    },
    async ensureByName(name: string) {
      if (!usesNativeStorage || this.items.some((item) => item.name === name)) return;
      const record = await loadProjectByName(name);
      if (!this.items.some((item) => item.id === record.id)) this.items.push(record);
    },
    async loadMore() {
      if (!usesNativeStorage) return;
      let page;
      try {
        page = await loadProjectPage(this.nextCursor ?? undefined, 200, this.indexQuery);
      } catch (error) {
        if (!this.nextCursor || !String(error).includes('CONFLICT:')) throw error;
        this.items = [];
        this.nextCursor = null;
        page = await loadProjectPage(undefined, 200, this.indexQuery);
      }

      const existing = new Set(this.items.map((item) => item.id));
      this.items.push(...page.items.filter((item) => !existing.has(item.id)));
      this.nextCursor = page.nextCursor;
      this.totalCount = page.totalCount;
    },
    async bootstrap() {
      if (!usesNativeStorage || this.ready) return;
      this.loadError = '';
      try {
        await this.loadMore();
        this.ready = true;
      } catch (error) {
        this.loadError = String(error);
      }
    },
    async loadInactive() {
      this.inactive = usesNativeStorage
        ? await loadInactiveProjects()
        : this.inactiveRecords.map((item) => ({
            id: item.id,
            name: item.name,
            state: item.lifecycleState,
            version: item.version ?? 1,
          }));
    },
    async setLifecycle(id: string, action: 'archive' | 'trash' | 'restore') {
      const current =
        this.items.find((item) => item.id === id) ?? this.inactive.find((item) => item.id === id);
      if (!current) throw new Error('Project no longer exists.');
      if (usesNativeStorage) await setProjectLifecycle(id, action, current.version ?? 0);
      if (action === 'restore') {
        const restored = usesNativeStorage
          ? await loadProject(id)
          : this.inactiveRecords.find((item) => item.id === id);
        if (restored) this.items.push(restored);
        if (usesNativeStorage) this.totalCount += 1;
        this.inactiveRecords = this.inactiveRecords.filter((item) => item.id !== id);
      } else {
        if (!usesNativeStorage && 'workspaceId' in current)
          this.inactiveRecords.push({
            ...current,
            lifecycleState: action === 'archive' ? 'archived' : 'trash',
          });
        this.items = this.items.filter((item) => item.id !== id);
        if (usesNativeStorage) this.totalCount = Math.max(0, this.totalCount - 1);
      }

      await this.loadInactive();
      notifySaved(
        `Project ${current.name} ${action === 'restore' ? 'restored' : action === 'archive' ? 'archived' : 'moved to trash'}`,
      );
    },
    async create(input: ProjectInput) {
      const trimmed = input.name.trim();
      const normalized = input.identifierPrefix.trim().toUpperCase();
      if (!trimmed || trimmed.length > 80 || !/^[A-Z]{2,8}$/.test(normalized))
        throw new Error('Enter a project name and a 2–8 letter identifier.');
      if (trimmed.toLowerCase() === 'unassigned' || normalized === 'LOCAL')
        throw new Error('Unassigned and LOCAL are reserved for issues without a project.');
      if (
        this.items.some(
          (item) =>
            item.name.toLowerCase() === trimmed.toLowerCase() ||
            item.identifierPrefix === normalized,
        )
      )
        throw new Error('Project name and identifier must be unique.');
      if (input.startDate && input.targetDate && input.startDate > input.targetDate)
        throw new Error('Target date must be on or after the start date.');
      if (input.milestones.some((milestone) => !milestone.title.trim()))
        throw new Error('Each milestone needs a title.');
      const project = usesNativeStorage
        ? await createProject({ ...input, name: trimmed, identifierPrefix: normalized })
        : {
            ...input,
            id: crypto.randomUUID(),
            workspaceId: 'preview',
            name: trimmed,
            identifierPrefix: normalized,
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
            milestones: input.milestones.map((milestone) => ({
              ...milestone,
              id: crypto.randomUUID(),
            })),
          };
      this.items.push(project);
      if (usesNativeStorage) this.totalCount += 1;
      if (usesNativeStorage) await useIssuesStore().refreshCounts();
      return project;
    },
    async update(id: string, input: ProjectInput) {
      const current = this.items.find((item) => item.id === id);
      if (!current) throw new Error('Project no longer exists.');
      const name = input.name.trim();
      const identifierPrefix = input.identifierPrefix.trim().toUpperCase();
      if (!name || name.length > 80 || !/^[A-Z]{2,8}$/.test(identifierPrefix))
        throw new Error('Enter a project name and a 2–8 letter identifier.');
      if (name.toLowerCase() === 'unassigned' || identifierPrefix === 'LOCAL')
        throw new Error('Unassigned and LOCAL are reserved.');
      if (
        this.items.some(
          (item) =>
            item.id !== id &&
            (item.name.toLowerCase() === name.toLowerCase() ||
              item.identifierPrefix === identifierPrefix),
        )
      )
        throw new Error('Project name and identifier must be unique.');
      if (input.startDate && input.targetDate && input.startDate > input.targetDate)
        throw new Error('Target date must be on or after the start date.');
      if (input.milestones.some((milestone) => !milestone.title.trim()))
        throw new Error('Each milestone needs a title.');
      const normalized = { ...input, name, identifierPrefix };
      const updated = usesNativeStorage
        ? await updateProject(id, normalized, current.version)
        : {
            ...current,
            ...normalized,
            milestones: normalized.milestones.map((milestone) => ({
              ...milestone,
              id: milestone.id ?? crypto.randomUUID(),
            })),
          };
      this.items.splice(this.items.indexOf(current), 1, updated);
      const retained = new Set(updated.milestones.map((milestone) => milestone.id));
      const issueStore = useIssuesStore();
      for (const issue of [...issueStore.items, ...issueStore.archivedItems]) {
        if (issue.project !== current.name) continue;
        issue.project = updated.name;
        if (issue.milestoneId && !retained.has(issue.milestoneId)) issue.milestoneId = null;
      }

      if (usesNativeStorage) await issueStore.refreshCounts();

      const activity = useProjectUpdatesStore();
      if (usesNativeStorage) await activity.load(id, true).catch(() => {});
      else activity.recordChange(id, projectChanges(current, normalized));
      notifySaved(`${updated.name}: ${projectChanges(current, normalized) || 'project saved'}`);
      return updated;
    },
  },
  persist: usesNativeStorage ? false : { pick: ['items', 'inactiveRecords'] },
});
