import { isTauri } from '@tauri-apps/api/core';
import { invoke } from './native-invoke';
import type { ThemePreference } from '../types/preferences';
import type { RichTextDocument } from '../types/rich-text';
import type { IconColor } from './icon-presentation';
import type { EstimateScale } from './estimates';

export interface WorkspaceIdentity {
  workspaceId: string;
  actorId: string;
  userName: string;
  workspaceName: string;
}

export interface WorkspaceBootstrap {
  dataRoot: string;
  contractVersion: number;
  workspaceRevision: number;
  identityVersion: number | null;
  identity: WorkspaceIdentity | null;
  workspaces: { id: string; name: string }[];
  restoreError: string | null;
  restoredBackup: boolean;
  recoveredTimers: number;
}

export interface ProjectRecord {
  id: string;
  version?: number;
  workspaceId: string;
  name: string;
  nameMarkdown: string;
  identifierPrefix: string;
  icon: string;
  color: IconColor;
  createdAt: string;
  updatedAt: string;
  summary: string;
  description: string;
  descriptionDocument?: RichTextDocument | null;
  status: ProjectStatus;
  priority: ProjectPriority;
  leadActorId: string | null;
  startDate: string | null;
  targetDate: string | null;
  labels: string[];
  resources: ProjectResource[];
  milestones: MilestoneRecord[];
}

export interface ProjectResource {
  id: string;
  title: string;
  url: string;
}

export interface ProjectUpdate {
  id: string;
  projectId: string;
  actorName: string;
  body: string;
  kind?: 'update' | 'change';
  createdAt: string;
}

export type ProjectStatus = 'Backlog' | 'Planned' | 'In Progress' | 'Completed' | 'Canceled';

export type ProjectPriority = 'No priority' | 'Low' | 'Medium' | 'High' | 'Urgent';

export interface MilestoneInput {
  id?: string;
  title: string;
  description: string;
  targetDate: string | null;
}

export interface MilestoneRecord extends MilestoneInput {
  id: string;
  archivedAt?: string | null;
}

export interface RemovedMilestone extends MilestoneRecord {
  detachedIssueCount: number;
}

export interface MilestoneRestoreReport {
  reattached: number;
  skipped: number;
}

export interface ProjectInput {
  name: string;
  nameMarkdown: string;
  identifierPrefix: string;
  icon: string;
  color: IconColor;
  summary: string;
  description: string;
  descriptionDocument?: RichTextDocument | null;
  status: ProjectStatus;
  priority: ProjectPriority;
  leadActorId: string | null;
  startDate: string | null;
  targetDate: string | null;
  labels: string[];
  resources: ProjectResource[];
  milestones: MilestoneInput[];
}

export interface WorkspacePreferences {
  theme: ThemePreference;
  issueScope: 'active' | 'all';
  issueLayout: 'list' | 'board';
  boardGroupBy: 'status' | 'priority';
  groupIssuesByStatus: boolean;
  showIssuePriority: boolean;
  showIssueProject: boolean;
  estimateScale: EstimateScale;
  timerOnExit: 'pause' | 'continue';
  concurrentTimers: boolean;
  timerOnSleep: 'pause' | 'include';
}

export const usesNativeStorage = isTauri();

export function loadWorkspace(): Promise<WorkspaceBootstrap> {
  return invoke<WorkspaceBootstrap>('storage_bootstrap');
}

export async function saveWorkspaceIdentity(
  userName: string,
  workspaceName: string,
  expectedVersion: number | null,
) {
  const identity = await invoke<WorkspaceIdentity>('storage_save_identity', {
    userName,
    workspaceName,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion },
  });
  const version = await invoke<number>('storage_identity_version');
  return { identity, version };
}

export function createWorkspace(name: string) {
  return invoke<WorkspaceIdentity>('storage_create_workspace', {
    name,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: null },
  });
}

export function switchWorkspace(id: string) {
  return invoke<WorkspaceIdentity>('storage_switch_workspace', {
    id,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: null },
  });
}

export interface ProjectIndexQuery {
  sort: 'updated' | 'name' | 'status' | 'target' | 'priority';
  status: string | null;
  priority: string | null;
  search: string;
}

