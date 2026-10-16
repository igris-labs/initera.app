import { computed } from 'vue';
import type { ComputedRef, Ref } from 'vue';
import { useNativeIssuePages } from './useNativeIssuePages';
import { useIssuesStore } from '../stores/issues';
import { useProjectsStore } from '../stores/projects';
import { useLabelsStore } from '../stores/labels';
import { usesNativeStorage } from '../lib/workspace-storage';
import { savedViewQuery } from '../lib/saved-view-query';
import { matchesAdvancedFilters } from '../lib/advanced-issue-filter';
import { matchesIssueFilterTree } from '../lib/issue-filter-tree';
import type { Issue, IssueQuery, Project } from '../types/issue';
import type { SavedView } from '../types/view';
import type { useIssueFilterModels } from './useIssueFilterModels';
import type { useIssueTableSettings } from './useIssueTableSettings';

interface QueryOptions {
  currentView: ComputedRef<SavedView | undefined>;
  filters: ReturnType<typeof useIssueFilterModels>;
  table: ReturnType<typeof useIssueTableSettings>;
  search: Ref<string>;
  project: ComputedRef<Project | undefined>;
  scope: ComputedRef<'active' | 'all'>;
  isTriage: ComputedRef<boolean>;
  layout: ComputedRef<'list' | 'board'>;
  grouping: ComputedRef<'none' | 'status' | 'priority' | 'project'>;
  unavailableLabels: ComputedRef<string[]>;
  invalidTreeValues: ComputedRef<string[]>;
}

