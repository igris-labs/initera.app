import { defineStore } from 'pinia';
import {
  archiveIssue,
  archiveIssueWithStrategy,
  createIssue,
  createIssueComment,
  createIssueRelation,
  deleteArchivedIssue,
  deleteIssueRelation,
  getIssueCounts,
  listIssueActivity,
  listIssuePage,
  listIssueRecords,
  listIssueRelations,
  moveIssueProject,
  previewIssueMove,
  removeIssueComment,
  reorderIssues,
  restoreIssue,
  restoreSubIssue,
  updateIssue,
  updateIssueComment,
} from '../lib/issue-storage';
import type {
  ChildStrategy,
  IssueCounts,
  IssueInput,
  IssueMovePreview,
} from '../lib/issue-storage';
import { usesNativeStorage } from '../lib/workspace-storage';
import { listWorkflow, saveWorkflow } from '../lib/workflow-storage';
import type { WorkflowSnapshot } from '../lib/workflow-storage';
import { collectDescendantIds } from '../lib/issue-tree';
import { useProjectsStore } from './projects';
import { notifications, notifySaved } from '../lib/notifications';
import {
  statusColor,
  statusIcon,
  statusIcons,
  statusOrder,
  statusPalette,
} from '../lib/issue-presentation';
import type { StatusColor } from '../lib/issue-presentation';
import type { RichTextDocument } from '../types/rich-text';
import { priorities } from '../types/issue';
import type {
  Issue,
  IssueActivity,
  IssueDraft,
  IssueRelation,
  IssueRelationKind,
  Priority,
} from '../types/issue';
import type {
  StatusAppearance,
  StatusMeta,
  WorkflowCategory,
  WorkflowStatus,
} from '../types/workflow';
import { useWorkspaceStore } from './workspace';

const baseMeta: Record<string, StatusMeta> = {
  Backlog: { category: 'Backlog', description: '', isDefault: true },
  Todo: { category: 'Unstarted', description: '', isDefault: false },
  'In Progress': { category: 'Started', description: '', isDefault: false },
  'On Hold': { category: 'Started', description: '', isDefault: false },
  Done: { category: 'Completed', description: '', isDefault: false },
  Canceled: { category: 'Canceled', description: '', isDefault: false },
};
const baseColors: Record<string, StatusColor> = {
  Backlog: 'gray',
  Todo: 'gray',
  'In Progress': 'amber',
  'On Hold': 'purple',
  Done: 'blue',
  Canceled: 'gray',
};

let saveQueue = Promise.resolve();
let workflowRevision = 0;

function issueSaveMessage(id: string, patch: Partial<Issue>): string {
  const fields = Object.keys(patch).filter((field) => field !== 'updated');
  const labels: Record<string, string> = {
    title: 'title',
    titleMarkdown: 'title formatting',
    description: 'description',
    descriptionDocument: 'description',
    labels: 'labels',
    due: 'due date',
    startDate: 'start date',
    estimate: 'estimate',
    milestoneId: 'milestone',
    parentId: 'parent issue',
  };
  if (fields.length > 1) {
    const names = [...new Set(fields.map((field) => labels[field] ?? field))];
    return `${id} ${names.slice(0, 3).join(', ')}${names.length > 3 ? ', and more' : ''} updated`;
  }

  const field = fields[0];
  if (!field) return `Issue ${id} saved`;
  if (field === 'status' || field === 'priority') {
    return `${id} ${field} changed to ${String(patch[field])}`;
  }

  return `${id} ${labels[field] ?? field} updated`;
}

function inputFor(issue: Issue): IssueInput {
  return {
    title: issue.title,
    titleMarkdown: issue.titleMarkdown,
    description: issue.description,
    descriptionDocument: issue.descriptionDocument,
    status: issue.status,
    priority: issue.priority,
    project: issue.project,
    milestoneId: issue.milestoneId ?? null,
    labels: issue.labels,
    due: issue.due,
    startDate: issue.startDate,
    triage: !!issue.triage,
    parentId: issue.parentId ?? null,
    estimate: issue.estimate ?? null,
  };
}

function displayUpdated(value: string): string {
  const time = Date.parse(value);
  if (Number.isNaN(time)) return value;
  const days = Math.floor((Date.now() - time) / 86_400_000);
  if (days <= 0) return 'Today';
  if (days === 1) return 'Yesterday';
  return new Date(time).toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
}

