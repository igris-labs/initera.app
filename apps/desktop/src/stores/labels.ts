import { defineStore } from 'pinia';
import { invoke } from '../lib/native-invoke';
import {
  archiveLabel,
  createLabel,
  listLabels,
  restoreArchivedLabel,
  restoreTrashedLabel,
  trashLabel,
  updateLabel,
} from '../lib/label-storage';
import type { LabelRecord, LabelRestoreReport } from '../lib/label-storage';
import { labelColor } from '../lib/label-presentation';
import { usesNativeStorage } from '../lib/workspace-storage';
import { useIssuesStore } from './issues';
import { useProjectsStore } from './projects';

const defaults = Object.entries(labelColor).map(([name, color]) => ({
  id: `preview-${name}`,
  version: 1,
  name,
  color,
  archivedAt: null,
  deletedAt: null,
  issueCount: 0,
  projectCount: 0,
}));

function validLabel(name: string, color: string) {
  if (!name.trim() || name.trim().length > 80 || !/^#[\da-f]{6}$/i.test(color))
    throw new Error('Enter a label name and a six-digit hex color.');
}

export const useLabelsStore = defineStore('labels', {
  state: (): { items: LabelRecord[]; ready: boolean; error: string } => ({
    items: usesNativeStorage ? [] : structuredClone(defaults),
    ready: !usesNativeStorage,
    error: '',
  }),
  getters: {
    assignable: (state) => state.items.filter((item) => !item.archivedAt && !item.deletedAt),
  },
  actions: {
    apply(items: LabelRecord[]) {
      this.items = items;
      for (const name of Object.keys(labelColor)) delete labelColor[name];
      for (const item of items) if (!item.deletedAt) labelColor[item.name] = item.color;
    },
    async bootstrap() {
      if (!usesNativeStorage || this.ready) return;
      try {
        this.apply(await listLabels());
        this.ready = true;
        this.error = '';
      } catch (error) {
        this.error = String(error);
      }
    },
    async refreshOwners() {
      if (!usesNativeStorage) return;
      const owners = [useIssuesStore().items, useIssuesStore().archivedItems];
      const projectOwners = useProjectsStore().items;

      for (const issues of owners) {
        for (let offset = 0; offset < issues.length; offset += 200) {
          const batch = issues.slice(offset, offset + 200);
          const labels = await invoke<Record<string, string[]>>('storage_list_owner_labels', {
            owners: batch.map((issue) => issue.id),
            project: false,
          });
          for (const issue of batch) issue.labels = labels[issue.id] ?? [];
        }
      }

      for (let offset = 0; offset < projectOwners.length; offset += 200) {
        const batch = projectOwners.slice(offset, offset + 200);
        const labels = await invoke<Record<string, string[]>>('storage_list_owner_labels', {
          owners: batch.map((project) => project.id),
          project: true,
        });
        for (const project of batch) project.labels = labels[project.id] ?? [];
      }
    },
    async create(name: string, color: string): Promise<void> {
      validLabel(name, color);
      if (
        this.items.some(
          (item) => !item.deletedAt && item.name.toLowerCase() === name.trim().toLowerCase(),
        )
      )
        throw new Error('A label with this name already exists.');
      if (usesNativeStorage) this.apply(await createLabel(name.trim(), color));
      else
        this.apply([
          ...this.items,
          {
            id: crypto.randomUUID(),
            version: 1,
            name: name.trim(),
            color,
            archivedAt: null,
            deletedAt: null,
            issueCount: 0,
            projectCount: 0,
          },
        ]);
    },
    async update(id: string, name: string, color: string) {
      validLabel(name, color);
      const label = this.items.find((item) => item.id === id);
      if (!label || label.deletedAt) throw new Error('Active label not found.');
      if (usesNativeStorage) {
        this.apply(await updateLabel(id, name.trim(), color, label.version));
        await this.refreshOwners();
        return;
      }

      const previous = label.name;
      label.name = name.trim();
      label.color = color;
      for (const owner of [...useIssuesStore().items, ...useProjectsStore().items])
        owner.labels = owner.labels.map((value) => (value === previous ? label.name : value));
      this.apply([...this.items]);
    },
    async archive(id: string) {
      const label = this.items.find((item) => item.id === id);
      if (!label) throw new Error('Label not found.');
      if (usesNativeStorage) this.apply(await archiveLabel(id, label.version));
      else {
        label.archivedAt = new Date().toISOString();
        label.version++;
      }
    },
    async restoreArchived(id: string) {
      const label = this.items.find((item) => item.id === id);
      if (!label) throw new Error('Label not found.');
      if (usesNativeStorage) this.apply(await restoreArchivedLabel(id, label.version));
      else {
        label.archivedAt = null;
        label.version++;
      }
    },
    async trash(id: string) {
      if (usesNativeStorage) {
        const label = this.items.find((item) => item.id === id);
        if (!label) throw new Error('Label not found.');
        this.apply(await trashLabel(id, label.version));
        await this.refreshOwners();
        return;
      }

      const label = this.items.find((item) => item.id === id);
      if (!label || label.deletedAt) return;
      const issues = useIssuesStore().items.filter((issue) => issue.labels.includes(label.name));
      const projects = useProjectsStore().items.filter((project) =>
        project.labels.includes(label.name),
      );
      label.detached = {
        issueIds: issues.map((issue) => issue.id),
        projectIds: projects.map((project) => project.id),
      };
      for (const owner of [...issues, ...projects])
        owner.labels = owner.labels.filter((value) => value !== label.name);
      label.deletedAt = new Date().toISOString();
      this.apply([...this.items]);
    },
    async restoreTrashed(id: string): Promise<LabelRestoreReport> {
      const label = this.items.find((item) => item.id === id);
      if (!label?.deletedAt) throw new Error('Trashed label not found.');
      if (
        this.items.some(
          (item) =>
            item.id !== id &&
            !item.deletedAt &&
            item.name.toLowerCase() === label.name.toLowerCase(),
        )
      )
        throw new Error('Another label already uses this name.');
      if (usesNativeStorage) {
        const report = await restoreTrashedLabel(id, label.version);
        this.apply(await listLabels());
        await this.refreshOwners();
        return report;
      }

      label.deletedAt = null;
      let restored = 0;
      let skipped = 0;
      for (const [owners, ids] of [
        [useIssuesStore().items, label.detached?.issueIds ?? []],
        [useProjectsStore().items, label.detached?.projectIds ?? []],
      ] as const) {
        for (const id of ids) {
          const owner = owners.find((item) => item.id === id);
          if (owner && !owner.labels.includes(label.name)) {
            owner.labels.push(label.name);
            restored++;
          } else skipped++;
        }
      }

      label.detached = undefined;
      this.apply([...this.items]);
      return { restored, skipped };
    },
  },
  persist: usesNativeStorage ? false : { pick: ['items'] },
});