export function loadProjectPage(cursor?: string, limit = 200, query?: ProjectIndexQuery) {
  return invoke<{ items: ProjectRecord[]; nextCursor: string | null; totalCount: number }>(
    'storage_list_projects_page',
    { cursor, limit, ...query },
  );
}

export function loadProject(id: string) {
  return invoke<ProjectRecord>('storage_get_project', { id });
}

export function loadProjectByName(name: string) {
  return invoke<ProjectRecord>('storage_get_project_by_name', { name });
}

export interface ProjectLifecycleEntry {
  id: string;
  name: string;
  state: 'archived' | 'trash';
  version: number;
}

export interface ProjectLifecyclePreview {
  activeIssues: number;
  nodes: number;
  milestones: number;
  runningTimers: number;
  cycleDependencies: number;
}

export function previewProjectLifecycle(id: string) {
  return invoke<ProjectLifecyclePreview>('storage_preview_project_lifecycle', { id });
}

export function loadInactiveProjects() {
  return invoke<ProjectLifecycleEntry[]>('storage_list_inactive_projects');
}

export function setProjectLifecycle(
  id: string,
  action: 'archive' | 'trash' | 'restore',
  expectedVersion: number,
  mutationId = crypto.randomUUID(),
) {
  return invoke<void>('storage_set_project_lifecycle', {
    id,
    action,
    mutation: { mutationId, expectedVersion },
  });
}

export function loadRemovedMilestones(projectId: string) {
  return invoke<RemovedMilestone[]>('storage_list_removed_milestones', { projectId });
}

export function previewMilestoneRemoval(projectId: string, milestoneId: string) {
  return invoke<number>('storage_preview_milestone_removal', { projectId, milestoneId });
}

export function setMilestoneArchived(
  projectId: string,
  milestoneId: string,
  archived: boolean,
  version: number,
) {
  return invoke<ProjectRecord>('storage_set_milestone_archived', {
    projectId,
    milestoneId,
    archived,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: version },
  });
}

export function restoreMilestone(
  projectId: string,
  milestoneId: string,
  reassignIssues: boolean,
  version: number,
) {
  return invoke<MilestoneRestoreReport>('storage_restore_milestone', {
    projectId,
    milestoneId,
    reassignIssues,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: version },
  });
}

export function searchProjectIds(text: string) {
  return invoke<string[]>('storage_search_projects', { text });
}

export function createProject(input: ProjectInput, mutationId = crypto.randomUUID()) {
  return invoke<ProjectRecord>('storage_create_project_full', {
    input,
    mutation: { mutationId },
  });
}

export function updateProject(
  id: string,
  input: ProjectInput,
  expectedVersion: number | undefined,
  mutationId = crypto.randomUUID(),
) {
  if (!expectedVersion) throw new Error('Reload the project before saving changes.');
  return invoke<ProjectRecord>('storage_update_project_full', {
    id,
    input,
    mutation: { mutationId, expectedVersion },
  });
}

export function changeProjectResource(
  projectId: string,
  expectedVersion: number,
  action: 'create' | 'update' | 'remove' | 'reorder',
  options: { resource?: ProjectResource; resourceId?: string; order?: string[] },
) {
  return invoke<ProjectRecord>('storage_change_project_resource', {
    projectId,
    action,
    ...options,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion },
  });
}

export function loadProjectUpdates(projectId: string, cursor?: string) {
  return invoke<{ items: ProjectUpdate[]; nextCursor: string | null }>(
    'storage_list_project_updates',
    { projectId, cursor },
  );
}

export function createProjectUpdate(
  projectId: string,
  body: string,
  mutationId = crypto.randomUUID(),
) {
  return invoke<ProjectUpdate>('storage_create_project_update', {
    projectId,
    body,
    mutation: { mutationId },
  });
}

export function loadPreferences() {
  return invoke<{ preferences: WorkspacePreferences; version: number }>('storage_load_preferences');
}

export function savePreferences(
  preferences: WorkspacePreferences,
  version: number,
  keepTimerIssue?: string,
) {
  return invoke<{ preferences: WorkspacePreferences; version: number }>(
    'storage_save_preferences',
    {
      preferences,
      keepTimerIssue,
      mutation: { mutationId: crypto.randomUUID(), expectedVersion: version },
    },
  );
}
