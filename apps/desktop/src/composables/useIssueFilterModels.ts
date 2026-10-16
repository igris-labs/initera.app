import { computed, ref, shallowRef } from 'vue';
import type { ComputedRef, Ref, WritableComputedRef } from 'vue';
import { useIssuesStore } from '../stores/issues';
import { useViewsStore } from '../stores/views';
import type { Priority, Project, Status } from '../types/issue';
import type { SavedView } from '../types/view';

export function useIssueFilterModels(
  currentView: ComputedRef<SavedView | undefined>,
  scope: WritableComputedRef<'active' | 'all'>,
  search: Ref<string>,
) {
  const issues = useIssuesStore();
  const views = useViewsStore();

  const localStatusFilter = ref<Status[]>([]);
  const localPriorityFilter = ref<Priority[]>([]);
  const localProjectFilter = ref<Project[]>([]);
  const localLabelFilter = ref<string[]>([]);

  // Most filters share one rule: use the saved view when present, otherwise local state.
  function viewField<K extends keyof SavedView>(key: K, initial: NonNullable<SavedView[K]>) {
    const local = shallowRef(initial);
    return computed<NonNullable<SavedView[K]>>({
      get: () => (currentView.value?.[key] ?? local.value) as NonNullable<SavedView[K]>,
      set: (value) => {
        if (currentView.value) views.update(currentView.value.id, { [key]: value });
        else local.value = value;
      },
    });
  }

  const statusFilter = computed<Status[]>({
    get: () =>
      currentView.value
        ? (currentView.value.statuses ??
          (currentView.value.status === 'All statuses' ? [] : [currentView.value.status]))
        : localStatusFilter.value,
    set: (value) => {
      if (value.some((status) => ['Completed', 'Canceled'].includes(issues.categoryFor(status))))
        scope.value = 'all';
      if (currentView.value)
        views.update(currentView.value.id, {
          statuses: value,
          status: 'All statuses',
        });
      else localStatusFilter.value = value;
    },
  });

  const priorityFilter = computed<Priority[]>({
    get: () =>
      currentView.value
        ? (currentView.value.priorities ??
          (currentView.value.priority ? [currentView.value.priority] : []))
        : localPriorityFilter.value,
    set: (value) =>
      currentView.value
        ? views.update(currentView.value.id, {
            priorities: value,
            priority: undefined,
          })
        : (localPriorityFilter.value = value),
  });

  const projectFilter = computed<Project[]>({
    get: () =>
      currentView.value
        ? (currentView.value.projects ??
          (currentView.value.project ? [currentView.value.project] : []))
        : localProjectFilter.value,
    set: (value) =>
      currentView.value
        ? views.update(currentView.value.id, {
            projects: value,
            project: undefined,
          })
        : (localProjectFilter.value = value),
  });

  const labelFilter = computed<string[]>({
    get: () =>
      currentView.value
        ? (currentView.value.labels ?? (currentView.value.label ? [currentView.value.label] : []))
        : localLabelFilter.value,
    set: (value) =>
      currentView.value
        ? views.update(currentView.value.id, { labels: value, label: undefined })
        : (localLabelFilter.value = value),
  });

  const excludedStatuses = viewField('excludedStatuses', []);
  const excludedPriorities = viewField('excludedPriorities', []);
  const excludedProjects = viewField('excludedProjects', []);
  const projectStatuses = viewField('projectStatuses', []);
  const excludedProjectStatuses = viewField('excludedProjectStatuses', []);
  const projectPriorities = viewField('projectPriorities', []);
  const excludedProjectPriorities = viewField('excludedProjectPriorities', []);
  const projectLabels = viewField('projectLabels', []);
  const excludedProjectLabels = viewField('excludedProjectLabels', []);
  const emptyProjectLabels = viewField('emptyProjectLabels', false);
  const excludedEmptyProjectLabels = viewField('excludedEmptyProjectLabels', false);
  const projectLeads = viewField('projectLeads', []);
  const excludedProjectLeads = viewField('excludedProjectLeads', []);
  const emptyProjectLead = viewField('emptyProjectLead', false);
  const excludedEmptyProjectLead = viewField('excludedEmptyProjectLead', false);
  const projectMilestones = viewField('projectMilestones', []);
  const excludedProjectMilestones = viewField('excludedProjectMilestones', []);
  const emptyProjectMilestones = viewField('emptyProjectMilestones', false);
  const excludedEmptyProjectMilestones = viewField('excludedEmptyProjectMilestones', false);
  const milestones = viewField('milestones', []);
  const excludedMilestones = viewField('excludedMilestones', []);
  const emptyMilestone = viewField('emptyMilestone', false);
  const excludedEmptyMilestone = viewField('excludedEmptyMilestone', false);
  const excludedLabels = viewField('excludedLabels', []);
  const emptyLabels = viewField('emptyLabels', false);
  const excludedEmptyLabels = viewField('excludedEmptyLabels', false);
  const dueFilter = viewField('due', 'any');
  const advancedGroups = viewField('advancedGroups', []);

  function clearFilters() {
    search.value = '';

    if (currentView.value) {
      void views.update(currentView.value.id, {
        status: 'All statuses',
        statuses: [],
        excludedStatuses: [],
        priority: undefined,
        priorities: [],
        excludedPriorities: [],
        project: undefined,
        projects: [],
        excludedProjects: [],
        projectStatuses: [],
        excludedProjectStatuses: [],
        projectPriorities: [],
        excludedProjectPriorities: [],
        projectLabels: [],
        excludedProjectLabels: [],
        emptyProjectLabels: false,
        excludedEmptyProjectLabels: false,
        projectLeads: [],
        excludedProjectLeads: [],
        emptyProjectLead: false,
        excludedEmptyProjectLead: false,
        projectMilestones: [],
        excludedProjectMilestones: [],
        emptyProjectMilestones: false,
        excludedEmptyProjectMilestones: false,
        milestones: [],
        excludedMilestones: [],
        emptyMilestone: false,
        excludedEmptyMilestone: false,
        label: undefined,
        labels: [],
        excludedLabels: [],
        emptyLabels: false,
        excludedEmptyLabels: false,
        due: 'any',
        advancedGroups: [],
      });
      return;
    }

    statusFilter.value = [];
    excludedStatuses.value = [];
    priorityFilter.value = [];
    excludedPriorities.value = [];
    projectFilter.value = [];
    excludedProjects.value = [];
    projectStatuses.value = [];
    excludedProjectStatuses.value = [];
    projectPriorities.value = [];
    excludedProjectPriorities.value = [];
    projectLabels.value = [];
    excludedProjectLabels.value = [];
    emptyProjectLabels.value = false;
    excludedEmptyProjectLabels.value = false;
    projectLeads.value = [];
    excludedProjectLeads.value = [];
    emptyProjectLead.value = false;
    excludedEmptyProjectLead.value = false;
    projectMilestones.value = [];
    excludedProjectMilestones.value = [];
    emptyProjectMilestones.value = false;
    excludedEmptyProjectMilestones.value = false;
    milestones.value = [];
    excludedMilestones.value = [];
    emptyMilestone.value = false;
    excludedEmptyMilestone.value = false;
    labelFilter.value = [];
    excludedLabels.value = [];
    emptyLabels.value = false;
    excludedEmptyLabels.value = false;
    dueFilter.value = 'any';
    advancedGroups.value = [];
  }

  return {
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
    clearFilters,
  };
}
