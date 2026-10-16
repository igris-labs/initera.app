import { invoke } from './native-invoke';
import type { RichTextDocument } from '../types/rich-text';
import type {
  CommentSearchResult,
  Issue,
  IssueActivity,
  IssueDraft,
  IssueQuery,
  IssueRelation,
  IssueRelationKind,
  StatusVisit,
} from '../types/issue';

export function listIssueStatusVisits(identifier: string) {
  return invoke<StatusVisit[]>('storage_list_issue_status_visits', { identifier });
}

export type IssueInput = IssueDraft & Pick<Issue, 'triage'>;

export interface IssueCounts {
  active: number;
  triage: number;
  archived: number;
  statuses: Record<string, number>;
  projects: Record<
    string,
    {
      total: number;
      completed: number;
      milestones: Record<string, { total: number; done: number }>;
    }
  >;
}

export function getIssueCounts() {
  return invoke<IssueCounts>('storage_issue_counts');
}

export function getIssueDescendantCount(identifier: string) {
  return invoke<number>('storage_issue_descendant_count', { identifier });
}

export function listIssueChildren(identifier: string, cursor?: string, limit = 50) {
  return invoke<{ items: Issue[]; totalCount: number; nextCursor: string | null }>(
    'storage_list_issue_children',
    {
      identifier,
      cursor,
      limit,
    },
  );
}

export function getIssueStatuses(identifiers: string[]) {
  return invoke<Record<string, string>>('storage_issue_statuses', { identifiers });
}

export interface IssueMovePreview {
  identifier: string;
  destination: string;
  proposedIdentifier: string;
  revision: string;
  affectedCount: number;
  clearsMilestones: number;
  clearsNodes: number;
  clearsCycles: number;
}

export function previewIssueMove(identifier: string, destination: string) {
  return invoke<IssueMovePreview>('storage_preview_issue_move', { identifier, destination });
}

export function moveIssueProject(
  preview: IssueMovePreview,
  policy: 'keep' | 'regenerate',
  version: number,
) {
  return invoke<Issue[]>('storage_move_issue_project', {
    identifier: preview.identifier,
    destination: preview.destination,
    revision: preview.revision,
    policy,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: version },
  });
}

export function listIssuePage(archived = false, cursor?: string, limit = 200) {
  return invoke<{ items: Issue[]; nextCursor: string | null }>('storage_list_issue_page', {
    limit,
    archived,
    cursor,
  });
}

export function listIssueRecords(identifiers: string[], archived = false) {
  return invoke<Issue[]>('storage_list_issue_records', { identifiers, archived });
}

export function searchIssueIds(text: string) {
  return invoke<string[]>('storage_search_issues', { text });
}

export interface PlanningNodeSearchResult {
  id: string;
  projectName: string;
  name: string;
  kind: string;
}

export function searchPlanningNodes(text: string) {
  return invoke<PlanningNodeSearchResult[]>('storage_search_planning_nodes', { text });
}

export function bulkTriage(
  targets: { identifier: string; version: number }[],
  accept: boolean,
  status: string,
  reason?: string,
) {
  return invoke<Issue[]>('storage_bulk_triage', {
    targets,
    accept,
    status,
    reason,
    mutation: { mutationId: crypto.randomUUID() },
  });
}

export function searchComments(text: string) {
  return invoke<CommentSearchResult[]>('storage_search_comments', { text });
}

export function findComment(identifier: string, activityId: string) {
  return invoke<IssueActivity | null>('storage_find_comment', { identifier, activityId });
}

export function queryIssueIdPage(query: IssueQuery, limit = 200, cursor?: string) {
  return invoke<{ ids: string[]; totalCount: number; nextCursor: string | null }>(
    'storage_query_issue_id_page',
    {
      query,
      limit,
      cursor,
    },
  );
}

function issueMutation(version: number) {
  return { mutationId: crypto.randomUUID(), expectedVersion: version };
}

