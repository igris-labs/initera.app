import { invoke } from './native-invoke';

export interface PlanningNode {
  id: string;
  projectId: string;
  parentId: string | null;
  kind: 'module' | 'feature';
  name: string;
  description: string;
  position: number;
  issueIds: string[];
  archivedAt?: string | null;
  deletedAt?: string | null;
  dissolveSnapshot?: { childIds: string[]; issueIds: string[] };
}

export interface PlanningSnapshot {
  items: PlanningNode[];
  version: number;
}

function mutate(command: string, input: Record<string, unknown>, version: number) {
  return invoke<PlanningSnapshot>(command, {
    ...input,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: version },
  });
}

export function loadPlanningNodes(projectId: string) {
  return invoke<PlanningSnapshot>('storage_list_planning_nodes', { projectId });
}

export function createPlanningNode(
  projectId: string,
  parentId: string | null,
  kind: PlanningNode['kind'],
  name: string,
  description: string,
  version: number,
) {
  return mutate(
    'storage_create_planning_node',
    {
      projectId,
      parentId,
      kind,
      name,
      description,
    },
    version,
  );
}

export function updatePlanningNode(node: PlanningNode, version: number) {
  return mutate(
    'storage_update_planning_node',
    {
      id: node.id,
      projectId: node.projectId,
      parentId: node.parentId,
      kind: node.kind,
      name: node.name,
      description: node.description,
    },
    version,
  );
}

export function reorderPlanningNode(id: string, projectId: string, index: number, version: number) {
  return mutate('storage_reorder_planning_node', { id, projectId, index }, version);
}

export function archivePlanningNode(id: string, projectId: string, version: number) {
  return mutate('storage_archive_planning_node', { id, projectId }, version);
}

export function restoreArchivedPlanningNode(id: string, projectId: string, version: number) {
  return mutate('storage_restore_archived_planning_node', { id, projectId }, version);
}

export function dissolvePlanningNode(id: string, projectId: string, version: number) {
  return mutate('storage_dissolve_planning_node', { id, projectId }, version);
}

export function restoreDissolvedPlanningNode(
  id: string,
  projectId: string,
  memberships: boolean,
  version: number,
) {
  return mutate('storage_restore_dissolved_planning_node', { id, projectId, memberships }, version);
}

export function setPlanningNodeIssue(
  projectId: string,
  identifier: string,
  nodeId: string | null,
  version: number,
) {
  return mutate(
    'storage_set_planning_node_issue',
    {
      projectId,
      identifier,
      nodeId,
    },
    version,
  );
}
