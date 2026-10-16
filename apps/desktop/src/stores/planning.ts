import { defineStore } from 'pinia';
import {
  archivePlanningNode,
  createPlanningNode,
  dissolvePlanningNode,
  loadPlanningNodes,
  reorderPlanningNode,
  restoreArchivedPlanningNode,
  restoreDissolvedPlanningNode,
  setPlanningNodeIssue,
  updatePlanningNode,
} from '../lib/planning-storage';
import type { PlanningNode, PlanningSnapshot } from '../lib/planning-storage';
import { usesNativeStorage } from '../lib/workspace-storage';

export const usePlanningStore = defineStore('planning', {
  state: (): {
    items: PlanningNode[];
    loadedProjects: string[];
    versions: Record<string, number>;
  } => ({
    items: [],
    loadedProjects: [],
    versions: {},
  }),
  actions: {
    async load(projectId: string) {
      if (this.loadedProjects.includes(projectId)) return;
      if (usesNativeStorage) {
        this.replace(projectId, await loadPlanningNodes(projectId));
      }

      this.loadedProjects.push(projectId);
    },
    replace(projectId: string, snapshot: PlanningSnapshot) {
      this.items = [
        ...this.items.filter((node) => node.projectId !== projectId),
        ...snapshot.items,
      ];
      this.versions[projectId] = snapshot.version;
    },
    async create(
      projectId: string,
      parentId: string | null,
      kind: PlanningNode['kind'],
      name: string,
      description: string,
    ) {
      const cleanName = name.trim();
      if (!cleanName || cleanName.length > 120)
        throw new Error('Enter a name of up to 120 characters.');
      if (
        parentId &&
        !this.items.some((node) => node.id === parentId && node.projectId === projectId)
      )
        throw new Error('Parent must belong to this project.');
      if (usesNativeStorage) {
        this.replace(
          projectId,
          await createPlanningNode(
            projectId,
            parentId,
            kind,
            cleanName,
            description,
            this.versions[projectId] ?? 0,
          ),
        );
      } else {
        this.items.push({
          id: crypto.randomUUID(),
          projectId,
          parentId,
          kind,
          name: cleanName,
          description,
          position: this.items.filter(
            (node) => node.projectId === projectId && node.parentId === parentId,
          ).length,
          issueIds: [],
          archivedAt: null,
          deletedAt: null,
        });
      }
    },
    async update(node: PlanningNode) {
      if (!node.name.trim() || node.name.length > 120)
        throw new Error('Enter a name of up to 120 characters.');
      // Reject moves into any descendant, regardless of nesting depth.
      const descendants = new Set([node.id]);
      let changed = true;
      while (changed) {
        changed = false;
        for (const item of this.items) {
          if (item.parentId && descendants.has(item.parentId) && !descendants.has(item.id)) {
            descendants.add(item.id);
            changed = true;
          }
        }
      }

      if (node.parentId && descendants.has(node.parentId))
        throw new Error('A node cannot be nested under itself or its descendants.');
      if (usesNativeStorage)
        this.replace(
          node.projectId,
          await updatePlanningNode(node, this.versions[node.projectId] ?? 0),
        );
      else
        this.items.splice(
          this.items.findIndex((item) => item.id === node.id),
          1,
          { ...node, name: node.name.trim() },
        );
    },
    async reorder(node: PlanningNode, index: number) {
      const siblings = this.items
        .filter(
          (item) =>
            item.projectId === node.projectId &&
            item.parentId === node.parentId &&
            !item.archivedAt &&
            !item.deletedAt,
        )
        .sort((a, b) => a.position - b.position);
      if (index < 0 || index >= siblings.length) return;
      if (usesNativeStorage) {
        this.replace(
          node.projectId,
          await reorderPlanningNode(
            node.id,
            node.projectId,
            index,
            this.versions[node.projectId] ?? 0,
          ),
        );
        return;
      }

      const old = siblings.findIndex((item) => item.id === node.id);
      const [moved] = siblings.splice(old, 1);
      siblings.splice(index, 0, moved!);
      siblings.forEach((item, position) => (item.position = position));
    },
    async archive(node: PlanningNode) {
      if (usesNativeStorage) {
        this.replace(
          node.projectId,
          await archivePlanningNode(node.id, node.projectId, this.versions[node.projectId] ?? 0),
        );
        return;
      }

      const group = crypto.randomUUID();
      for (const item of this.subtree(node)) {
        if (!item.archivedAt) {
          item.archivedAt = group;
        }
      }
    },
    async restoreArchived(node: PlanningNode) {
      if (usesNativeStorage) {
        this.replace(
          node.projectId,
          await restoreArchivedPlanningNode(
            node.id,
            node.projectId,
            this.versions[node.projectId] ?? 0,
          ),
        );
        return;
      }

      const group = node.archivedAt;
      for (const item of this.items) {
        if (item.archivedAt === group) item.archivedAt = null;
      }
    },
    subtree(node: PlanningNode): PlanningNode[] {
      return [
        node,
        ...this.items
          .filter((item) => item.parentId === node.id)
          .flatMap((item) => this.subtree(item)),
      ];
    },
    async dissolve(node: PlanningNode) {
      if (usesNativeStorage) {
        this.replace(
          node.projectId,
          await dissolvePlanningNode(node.id, node.projectId, this.versions[node.projectId] ?? 0),
        );
        return;
      }

      node.deletedAt = new Date().toISOString();
      node.dissolveSnapshot = {
        childIds: this.items.filter((item) => item.parentId === node.id).map((item) => item.id),
        issueIds: [...node.issueIds],
      };
      for (const child of this.items.filter((item) => item.parentId === node.id))
        child.parentId = node.parentId;
      const parent = this.items.find((item) => item.id === node.parentId);
      if (parent) parent.issueIds.push(...node.issueIds);
      node.issueIds = [];
    },
    async restoreDissolved(node: PlanningNode, memberships: boolean) {
      if (usesNativeStorage) {
        this.replace(
          node.projectId,
          await restoreDissolvedPlanningNode(
            node.id,
            node.projectId,
            memberships,
            this.versions[node.projectId] ?? 0,
          ),
        );
        return;
      }

      node.deletedAt = null;
      if (memberships) {
        for (const child of this.items.filter(
          (item) =>
            node.dissolveSnapshot?.childIds.includes(item.id) && item.parentId === node.parentId,
        ))
          child.parentId = node.id;
        const parent = this.items.find((item) => item.id === node.parentId);
        for (const issueId of node.dissolveSnapshot?.issueIds ?? []) {
          if (!this.items.some((item) => item.issueIds.includes(issueId)))
            node.issueIds.push(issueId);
          else if (parent?.issueIds.includes(issueId)) {
            parent.issueIds = parent.issueIds.filter((id) => id !== issueId);
            node.issueIds.push(issueId);
          }
        }
      }
    },
    async assign(projectId: string, identifier: string, nodeId: string | null) {
      if (nodeId && !this.items.some((node) => node.id === nodeId && node.projectId === projectId))
        throw new Error('Node not found in this project.');
      if (usesNativeStorage) {
        this.replace(
          projectId,
          await setPlanningNodeIssue(projectId, identifier, nodeId, this.versions[projectId] ?? 0),
        );
      } else {
        for (const node of this.items)
          node.issueIds = node.issueIds.filter((id) => id !== identifier);
        const target = this.items.find((node) => node.id === nodeId);
        if (target) target.issueIds.push(identifier);
      }
    },
  },
  persist: usesNativeStorage ? false : { pick: ['items'] },
});