export function archiveIssue(identifier: string, version: number) {
  return invoke<void>('storage_archive_issue', { identifier, mutation: issueMutation(version) });
}

export type ChildStrategy = 'all' | 'detach' | 'move';

export function archiveIssueWithStrategy(
  identifier: string,
  strategy: ChildStrategy,
  version: number,
  targetIdentifier?: string,
) {
  return invoke<void>('storage_archive_issue_with_strategy', {
    identifier,
    strategy,
    targetIdentifier,
    mutation: issueMutation(version),
  });
}

export function deleteArchivedIssue(
  identifier: string,
  strategy: ChildStrategy,
  version: number,
  targetIdentifier?: string,
) {
  return invoke<void>('storage_delete_archived_issue', {
    identifier,
    strategy,
    targetIdentifier,
    mutation: issueMutation(version),
  });
}

export function restoreIssue(identifier: string, version: number) {
  return invoke<void>('storage_restore_issue', { identifier, mutation: issueMutation(version) });
}

export function restoreSubIssue(identifier: string, version: number) {
  return invoke<void>('storage_restore_sub_issue', {
    identifier,
    mutation: issueMutation(version),
  });
}

export function listIssueActivity(identifier: string, cursor?: string) {
  return invoke<{ items: IssueActivity[]; nextCursor: string | null }>(
    'storage_list_issue_activity',
    { identifier, cursor },
  );
}

export function listIssueRelations(identifier: string) {
  return invoke<IssueRelation[]>('storage_list_issue_relations', { identifier });
}

export function createIssueRelation(
  identifier: string,
  targetIdentifier: string,
  kind: IssueRelationKind,
  version: number,
  targetVersion: number,
  mutationId = crypto.randomUUID(),
) {
  return invoke<IssueRelation>('storage_create_issue_relation', {
    identifier,
    targetIdentifier,
    kind,
    targetVersion,
    mutation: { mutationId, expectedVersion: version },
  });
}

export function deleteIssueRelation(
  identifier: string,
  relationId: string,
  targetIdentifier: string,
  version: number,
  targetVersion: number,
  mutationId = crypto.randomUUID(),
) {
  return invoke<void>('storage_delete_issue_relation', {
    identifier,
    relationId,
    targetIdentifier,
    targetVersion,
    mutation: { mutationId, expectedVersion: version },
  });
}

export function createIssueComment(
  identifier: string,
  body: string,
  bodyDocument?: RichTextDocument | null,
  mutationId = crypto.randomUUID(),
) {
  return invoke<IssueActivity>('storage_create_issue_comment', {
    identifier,
    body,
    bodyDocument,
    mutation: { mutationId },
  });
}

export function updateIssueComment(
  identifier: string,
  commentId: string,
  body: string,
  bodyDocument: RichTextDocument | null,
  version: number,
) {
  return invoke<IssueActivity>('storage_update_issue_comment', {
    identifier,
    commentId,
    body,
    bodyDocument,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: version },
  });
}

export function removeIssueComment(identifier: string, commentId: string, version: number) {
  return invoke<IssueActivity>('storage_remove_issue_comment', {
    identifier,
    commentId,
    mutation: { mutationId: crypto.randomUUID(), expectedVersion: version },
  });
}

export function createIssue(input: IssueInput, mutationId = crypto.randomUUID()) {
  return invoke<Issue>('storage_create_issue', { input, mutation: { mutationId } });
}

export function updateIssue(
  identifier: string,
  input: IssueInput,
  order: string[] | undefined,
  expectedVersion: number | undefined,
  mutationId = crypto.randomUUID(),
) {
  if (!expectedVersion) throw new Error('Reload the issue before saving changes.');
  return invoke<Issue>('storage_update_issue', {
    identifier,
    input,
    order,
    mutation: { mutationId, expectedVersion },
  });
}

export function reorderIssues(identifiers: string[], versions: Record<string, number>) {
  return invoke<void>('storage_reorder_issues', {
    identifiers,
    mutation: { mutationId: crypto.randomUUID(), versions },
  });
}