export const useIssuesStore = defineStore('issues', {
  state: (): {
    items: Issue[];
    archivedItems: Issue[];
    archivedHasMore: boolean;
    archivedCursor: string | null;
    activityByIssue: Record<string, IssueActivity[]>;
    activityHasMore: Record<string, boolean>;
    activityCursorByIssue: Record<string, string | null>;
    relations: IssueRelation[];
    relationsByIssue: Record<string, IssueRelation[]>;
    identifierAliases: Record<string, string>;
    statusGroups: string[];
    archivedStatusGroups: string[];
    statusAppearance: Record<string, StatusAppearance>;
    statusMeta: Record<string, StatusMeta>;
    workflowVersion: number;
    priorityGroupOrder: Priority[];
    ready: boolean;
    loadError: string;
    saveError: string;
    counts: IssueCounts | null;
  } => ({
    items: [],
    archivedItems: [],
    archivedHasMore: false,
    archivedCursor: null,
    activityByIssue: {},
    activityHasMore: {},
    activityCursorByIssue: {},
    relations: [],
    relationsByIssue: {},
    identifierAliases: {},
    statusGroups: [...statusOrder],
    archivedStatusGroups: [],
    statusAppearance: {},
    statusMeta: structuredClone(baseMeta),
    workflowVersion: 0,
    priorityGroupOrder: [...priorities],
    ready: !usesNativeStorage,
    loadError: '',
    saveError: '',
    counts: null,
  }),
  getters: {
    defaultStatus: (state) =>
      state.statusGroups.find((name) => state.statusMeta[name]?.isDefault) ?? 'Backlog',
    visibleStatusGroups: (state) => [
      ...state.statusGroups,
      ...(state.archivedStatusGroups ?? []).filter((name) =>
        state.items.some((issue) => issue.status === name),
      ),
    ],
    activeCount: (state) =>
      state.counts?.active ??
      state.items.filter(
        (issue) =>
          !issue.triage &&
          !['Completed', 'Canceled'].includes(state.statusMeta[issue.status]?.category ?? ''),
      ).length,
    triageCount: (state) =>
      state.counts?.triage ?? state.items.filter((issue) => issue.triage).length,
  },
  actions: {
    applyWorkflow(snapshot: WorkflowSnapshot) {
      const { statuses } = snapshot;
      this.workflowVersion = snapshot.version;
      this.statusGroups = statuses.filter((item) => !item.archived).map((item) => item.name);
      this.archivedStatusGroups = statuses.filter((item) => item.archived).map((item) => item.name);
      this.statusAppearance = Object.fromEntries(
        statuses.map((item) => [item.name, { icon: item.icon, color: item.color }]),
      );
      this.statusMeta = Object.fromEntries(
        statuses.map((item) => [
          item.name,
          {
            category: item.category,
            description: item.description,
            isDefault: item.isDefault,
          },
        ]),
      );
    },
    workflowSnapshot(): WorkflowStatus[] {
      return [...this.statusGroups, ...this.archivedStatusGroups].map((name, position) => ({
        name,
        category: this.statusMeta[name]?.category ?? 'Started',
        description: this.statusMeta[name]?.description ?? '',
        icon: this.iconFor(name),
        color: this.statusAppearance[name]?.color ?? baseColors[name] ?? 'gray',
        position,
        archived: this.archivedStatusGroups.includes(name),
        isDefault: !!this.statusMeta[name]?.isDefault,
      }));
    },
    queueWorkflowSave(renameFrom?: string, renameTo?: string) {
      if (!usesNativeStorage) {
        notifySaved('Workflow updated');
        return;
      }

      const snapshot = this.workflowSnapshot();
      const revision = ++workflowRevision;

      // Issue writes share this queue so a newly created issue never refers to an unsaved status.
      saveQueue = saveQueue.then(async () => {
        try {
          const saved = await saveWorkflow(snapshot, this.workflowVersion, renameFrom, renameTo);
          this.workflowVersion = saved.version;
          await this.refreshCounts();
          this.saveError = '';
          notifySaved('Workflow updated');
        } catch (error) {
          // A newer edit has its own snapshot queued. Let it settle before restoring UI state.
          if (revision !== workflowRevision) return;
          this.saveError = String(error);
          try {
            const [statuses, page] = await Promise.all([listWorkflow(), listIssuePage()]);
            this.applyWorkflow(statuses);
            this.items = page.items.map((item) => ({
              ...item,
              updated: displayUpdated(item.updated),
            }));
          } catch (reloadError) {
            this.loadError = String(reloadError);
          }

          notifications.add({
            title: 'Workflow change failed',
            description: this.saveError,
            type: 'error',
          });
        }
      });
    },
    async loadRelations(id: string) {
      this.relationsByIssue[id] = usesNativeStorage
        ? await listIssueRelations(id)
        : this.relations.filter((relation) => relation.sourceId === id || relation.targetId === id);
    },
    async linkIssue(id: string, targetId: string, kind: IssueRelationKind) {
      if (id === targetId) throw new Error('An issue cannot link to itself.');
      if (
        !this.items.some((item) => item.id === id) ||
        !this.items.some((item) => item.id === targetId)
      )
        throw new Error('Issue no longer exists.');
      if (usesNativeStorage) {
        const source = this.items.find((item) => item.id === id)!;
        const target = this.items.find((item) => item.id === targetId)!;
        await createIssueRelation(id, targetId, kind, source.version ?? 0, target.version ?? 0);
        await this.refreshRelationIssueVersions(id, targetId);
      } else {
        let sourceId = kind === 'blocked_by' ? targetId : id;
        let resolvedTargetId = kind === 'blocked_by' ? id : targetId;
        const storedKind = kind === 'blocked_by' ? 'blocks' : kind;
        if (storedKind !== 'blocks' && sourceId > resolvedTargetId)
          [sourceId, resolvedTargetId] = [resolvedTargetId, sourceId];
        if (
          this.relations.some(
            (relation) =>
              relation.sourceId === sourceId &&
              relation.targetId === resolvedTargetId &&
              relation.kind === storedKind,
          )
        )
          throw new Error('These issues are already linked.');
        if (storedKind === 'blocks') {
          const visited = new Set<string>();
          const pending = [resolvedTargetId];
          while (pending.length) {
            const current = pending.pop()!;
            if (current === sourceId)
              throw new Error('Blocking this issue would create a dependency cycle.');
            if (visited.has(current)) continue;
            visited.add(current);
            pending.push(
              ...this.relations
                .filter((relation) => relation.kind === 'blocks' && relation.sourceId === current)
                .map((relation) => relation.targetId),
            );
          }
        }

        this.relations.push({
          id: crypto.randomUUID(),
          sourceId,
          targetId: resolvedTargetId,
          kind: storedKind,
        });
        this.recordPreviewActivity(id, 'changed', `linked issue ${targetId}`);
      }

      await this.loadRelations(id);
      if (this.relationsByIssue[targetId]) await this.loadRelations(targetId);
      notifySaved(`Relation added to ${id}`);
    },
    async unlinkIssue(id: string, relationId: string) {
      const relation = this.relationsByIssue[id]?.find((item) => item.id === relationId);
      if (!relation) throw new Error('Relation no longer exists.');
      const otherId = relation.sourceId === id ? relation.targetId : relation.sourceId;
      if (usesNativeStorage) {
        const source = this.items.find((item) => item.id === id);
        const target = this.items.find((item) => item.id === otherId);
        await deleteIssueRelation(
          id,
          relationId,
          otherId,
          source?.version ?? 0,
          target?.version ?? 0,
        );
        await this.refreshRelationIssueVersions(id, otherId);
      } else {
        this.relations = this.relations.filter((item) => item.id !== relationId);
        this.recordPreviewActivity(id, 'changed', 'removed an issue relation');
      }

      await this.loadRelations(id);
      if (this.relationsByIssue[otherId]) await this.loadRelations(otherId);
      notifySaved(`Relation removed from ${id}`);
    },
    async refreshRelationIssueVersions(...identifiers: string[]) {
      const records = await listIssueRecords(identifiers);
      for (const record of records) {
        const current = this.items.find((item) => item.id === record.id);
        if (current) current.version = record.version;
      }
    },
    async flushSaves() {
      await saveQueue;
    },
    recordPreviewActivity(
      id: string,
      kind: IssueActivity['kind'],
      body: string,
      document?: RichTextDocument | null,
    ) {
      const entry: IssueActivity = {
        id: crypto.randomUUID(),
        issueId: id,
        actorName: useWorkspaceStore().userName || 'You',
        kind,
        body,
        bodyDocumentJson: document ? JSON.stringify(document) : null,
        createdAt: new Date().toISOString(),
      };
      this.activityByIssue[id] = [entry, ...(this.activityByIssue[id] ?? [])];
      return entry;
    },
    async loadActivity(id: string, more = false) {
      if (usesNativeStorage) {
        const previous = more ? (this.activityByIssue[id] ?? []) : [];
        const page = await listIssueActivity(
          id,
          more ? (this.activityCursorByIssue[id] ?? undefined) : undefined,
        );
        this.activityByIssue[id] = [...previous, ...page.items];
        this.activityCursorByIssue[id] = page.nextCursor;
        this.activityHasMore[id] = Boolean(page.nextCursor);
      } else if (!this.activityByIssue[id]) {
        const issue = [...this.items, ...this.archivedItems].find((item) => item.id === id);
        if (issue) this.recordPreviewActivity(id, 'created', 'created this issue');
        this.activityHasMore[id] = false;
      }
    },
    async addComment(id: string, body: string, document?: RichTextDocument | null) {
      const trimmed = body.trim();
      if (!trimmed || trimmed.length > 100_000)
        throw new Error('Write a comment of up to 100,000 characters.');
      if (!this.items.some((item) => item.id === id)) throw new Error('Issue no longer exists.');
      const entry = usesNativeStorage
        ? await createIssueComment(id, trimmed, document)
        : this.recordPreviewActivity(id, 'comment', trimmed, document);
      if (usesNativeStorage)
        this.activityByIssue[id] = [entry, ...(this.activityByIssue[id] ?? [])];
      notifySaved(`Comment added to ${id}`);
      return entry;
    },
    async editComment(
      id: string,
      entry: IssueActivity,
      body: string,
      document: RichTextDocument | null,
    ) {
      const updated = usesNativeStorage
        ? await updateIssueComment(id, entry.id, body, document, entry.version ?? 0)
        : {
            ...entry,
            body,
            bodyDocumentJson: document ? JSON.stringify(document) : null,
            editedAt: new Date().toISOString(),
            version: (entry.version ?? 1) + 1,
          };
      this.activityByIssue[id] = (this.activityByIssue[id] ?? []).map((item) =>
        item.id === entry.id ? updated : item,
      );
      return updated;
    },
    async removeComment(id: string, entry: IssueActivity) {
      const removed = usesNativeStorage
        ? await removeIssueComment(id, entry.id, entry.version ?? 0)
        : {
            ...entry,
            body: 'Comment deleted',
            bodyDocumentJson: null,
            deletedAt: new Date().toISOString(),
            version: (entry.version ?? 1) + 1,
          };
      this.activityByIssue[id] = (this.activityByIssue[id] ?? []).map((item) =>
        item.id === entry.id ? removed : item,
      );
      return removed;
    },
    async bootstrap() {
      if (!usesNativeStorage || this.ready) return;
      this.loadError = '';
      try {
        const workspaceId = useWorkspaceStore().workspaceId;
        const [active, archived, statuses, counts] = await Promise.all([
          listIssuePage(),
          listIssuePage(true),
          listWorkflow(),
          getIssueCounts(),
        ]);
        if (workspaceId !== useWorkspaceStore().workspaceId) return;
        this.applyWorkflow(statuses);
        this.counts = counts;
        this.items = active.items.map((issue) => ({
          ...issue,
          updated: displayUpdated(issue.updated),
        }));
        this.archivedItems = archived.items.map((issue) => ({
          ...issue,
          updated: displayUpdated(issue.updated),
        }));
        this.archivedCursor = archived.nextCursor;
        this.archivedHasMore = Boolean(archived.nextCursor);
        this.ready = true;
      } catch (error) {
        this.loadError = String(error);
      }
    },
    cacheRecords(records: Issue[]) {
      const known = new Set(this.items.map((issue) => issue.id));
      for (const record of records) {
        const issue = { ...record, updated: displayUpdated(record.updated) };
        if (known.has(issue.id)) {
          const index = this.items.findIndex((item) => item.id === issue.id);
          this.items[index] = issue;
        } else {
          this.items.push(issue);
          known.add(issue.id);
        }
      }
    },
    async reloadFirstPages() {
      const [active, archived, counts] = await Promise.all([
        listIssuePage(),
        listIssuePage(true),
        getIssueCounts(),
      ]);
      this.counts = counts;
      this.items = active.items.map((issue) => ({
        ...issue,
        updated: displayUpdated(issue.updated),
      }));
      this.archivedItems = archived.items.map((issue) => ({
        ...issue,
        updated: displayUpdated(issue.updated),
      }));
      this.archivedCursor = archived.nextCursor;
      this.archivedHasMore = Boolean(archived.nextCursor);
    },
    async loadMoreArchived() {
      if (!usesNativeStorage || !this.archivedHasMore) return;
      let page: Awaited<ReturnType<typeof listIssuePage>>;
      try {
        page = await listIssuePage(true, this.archivedCursor ?? undefined);
      } catch (error) {
        if (String(error).includes('Issue page changed')) {
          await this.reloadFirstPages();
          return;
        }

        throw error;
      }

      const known = new Set(this.archivedItems.map((issue) => issue.id));
      this.archivedItems.push(
        ...page.items
          .filter((issue) => !known.has(issue.id))
          .map((issue) => ({ ...issue, updated: displayUpdated(issue.updated) })),
      );
      this.archivedCursor = page.nextCursor;
      this.archivedHasMore = Boolean(page.nextCursor);
    },
    async refreshCounts() {
      if (usesNativeStorage) this.counts = await getIssueCounts();
    },
    queueSave(work: () => Promise<unknown>, context: string) {
      if (!usesNativeStorage) return;
      saveQueue = saveQueue.then(async () => {
        try {
          await work();
          await this.refreshCounts();
          this.saveError = '';
          notifySaved(context);
        } catch (error) {
          this.saveError = String(error);
          this.items = await listIssuePage()
            .then((page) =>
              page.items.map((issue) => ({ ...issue, updated: displayUpdated(issue.updated) })),
            )
            .catch(() => this.items);
        }
      });
    },
    ensureStatusGroups() {
      this.archivedStatusGroups ??= [];
      this.statusAppearance ??= {};
      this.statusMeta ??= structuredClone(baseMeta);
      if (
        !this.statusGroups.includes('On Hold') &&
        !this.archivedStatusGroups.includes('On Hold')
      ) {
        this.statusGroups.splice(this.statusGroups.indexOf('In Progress') + 1, 0, 'On Hold');
      }
    },
    iconFor(name: string) {
      return this.statusAppearance?.[name]?.icon ?? statusIcon[name] ?? statusIcons[0];
    },
    categoryFor(name: string): WorkflowCategory {
      return this.statusMeta[name]?.category ?? 'Started';
    },
    colorFor(name: string) {
      const color = this.statusAppearance?.[name]?.color;
      return color ? statusPalette[color] : (statusColor[name] ?? statusPalette.gray);
    },
    addStatusGroup(
      name: string,
      appearance?: StatusAppearance,
      category: WorkflowCategory = 'Started',
      description = '',
    ): boolean {
      const trimmed = name.trim();
      if (
        !trimmed ||
        trimmed.length > 32 ||
        [...this.statusGroups, ...(this.archivedStatusGroups ?? [])].some(
          (group) => group.toLowerCase() === trimmed.toLowerCase(),
        )
      )
        return false;
      this.statusGroups.push(trimmed);
      this.statusAppearance[trimmed] = appearance ?? { icon: statusIcons[0], color: 'gray' };
      this.statusMeta[trimmed] = { category, description, isDefault: false };
      this.queueWorkflowSave();
      return true;
    },
    customizeStatusGroup(
      name: string,
      nextName: string,
      appearance: StatusAppearance,
      category?: WorkflowCategory,
      description?: string,
    ): boolean {
      if (statusOrder.includes(name)) return false;
      const trimmed = nextName.trim();
      if (
        !trimmed ||
        trimmed.length > 32 ||
        !statusIcons.includes(appearance.icon as (typeof statusIcons)[number]) ||
        !(appearance.color in statusPalette)
      )
        return false;
      if (
        [...this.statusGroups, ...(this.archivedStatusGroups ?? [])].some(
          (group) => group !== name && group.toLowerCase() === trimmed.toLowerCase(),
        )
      )
        return false;
      const list = this.statusGroups.includes(name) ? this.statusGroups : this.archivedStatusGroups;
      const index = list?.indexOf(name) ?? -1;
      if (index < 0) return false;
      list[index] = trimmed;
      [...this.items, ...this.archivedItems].forEach((issue) => {
        if (issue.status === name) issue.status = trimmed;
      });
      const meta = this.statusMeta[name] ?? {
        category: 'Started',
        description: '',
        isDefault: false,
      };
      if (trimmed !== name) {
        delete this.statusAppearance[name];
        delete this.statusMeta[name];
      }

      this.statusAppearance[trimmed] = appearance;
      this.statusMeta[trimmed] = {
        ...meta,
        category: category ?? meta.category,
        description: description ?? meta.description,
      };
      if (
        this.statusMeta[trimmed]?.isDefault &&
        !['Backlog', 'Unstarted'].includes(this.statusMeta[trimmed]!.category)
      ) {
        this.statusMeta[trimmed]!.isDefault = false;
        this.statusMeta.Backlog!.isDefault = true;
      }

      this.queueWorkflowSave(
        trimmed === name ? undefined : name,
        trimmed === name ? undefined : trimmed,
      );
      return true;
    },
    archiveStatusGroup(name: string): boolean {
      if (
        statusOrder.includes(name) ||
        !this.statusGroups.includes(name) ||
        this.statusMeta[name]?.isDefault
      )
        return false;
      this.statusGroups.splice(this.statusGroups.indexOf(name), 1);
      this.archivedStatusGroups ??= [];
      this.archivedStatusGroups.push(name);
      this.queueWorkflowSave();
      return true;
    },
    restoreStatusGroup(name: string): boolean {
      const index = this.archivedStatusGroups?.indexOf(name) ?? -1;
      if (index < 0) return false;
      this.archivedStatusGroups.splice(index, 1);
      this.statusGroups.push(name);
      this.queueWorkflowSave();
      return true;
    },
    deleteStatusGroup(name: string): boolean {
      if (
        statusOrder.includes(name) ||
        this.statusMeta[name]?.isDefault ||
        (usesNativeStorage && (this.counts?.statuses[name] ?? 0) > 0) ||
        [...this.items, ...this.archivedItems].some((issue) => issue.status === name)
      )
        return false;
      const activeIndex = this.statusGroups.indexOf(name);
      const archivedIndex = this.archivedStatusGroups?.indexOf(name) ?? -1;
      if (activeIndex < 0 && archivedIndex < 0) return false;
      if (activeIndex >= 0) this.statusGroups.splice(activeIndex, 1);
      else this.archivedStatusGroups.splice(archivedIndex, 1);
      delete this.statusAppearance[name];
      delete this.statusMeta[name];
      this.queueWorkflowSave();
      return true;
    },
    setDefaultStatusGroup(name: string): boolean {
      if (
        !this.statusGroups.includes(name) ||
        !['Backlog', 'Unstarted'].includes(this.statusMeta[name]?.category ?? '')
      )
        return false;
      for (const meta of Object.values(this.statusMeta)) meta.isDefault = false;
      this.statusMeta[name]!.isDefault = true;
      this.queueWorkflowSave();
      return true;
    },
    reorderGroup(property: 'status' | 'priority', name: string, index: number) {
      const groups = property === 'status' ? this.statusGroups : this.priorityGroupOrder;
      const previous = groups.indexOf(name as Priority);
      if (previous < 0 || index < 0 || index >= groups.length) return;
      groups.splice(previous, 1);
      groups.splice(index, 0, name as Priority);
      if (property === 'status') this.queueWorkflowSave();
    },
    reorder(id: string, index: number, visibleIds: string[]) {
      const oldIndex = this.items.findIndex((item) => item.id === id);
      if (oldIndex < 0) return;
      const [issue] = this.items.splice(oldIndex, 1);
      if (!issue) return;
      const peers = visibleIds.filter((visibleId) => visibleId !== id);
      const before = this.items.findIndex((item) => item.id === peers[index]);
      const after = this.items.findIndex((item) => item.id === peers.at(-1));
      this.items.splice(before >= 0 ? before : after >= 0 ? after + 1 : oldIndex, 0, issue);
      const identifiers = this.items.map((item) => item.id);
      this.queueSave(async () => {
        const versions = Object.fromEntries(
          identifiers.map((identifier) => {
            const item = this.items.find((candidate) => candidate.id === identifier);
            if (!item?.version) throw new Error('Reload issues before reordering.');
            return [identifier, item.version];
          }),
        );
        await reorderIssues(identifiers, versions);
        const records = await listIssueRecords(identifiers);
        const returnedVersions = new Map(records.map((record) => [record.id, record.version]));
        for (const item of this.items) {
          const version = returnedVersions.get(item.id);
          if (version !== undefined) item.version = version;
        }
      }, `Issue ${id} reordered`);
      if (!usesNativeStorage) notifySaved(`Issue ${id} reordered`);
    },
    moveOnBoard(id: string, property: 'status' | 'priority', value: string, index: number) {
      const oldIndex = this.items.findIndex((item) => item.id === id);
      if (oldIndex < 0) return;
      const [issue] = this.items.splice(oldIndex, 1);
      if (!issue) return;
      if (property === 'status') issue.status = value as Issue['status'];
      else issue.priority = value as Issue['priority'];
      issue.updated = 'Just now';
      const matching = this.items.filter((item) => item[property] === value);
      const next = matching[index];
      if (next) this.items.splice(this.items.indexOf(next), 0, issue);
      else if (matching.length)
        this.items.splice(this.items.indexOf(matching.at(-1)!) + 1, 0, issue);
      else this.items.unshift(issue);
      const input = inputFor(issue);
      const identifiers = this.items.map((item) => item.id);
      this.queueSave(async () => {
        const saved = await updateIssue(id, input, identifiers, issue.version);
        issue.version = saved.version;
        void this.loadActivity(id).catch(() => {});
      }, `${id} ${property} changed to ${value}`);
      if (!usesNativeStorage) notifySaved(`${id} ${property} changed to ${value}`);
      if (!usesNativeStorage)
        this.recordPreviewActivity(id, 'changed', `changed ${property} to ${value}`);
    },
    async create(draft: IssueDraft): Promise<Issue> {
      if (usesNativeStorage) {
        await saveQueue;
        const issue = await createIssue({
          ...draft,
          triage: draft.project === 'Unassigned',
        });
        this.items.unshift({ ...issue, updated: displayUpdated(issue.updated) });
        await this.refreshCounts();
        void this.loadActivity(issue.id).catch(() => {});
        return issue;
      }

      const prefix =
        useProjectsStore().items.find((project) => project.name === draft.project)
          ?.identifierPrefix ?? 'LOCAL';
      const number =
        Math.max(
          0,
          ...this.items
            .filter((issue) => issue.id.startsWith(`${prefix}-`))
            .map((issue) => Number(issue.id.split('-')[1]) || 0),
        ) + 1;
      const issue: Issue = {
        ...draft,
        milestoneId: draft.milestoneId ?? null,
        title: draft.title.trim(),
        description: draft.description.trim(),
        id: `${prefix}-${number}`,
        updated: 'Just now',
        triage: draft.project === 'Unassigned',
      };
      this.items.unshift(issue);
      this.recordPreviewActivity(issue.id, 'created', 'created this issue');
      return issue;
    },
    update(id: string, patch: Partial<Issue>) {
      const issue = this.items.find((item) => item.id === id);
      if (!issue) return;

      if (patch.project !== undefined && patch.project !== issue.project)
        throw new Error('Use the reviewed project move to change an issue’s project.');

      if (
        patch.project !== undefined &&
        patch.project !== issue.project &&
        patch.milestoneId === undefined
      ) {
        patch = { ...patch, milestoneId: null };
      }

      Object.assign(issue, patch, { updated: 'Just now' });
      const input = inputFor(issue);
      this.queueSave(
        async () => {
          const saved = await updateIssue(id, input, undefined, issue.version);
          issue.version = saved.version;
          void this.loadActivity(id).catch(() => {});
        },
        issueSaveMessage(id, patch),
      );
      if (!usesNativeStorage) notifySaved(issueSaveMessage(id, patch));
      if (!usesNativeStorage) this.recordPreviewActivity(id, 'changed', 'updated the issue');
    },
    async save(id: string, patch: Partial<Issue>) {
      if (usesNativeStorage) await saveQueue;
      const index = this.items.findIndex((item) => item.id === id);
      if (index < 0) throw new Error('Issue no longer exists.');
      if (patch.project !== undefined && patch.project !== this.items[index]!.project)
        throw new Error('Use the reviewed project move to change an issue’s project.');
      const next = { ...this.items[index], ...patch } as Issue;
      if (
        patch.project !== undefined &&
        patch.project !== this.items[index]!.project &&
        patch.milestoneId === undefined
      ) {
        next.milestoneId = null;
      }

      if (!next.title.trim()) throw new Error('Enter an issue title.');
      const saved = usesNativeStorage
        ? await updateIssue(id, inputFor(next), undefined, next.version)
        : next;
      this.items.splice(index, 1, {
        ...saved,
        updated: usesNativeStorage ? displayUpdated(saved.updated) : 'Just now',
      });
      await this.refreshCounts();
      if (usesNativeStorage) void this.loadActivity(id).catch(() => {});
      else this.recordPreviewActivity(id, 'changed', 'updated the issue');
      notifySaved(issueSaveMessage(id, patch));
      return this.items[index];
    },
    async previewProjectMove(id: string, destination: string): Promise<IssueMovePreview> {
      await saveQueue;
      if (usesNativeStorage) return previewIssueMove(id, destination);

      const issue = this.items.find((item) => item.id === id);
      if (!issue || issue.project === destination) throw new Error('Choose another project.');
      const descendants = collectDescendantIds(this.items, id);
      const prefix =
        useProjectsStore().items.find((project) => project.name === destination)
          ?.identifierPrefix ?? 'LOCAL';
      const next =
        Math.max(
          0,
          ...this.items
            .filter((item) => item.id.startsWith(`${prefix}-`))
            .map((item) => Number(item.id.slice(prefix.length + 1)) || 0),
        ) + 1;

      return {
        identifier: id,
        destination,
        proposedIdentifier: `${prefix}-${next}`,
        revision: issue.updated,
        affectedCount: descendants.size,
        clearsMilestones: this.items.filter((item) => descendants.has(item.id) && item.milestoneId)
          .length,
        clearsNodes: 0,
        clearsCycles: 0,
      };
    },
    async moveProject(preview: IssueMovePreview, policy: 'keep' | 'regenerate') {
      await saveQueue;
      const source = this.items.find((item) => item.id === preview.identifier);
      if (!source) throw new Error('Issue no longer exists.');
      let moved: Issue[];

      if (usesNativeStorage) {
        moved = await moveIssueProject(preview, policy, source.version ?? 0);
      } else {
        if (source.updated !== preview.revision)
          throw new Error('Issue changed. Review the move again.');
        const affected = collectDescendantIds(this.items, preview.identifier);
        const prefix = preview.proposedIdentifier.replace(/-\d+$/, '');
        let next = Number(preview.proposedIdentifier.slice(prefix.length + 1));
        const aliases = new Map<string, string>();
        moved = this.items
          .filter((item) => affected.has(item.id))
          .map((item) => {
            const id = policy === 'regenerate' ? `${prefix}-${next++}` : item.id;
            aliases.set(item.id, id);
            return {
              ...item,
              id,
              project: preview.destination,
              milestoneId: null,
              triage: preview.destination === 'Unassigned',
              updated: 'Just now',
            };
          });
        moved = moved.map((item) => ({
          ...item,
          parentId: item.parentId ? (aliases.get(item.parentId) ?? item.parentId) : null,
        }));
        if (policy === 'regenerate') {
          for (const [alias, current] of Object.entries(this.identifierAliases)) {
            this.identifierAliases[alias] = aliases.get(current) ?? current;
          }

          for (const [oldId, nextId] of aliases) this.identifierAliases[oldId] = nextId;
        }
      }

      const affected = collectDescendantIds(this.items, preview.identifier);
      this.items = [
        ...this.items.filter((item) => !affected.has(item.id)),
        ...moved.map((item) => ({
          ...item,
          updated: usesNativeStorage ? displayUpdated(item.updated) : item.updated,
        })),
      ];
      await this.refreshCounts();
      notifySaved(`${preview.identifier} moved to ${preview.destination}`);
      if (policy === 'keep') return preview.identifier;

      // Another issue may consume the previewed sequence before the move commits.
      const movedRoot = moved.find((item) => (item.parentId ?? null) === (source.parentId ?? null));
      if (!movedRoot) throw new Error('Moved issue could not be found.');
      return movedRoot.id;
    },
    async archive(id: string, strategy: ChildStrategy = 'all', targetId?: string) {
      await saveQueue;
      const issue = this.items.find((item) => item.id === id);
      if (!issue) throw new Error('Issue no longer exists.');
      if (usesNativeStorage) {
        if (strategy === 'all') await archiveIssue(id, issue.version ?? 0);
        else await archiveIssueWithStrategy(id, strategy, issue.version ?? 0, targetId);
        await this.reloadFirstPages();
      } else {
        if (strategy !== 'all') {
          if (
            strategy === 'move' &&
            (!targetId || !this.items.some((item) => item.id === targetId))
          )
            throw new Error('Choose an active destination issue.');
          const descendants = collectDescendantIds(this.items, id);
          if (targetId && descendants.has(targetId))
            throw new Error('Move sub-issues outside their current issue tree.');
          this.items = this.items.map((item) =>
            item.parentId === id
              ? { ...item, parentId: strategy === 'move' ? targetId : null }
              : item,
          );
        }

        const descendants = collectDescendantIds(this.items, id);
        const archivedAt = new Date().toISOString();
        const moved = this.items.filter((item) => descendants.has(item.id));
        this.items = this.items.filter((item) => !descendants.has(item.id));
        this.archivedItems.push(
          ...moved.map((item) => ({ ...item, archivedAt, archiveRootId: id })),
        );
      }

      notifySaved(`Issue ${id} archived`);
    },
    async deleteArchived(id: string, strategy: ChildStrategy = 'all', targetId?: string) {
      await saveQueue;
      const issue = this.archivedItems.find((item) => item.id === id);
      if (!issue) throw new Error('Archived issue no longer exists.');
      if (usesNativeStorage) {
        await deleteArchivedIssue(id, strategy, issue.version ?? 0, targetId);
        await this.reloadFirstPages();
      } else {
        const descendants = collectDescendantIds(this.archivedItems, id);
        if (strategy === 'move' && (!targetId || !this.items.some((item) => item.id === targetId)))
          throw new Error('Choose an active destination issue.');
        if (strategy === 'all') {
          this.archivedItems = this.archivedItems.filter((item) => !descendants.has(item.id));
        } else {
          const restored = this.archivedItems
            .filter((item) => descendants.has(item.id) && item.id !== id)
            .map((item) => ({
              ...item,
              parentId:
                item.parentId === id ? (strategy === 'move' ? targetId : null) : item.parentId,
              archivedAt: null,
              archiveRootId: null,
            }));
          this.items.push(...restored);
          this.archivedItems = this.archivedItems.filter((item) => !descendants.has(item.id));
        }
      }

      notifySaved(`Issue ${id} deleted`);
    },
    async restore(id: string) {
      await saveQueue;
      if (usesNativeStorage) {
        const issue = this.archivedItems.find((item) => item.id === id);
        if (!issue) throw new Error('Archived issue no longer exists.');
        if (issue.archiveRootId === id) await restoreIssue(id, issue.version ?? 0);
        else await restoreSubIssue(id, issue.version ?? 0);
        await this.reloadFirstPages();
      } else {
        const issue = this.archivedItems.find((item) => item.id === id);
        if (!issue) throw new Error('Issue no longer exists.');
        const descendants =
          issue.archiveRootId === id
            ? new Set(
                this.archivedItems
                  .filter((item) => item.archiveRootId === id)
                  .map((item) => item.id),
              )
            : collectDescendantIds(this.archivedItems, id);
        const moved = this.archivedItems.filter((item) => descendants.has(item.id));
        this.archivedItems = this.archivedItems.filter((item) => !descendants.has(item.id));
        this.items.push(
          ...moved.map((item) => ({
            ...item,
            parentId: item.id === id && issue.archiveRootId !== id ? null : item.parentId,
            archivedAt: null,
            archiveRootId: null,
          })),
        );
      }

      notifySaved(`Issue ${id} restored`);
    },
  },
  // Preview only. Native SQLite replaces this for real work data.
  persist: usesNativeStorage
    ? false
    : {
        pick: [
          'items',
          'archivedItems',
          'activityByIssue',
          'relations',
          'identifierAliases',
          'statusGroups',
          'archivedStatusGroups',
          'statusAppearance',
          'statusMeta',
          'priorityGroupOrder',
        ],
      },
});
