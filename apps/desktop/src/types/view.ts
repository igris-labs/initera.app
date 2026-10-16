import type {
  DueFilter,
  IssueFilterGroup,
  IssueFilterTree,
  Priority,
  Project,
  Status,
} from './issue';
import type { ProjectPriority, ProjectStatus } from '../lib/workspace-storage';

export interface SavedView {
  id: string;
  version?: number;
  name: string;
  description: string;
  layout: 'list' | 'board';
  groupBy: 'none' | 'status' | 'priority' | 'project';
  scope: 'active' | 'all';
  status: Status | 'All statuses';
  statuses?: Status[];
  excludedStatuses?: Status[];
  project?: Project;
  projects?: Project[];
  excludedProjects?: Project[];
  projectStatuses?: ProjectStatus[];
  excludedProjectStatuses?: ProjectStatus[];
  projectPriorities?: ProjectPriority[];
  excludedProjectPriorities?: ProjectPriority[];
  projectLabels?: string[];
  excludedProjectLabels?: string[];
  emptyProjectLabels?: boolean;
  excludedEmptyProjectLabels?: boolean;
  projectLeads?: string[];
  excludedProjectLeads?: string[];
  emptyProjectLead?: boolean;
  excludedEmptyProjectLead?: boolean;
  projectMilestones?: string[];
  excludedProjectMilestones?: string[];
  emptyProjectMilestones?: boolean;
  excludedEmptyProjectMilestones?: boolean;
  milestones?: string[];
  excludedMilestones?: string[];
  emptyMilestone?: boolean;
  excludedEmptyMilestone?: boolean;
  priority?: Priority;
  priorities?: Priority[];
  excludedPriorities?: Priority[];
  label?: string;
  labels?: string[];
  excludedLabels?: string[];
  emptyLabels?: boolean;
  excludedEmptyLabels?: boolean;
  due?: DueFilter;
  advancedGroups?: IssueFilterGroup[];
  filterTree?: IssueFilterTree;
  columns?: string[];
  columnWidths?: Partial<
    Record<
      | 'identifier'
      | 'title'
      | 'status'
      | 'priority'
      | 'labels'
      | 'project'
      | 'node'
      | 'milestone'
      | 'cycle'
      | 'estimate'
      | 'dueDate'
      | 'trackedTime'
      | 'updatedAt',
      number
    >
  >;
  sortField?: 'position' | 'title' | 'status' | 'priority' | 'dueDate' | 'updatedAt';
  sortDirection?: 'asc' | 'desc';
  density?: 'compact' | 'default';
  showEmptyGroups?: boolean;
  includeSubIssues?: boolean;
  favorite: boolean;
}