export function useIssueCollectionQuery({
  currentView,
  filters,
  table,
  search,
  project,
  scope,
  isTriage,
  layout,
  grouping,
  unavailableLabels,
  invalidTreeValues,
}: QueryOptions) {
  const issues = useIssuesStore();
  const projectStore = useProjectsStore();
  const labelsStore = useLabelsStore();
  const {
    statusFilter,
    priorityFilter,
    projectFilter,
    labelFilter,
    excludedStatuses,
    excludedPriorities,
    excludedProjects,
    projectStatuses,
    excludedProjectStatuses,
    projectPriorities,
    excludedProjectPriorities,
    projectLabels,
    excludedProjectLabels,
    emptyProjectLabels,
    excludedEmptyProjectLabels,
    projectLeads,
    excludedProjectLeads,
    emptyProjectLead,
    excludedEmptyProjectLead,
    projectMilestones,
    excludedProjectMilestones,
    emptyProjectMilestones,
    excludedEmptyProjectMilestones,
    milestones,
    excludedMilestones,
    emptyMilestone,
    excludedEmptyMilestone,
    excludedLabels,
    emptyLabels,
    excludedEmptyLabels,
    dueFilter,
    advancedGroups,
  } = filters;
  const { localTable, tableSortField, tableSortDirection } = table;

  // The browser preview filters its local records; native storage applies the same query before paging.
  const localVisibleIssues = computed(() =>
    unavailableLabels.value.length || invalidTreeValues.value.length
      ? []
      : issues.items.filter((issue) => {
          if (isTriage.value ? !issue.triage : issue.triage) return false;
          if (currentView.value?.includeSubIssues === false && issue.parentId) return false;

          if (
            !isTriage.value &&
            scope.value === 'active' &&
            ['Completed', 'Canceled'].includes(issues.categoryFor(issue.status))
          )
            return false;

          if (
            project.value
              ? issue.project !== project.value
              : projectFilter.value.length > 0 && !projectFilter.value.includes(issue.project)
          )
            return false;

          if (statusFilter.value.length && !statusFilter.value.includes(issue.status)) return false;
          if (excludedStatuses.value.includes(issue.status)) return false;

          if (priorityFilter.value.length && !priorityFilter.value.includes(issue.priority))
            return false;
          if (excludedPriorities.value.includes(issue.priority)) return false;

          if (!project.value && excludedProjects.value.includes(issue.project)) return false;

          const issueProject = projectStore.items.find((item) => item.name === issue.project);
          if (
            projectStatuses.value.length &&
            (!issueProject || !projectStatuses.value.includes(issueProject.status))
          )
            return false;
          if (issueProject && excludedProjectStatuses.value.includes(issueProject.status))
            return false;
          if (
            projectPriorities.value.length &&
            (!issueProject || !projectPriorities.value.includes(issueProject.priority))
          )
            return false;
          if (issueProject && excludedProjectPriorities.value.includes(issueProject.priority))
            return false;

          const labelsOnProject = issueProject?.labels ?? [];
          if (
            (projectLabels.value.length > 0 || emptyProjectLabels.value) &&
            !projectLabels.value.some((label) => labelsOnProject.includes(label)) &&
            !(emptyProjectLabels.value && labelsOnProject.length === 0)
          )
            return false;
          if (excludedProjectLabels.value.some((label) => labelsOnProject.includes(label)))
            return false;
          if (excludedEmptyProjectLabels.value && labelsOnProject.length === 0) return false;

          const lead = issueProject?.leadActorId ?? null;
          if (
            (projectLeads.value.length > 0 || emptyProjectLead.value) &&
            !(lead && projectLeads.value.includes(lead)) &&
            !(emptyProjectLead.value && !lead)
          )
            return false;
          if (lead && excludedProjectLeads.value.includes(lead)) return false;
          if (excludedEmptyProjectLead.value && !lead) return false;

          const projectMilestonesOnProject = issueProject?.milestones ?? [];
          if (
            (projectMilestones.value.length > 0 || emptyProjectMilestones.value) &&
            !projectMilestonesOnProject.some((milestone) =>
              projectMilestones.value.includes(milestone.title),
            ) &&
            !(emptyProjectMilestones.value && projectMilestonesOnProject.length === 0)
          )
            return false;
          if (
            projectMilestonesOnProject.some((milestone) =>
              excludedProjectMilestones.value.includes(milestone.title),
            )
          )
            return false;
          if (excludedEmptyProjectMilestones.value && projectMilestonesOnProject.length === 0)
            return false;

          const milestoneId = issue.milestoneId ?? null;
          if (
            (milestones.value.length > 0 || emptyMilestone.value) &&
            !(milestoneId && milestones.value.includes(milestoneId)) &&
            !(emptyMilestone.value && !milestoneId)
          )
            return false;
          if (milestoneId && excludedMilestones.value.includes(milestoneId)) return false;
          if (excludedEmptyMilestone.value && !milestoneId) return false;

          if (
            (labelFilter.value.length > 0 || emptyLabels.value) &&
            !labelFilter.value.some((label) => issue.labels.includes(label)) &&
            !(emptyLabels.value && issue.labels.length === 0)
          )
            return false;
          if (excludedLabels.value.some((label) => issue.labels.includes(label))) return false;
          if (excludedEmptyLabels.value && issue.labels.length === 0) return false;

          if (dueFilter.value === 'has' && !issue.due) return false;

          if (dueFilter.value === 'none' && issue.due) return false;
          if (!matchesAdvancedFilters(issue, advancedGroups.value, issues.items, issues.relations))
            return false;
          if (
            !matchesIssueFilterTree(issue, currentView.value?.filterTree, {
              projectId: projectStore.items.find((project) => project.name === issue.project)?.id,
              statusCategory: issues.categoryFor(issue.status),
              labelIds: labelsStore.items
                .filter((label) => issue.labels.includes(label.name))
                .map((label) => label.id),
            })
          )
            return false;

          const query = search.value.trim().toLowerCase();

          return (
            !query ||
            `${issue.id} ${issue.title} ${issue.project} ${issue.labels.join(' ')}`
              .toLowerCase()
              .includes(query)
          );
        }),
  );

  const nativeQuery = computed<IssueQuery>(() =>
    currentView.value && !project.value
      ? { ...savedViewQuery(currentView.value), text: search.value.trim() }
      : {
          scope: scope.value,
          triage: !!isTriage.value,
          statuses: statusFilter.value,
          excludedStatuses: excludedStatuses.value,
          priorities: priorityFilter.value,
          excludedPriorities: excludedPriorities.value,
          projects: project.value ? [project.value] : projectFilter.value,
          excludedProjects: project.value ? [] : excludedProjects.value,
          projectStatuses: projectStatuses.value,
          excludedProjectStatuses: excludedProjectStatuses.value,
          projectPriorities: projectPriorities.value,
          excludedProjectPriorities: excludedProjectPriorities.value,
          projectLabels: projectLabels.value,
          excludedProjectLabels: excludedProjectLabels.value,
          emptyProjectLabels: emptyProjectLabels.value,
          excludedEmptyProjectLabels: excludedEmptyProjectLabels.value,
          projectLeads: projectLeads.value,
          excludedProjectLeads: excludedProjectLeads.value,
          emptyProjectLead: emptyProjectLead.value,
          excludedEmptyProjectLead: excludedEmptyProjectLead.value,
          projectMilestones: projectMilestones.value,
          excludedProjectMilestones: excludedProjectMilestones.value,
          emptyProjectMilestones: emptyProjectMilestones.value,
          excludedEmptyProjectMilestones: excludedEmptyProjectMilestones.value,
          milestones: milestones.value,
          excludedMilestones: excludedMilestones.value,
          emptyMilestone: emptyMilestone.value,
          excludedEmptyMilestone: excludedEmptyMilestone.value,
          labels: labelFilter.value,
          excludedLabels: excludedLabels.value,
          emptyLabels: emptyLabels.value,
          excludedEmptyLabels: excludedEmptyLabels.value,
          due: dueFilter.value,
          text: search.value.trim(),
          sortField: localTable.value.sortField,
          sortDirection: localTable.value.sortDirection,
          advancedGroups: advancedGroups.value,
        },
  );

  const {
    nativeIssueIds,
    nativeTotalCount,
    nativeIssueRecords,
    nativeLoadedLimit,
    nativeLoadingMore,
    nativeQueryError,
    nativeBoardLoading,
    projectGroupNames,
    nativeBoardCounts,
    refreshNativeQuery,
    loadMoreNativeBoard,
    loadMoreNativeIssues,
  } = useNativeIssuePages(nativeQuery, layout, grouping, unavailableLabels, invalidTreeValues);

  const visibleIssues = computed(() => {
    if (!usesNativeStorage || nativeIssueIds.value === null) {
      const items = [...localVisibleIssues.value];
      const field = tableSortField.value;
      if (field && field !== 'position') {
        const direction = tableSortDirection.value === 'desc' ? -1 : 1;
        items.sort((left, right) => {
          const key = (issue: Issue) => {
            if (field === 'dueDate') return issue.due ?? '9999-12-31';
            if (field === 'updatedAt') return issue.updated;

            return issue[field];
          };

          return (
            direction * String(key(left)).localeCompare(String(key(right))) ||
            left.id.localeCompare(right.id)
          );
        });
      }

      return items;
    }

    const byId = new Map(nativeIssueRecords.value.map((issue) => [issue.id, issue]));
    for (const issue of issues.items) if (byId.has(issue.id)) byId.set(issue.id, issue);

    return nativeIssueIds.value
      .slice(0, nativeLoadedLimit.value)
      .map((id) => byId.get(id))
      .filter((issue): issue is Issue => issue !== undefined);
  });

  return {
    nativeIssueIds,
    nativeTotalCount,
    nativeIssueRecords,
    nativeLoadedLimit,
    nativeLoadingMore,
    nativeQueryError,
    nativeBoardLoading,
    projectGroupNames,
    nativeBoardCounts,
    refreshNativeQuery,
    loadMoreNativeBoard,
    loadMoreNativeIssues,
    visibleIssues,
  };
}
