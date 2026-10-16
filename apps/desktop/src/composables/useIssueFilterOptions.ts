import { computed, ref, watch } from 'vue';
import type { ComputedRef, Ref } from 'vue';
import { useLabelsStore } from '../stores/labels';
import { useProjectsStore } from '../stores/projects';
import { useIssuesStore } from '../stores/issues';
import { useWorkspaceStore } from '../stores/workspace';
import { unavailableLabelFilters } from '../lib/label-filter-availability';
import { usesNativeStorage } from '../lib/workspace-storage';
import type { IssueFilterTree, Project } from '../types/issue';
import type { SavedView } from '../types/view';
import type { useIssueFilterModels } from './useIssueFilterModels';

export function useIssueFilterOptions(
  filters: ReturnType<typeof useIssueFilterModels>,
  currentView: ComputedRef<SavedView | undefined>,
  project: ComputedRef<Project | undefined>,
  search: Ref<string>,
) {
  const labelsStore = useLabelsStore();
  const projectStore = useProjectsStore();
  const issues = useIssuesStore();
  const workspace = useWorkspaceStore();
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

  const unavailableLabels = computed(() => {
    if (!labelsStore.ready) return [];
    return unavailableLabelFilters(
      {
        labels: labelFilter.value,
        excludedLabels: excludedLabels.value,
        projectLabels: projectLabels.value,
        excludedProjectLabels: excludedProjectLabels.value,
      },
      labelsStore.items,
    );
  });

  const missingProjectIds = ref<string[]>([]);
  let projectValidationRevision = 0;

  watch(
    [() => currentView.value?.filterTree, () => projectStore.ready, () => workspace.workspaceId],
    async ([tree, ready]) => {
      // A saved view may reference a project outside the current index page. Resolve it by ID
      // before treating it as deleted, and discard answers from an older workspace or filter.
      const revision = ++projectValidationRevision;
      missingProjectIds.value = [];
      if (!usesNativeStorage || !ready || !tree) return;

      const ids = new Set<string>();
      const visit = (node: IssueFilterTree) => {
        if ('children' in node) return node.children.forEach(visit);
        if (node.field !== 'projectId') return;

        for (const value of Array.isArray(node.value) ? node.value : [node.value]) {
          if (
            typeof value === 'string' &&
            value &&
            !projectStore.items.some((project) => project.id === value)
          )
            ids.add(value);
        }
      };

      visit(tree);
      const results = await Promise.allSettled([...ids].map((id) => projectStore.refreshOne(id)));
      if (revision !== projectValidationRevision) return;
      missingProjectIds.value = [...ids].filter(
        (_, index) => results[index]?.status === 'rejected',
      );
    },
    { immediate: true },
  );

  const invalidTreeValues = computed(() => {
    const invalid: string[] = [];
    if (!labelsStore.ready || !currentView.value?.filterTree) return invalid;

    const visit = (node: IssueFilterTree) => {
      if ('children' in node) return node.children.forEach(visit);
      const values = Array.isArray(node.value) ? node.value : [node.value];
      if (node.field === 'labelIds') {
        for (const value of values) {
          if (!labelsStore.assignable.some((label) => label.id === value))
            invalid.push(`Label ${value}`);
        }
      }

      if (node.field === 'projectId') {
        for (const value of values) {
          if (
            value &&
            (usesNativeStorage
              ? missingProjectIds.value.includes(String(value))
              : !projectStore.items.some((project) => project.id === value))
          )
            invalid.push(`Project ${value}`);
        }
      }
    };

    visit(currentView.value.filterTree);
    return invalid;
  });

  const labelOptions = computed(() =>
    [
      ...new Set([
        ...labelsStore.items.filter((label) => !label.deletedAt).map((label) => label.name),
        ...issues.items.flatMap((issue) => issue.labels),
      ]),
    ].sort(),
  );

  const projectLabelOptions = computed(() =>
    [...new Set(projectStore.items.flatMap((project) => project.labels ?? []))].sort(),
  );

  const projectMilestoneOptions = computed(() =>
    [
      ...new Set(
        projectStore.items.flatMap((project) =>
          (project.milestones ?? []).map((milestone) => milestone.title),
        ),
      ),
    ].sort(),
  );

  const milestoneOptions = computed(() =>
    projectStore.items
      .filter((item) => !project.value || item.name === project.value)
      .flatMap((project) =>
        (project.milestones ?? []).map((milestone) => ({
          id: milestone.id,
          label: `${project.name} · ${milestone.title}`,
        })),
      ),
  );

  function milestoneLabel(id: string) {
    return (
      milestoneOptions.value.find((milestone) => milestone.id === id)?.label ?? 'Removed milestone'
    );
  }

  const activeFilterCount = computed(
    () =>
      Number(!!search.value.trim()) +
      Number(statusFilter.value.length > 0 || excludedStatuses.value.length > 0) +
      Number(priorityFilter.value.length > 0 || excludedPriorities.value.length > 0) +
      Number(
        labelFilter.value.length > 0 ||
          excludedLabels.value.length > 0 ||
          emptyLabels.value ||
          excludedEmptyLabels.value,
      ) +
      Number(
        (projectFilter.value.length > 0 || excludedProjects.value.length > 0) && !project.value,
      ) +
      Number(projectStatuses.value.length > 0 || excludedProjectStatuses.value.length > 0) +
      Number(projectPriorities.value.length > 0 || excludedProjectPriorities.value.length > 0) +
      Number(
        projectLabels.value.length > 0 ||
          excludedProjectLabels.value.length > 0 ||
          emptyProjectLabels.value ||
          excludedEmptyProjectLabels.value,
      ) +
      Number(
        projectLeads.value.length > 0 ||
          excludedProjectLeads.value.length > 0 ||
          emptyProjectLead.value ||
          excludedEmptyProjectLead.value,
      ) +
      Number(
        projectMilestones.value.length > 0 ||
          excludedProjectMilestones.value.length > 0 ||
          emptyProjectMilestones.value ||
          excludedEmptyProjectMilestones.value,
      ) +
      Number(
        milestones.value.length > 0 ||
          excludedMilestones.value.length > 0 ||
          emptyMilestone.value ||
          excludedEmptyMilestone.value,
      ) +
      Number(dueFilter.value !== 'any') +
      advancedGroups.value.reduce((count, group) => count + group.rules.length, 0),
  );

  type PlanningChipKind =
    | 'lead'
    | 'excludedLead'
    | 'noLead'
    | 'hasLead'
    | 'milestone'
    | 'excludedMilestone'
    | 'noMilestones'
    | 'hasMilestones'
    | 'assignedMilestone'
    | 'excludedAssignedMilestone'
    | 'noAssignedMilestone'
    | 'hasAssignedMilestone';

  interface PlanningChip {
    kind: PlanningChipKind;
    value?: string;
    label: string;
  }

  const planningChips = computed<PlanningChip[]>(() => {
    const leadName = workspace.userName || 'You';

    return [
      ...projectLeads.value.map((value) => ({
        kind: 'lead' as const,
        value,
        label: `Project lead: ${value === workspace.actorId ? leadName : value}`,
      })),
      ...excludedProjectLeads.value.map((value) => ({
        kind: 'excludedLead' as const,
        value,
        label: `Not project lead: ${value === workspace.actorId ? leadName : value}`,
      })),
      ...(emptyProjectLead.value ? [{ kind: 'noLead' as const, label: 'No project lead' }] : []),
      ...(excludedEmptyProjectLead.value
        ? [{ kind: 'hasLead' as const, label: 'Has project lead' }]
        : []),
      ...projectMilestones.value.map((value) => ({
        kind: 'milestone' as const,
        value,
        label: `Project milestone: ${value}`,
      })),
      ...excludedProjectMilestones.value.map((value) => ({
        kind: 'excludedMilestone' as const,
        value,
        label: `Not project milestone: ${value}`,
      })),
      ...(emptyProjectMilestones.value
        ? [{ kind: 'noMilestones' as const, label: 'No project milestones' }]
        : []),
      ...(excludedEmptyProjectMilestones.value
        ? [{ kind: 'hasMilestones' as const, label: 'Has project milestones' }]
        : []),
      ...milestones.value.map((value) => ({
        kind: 'assignedMilestone' as const,
        value,
        label: `Milestone: ${milestoneLabel(value)}`,
      })),
      ...excludedMilestones.value.map((value) => ({
        kind: 'excludedAssignedMilestone' as const,
        value,
        label: `Not milestone: ${milestoneLabel(value)}`,
      })),
      ...(emptyMilestone.value
        ? [{ kind: 'noAssignedMilestone' as const, label: 'No milestone' }]
        : []),
      ...(excludedEmptyMilestone.value
        ? [{ kind: 'hasAssignedMilestone' as const, label: 'Has milestone' }]
        : []),
    ];
  });

  function removePlanningChip(chip: PlanningChip) {
    switch (chip.kind) {
      case 'lead':
        projectLeads.value = projectLeads.value.filter((item) => item !== chip.value);
        break;
      case 'excludedLead':
        excludedProjectLeads.value = excludedProjectLeads.value.filter(
          (item) => item !== chip.value,
        );
        break;
      case 'noLead':
        emptyProjectLead.value = false;
        break;
      case 'hasLead':
        excludedEmptyProjectLead.value = false;
        break;
      case 'milestone':
        projectMilestones.value = projectMilestones.value.filter((item) => item !== chip.value);
        break;
      case 'excludedMilestone':
        excludedProjectMilestones.value = excludedProjectMilestones.value.filter(
          (item) => item !== chip.value,
        );
        break;
      case 'noMilestones':
        emptyProjectMilestones.value = false;
        break;
      case 'hasMilestones':
        excludedEmptyProjectMilestones.value = false;
        break;
      case 'assignedMilestone':
        milestones.value = milestones.value.filter((item) => item !== chip.value);
        break;
      case 'excludedAssignedMilestone':
        excludedMilestones.value = excludedMilestones.value.filter((item) => item !== chip.value);
        break;
      case 'noAssignedMilestone':
        emptyMilestone.value = false;
        break;
      case 'hasAssignedMilestone':
        excludedEmptyMilestone.value = false;
        break;
    }
  }

  return {
    unavailableLabels,
    invalidTreeValues,
    labelOptions,
    projectLabelOptions,
    projectMilestoneOptions,
    milestoneOptions,
    activeFilterCount,
    planningChips,
    removePlanningChip,
  };
}
