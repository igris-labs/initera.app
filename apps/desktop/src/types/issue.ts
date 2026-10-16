import type { RichTextDocument } from './rich-text';
import type { ProjectPriority, ProjectStatus } from '../lib/workspace-storage';

export const statuses = ['Backlog', 'Todo', 'In Progress', 'On Hold', 'Done', 'Canceled'] as const;
export const priorities = ['No priority', 'Low', 'Medium', 'High', 'Urgent'] as const;

export type Status = string;

export type Priority = (typeof priorities)[number];

export type Project = string;

export type DueFilter = 'any' | 'has' | 'none';

export interface IssueFilterRule {
  field:
    | 'due'
    | 'start'
    | 'created'
    | 'updated'
    | 'triaged'
    | 'parent'
    | 'subissues'
    | 'blocks'
    | 'blockedBy'
    | 'related'
    | 'duplicates';
  operator: 'before' | 'after' | 'between' | 'has' | 'none';
  value?: string;
  end?: string;
}

export interface IssueFilterGroup {
  mode: 'all' | 'any';
  rules: IssueFilterRule[];
}

export type IssueFilterTree =
  | { op: 'and' | 'or'; children: IssueFilterTree[] }
  | { field: string; operator: string; value?: string | number | (string | number)[] };

export interface IssueQuery {
  scope: 'active' | 'all';
  triage: boolean;
  statuses: Status[];
  excludedStatuses: Status[];
  priorities: Priority[];
  excludedPriorities: Priority[];
  projects: Project[];
  excludedProjects: Project[];
  projectStatuses: ProjectStatus[];
  excludedProjectStatuses: ProjectStatus[];
  projectPriorities: ProjectPriority[];
  excludedProjectPriorities: ProjectPriority[];
  projectLabels: string[];
  excludedProjectLabels: string[];
  emptyProjectLabels: boolean;
  excludedEmptyProjectLabels: boolean;
  projectLeads: string[];
  excludedProjectLeads: string[];
  emptyProjectLead: boolean;
  excludedEmptyProjectLead: boolean;
  projectMilestones: string[];
  excludedProjectMilestones: string[];
  emptyProjectMilestones: boolean;
  excludedEmptyProjectMilestones: boolean;
  milestones: string[];
  excludedMilestones: string[];
  emptyMilestone: boolean;
  excludedEmptyMilestone: boolean;
  labels: string[];
  excludedLabels: string[];
  emptyLabels: boolean;
  excludedEmptyLabels: boolean;
  due: DueFilter;
  text: string;
  advancedGroups: IssueFilterGroup[];
  filterTree?: IssueFilterTree;
  sortField?: 'position' | 'title' | 'status' | 'priority' | 'dueDate' | 'updatedAt';
  sortDirection?: 'asc' | 'desc';
  includeSubIssues?: boolean;
}

export type Estimate = number;

export interface Issue {
  id: string;
  entityId?: string;
  version?: number;
  title: string;
  titleMarkdown?: string;
  description: string;
  descriptionDocument?: RichTextDocument | null;
  status: Status;
  priority: Priority;
  project: Project;
  milestoneId?: string | null;
  milestoneName?: string | null;
  nodeName?: string | null;
  cycleName?: string | null;
  trackedSeconds?: number;
  labels: string[];
  updated: string;
  created?: string;
  due?: string;
  startDate?: string | null;
  triage?: boolean;
  triageEnteredAt?: string | null;
  triagedAt?: string | null;
  parentId?: string | null;
  archivedAt?: string | null;
  archiveRootId?: string | null;
  estimate?: Estimate | null;
}

export interface IssueDraft {
  title: string;
  titleMarkdown?: string;
  description: string;
  descriptionDocument?: RichTextDocument | null;
  status: Status;
  priority: Priority;
  project: Project;
  milestoneId?: string | null;
  labels: string[];
  due?: string;
  startDate?: string | null;
  parentId?: string | null;
  estimate?: Estimate | null;
}

export interface IssueActivity {
  id: string;
  issueId: string;
  actorName: string;
  kind: 'created' | 'changed' | 'comment' | 'archived' | 'restored';
  body: string;
  bodyDocumentJson?: string | null;
  createdAt: string;
  editedAt?: string | null;
  deletedAt?: string | null;
  version?: number;
  changes?: { field: string; before: unknown; after: unknown }[] | null;
  mutationId?: string | null;
}

export interface StatusVisit {
  statusName: string;
  statusCategory: string;
  enteredAt: string;
  leftAt: string | null;
  mutationId: string | null;
}

export interface CommentSearchResult {
  activityId: string;
  identifier: string;
  body: string;
}

export type IssueRelationKind = 'blocks' | 'blocked_by' | 'related' | 'duplicates';

export interface IssueRelation {
  id: string;
  sourceId: string;
  targetId: string;
  kind: Exclude<IssueRelationKind, 'blocked_by'>;
}
