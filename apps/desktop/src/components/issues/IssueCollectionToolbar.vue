<script setup lang="ts">
import { storeToRefs } from 'pinia';
import type { useIssueFilterModels } from '../../composables/useIssueFilterModels';
import type { useIssueFilterOptions } from '../../composables/useIssueFilterOptions';
import { usePreferencesStore } from '../../stores/preferences';
import { useIssuesStore } from '../../stores/issues';
import { useProjectsStore } from '../../stores/projects';
import { useViewsStore } from '../../stores/views';
import { useWorkspaceStore } from '../../stores/workspace';
import type { Issue, Project } from '../../types/issue';
import type { SavedView } from '../../types/view';
import ModeControl from '../common/ModeControl.vue';
import ShortcutTooltip from '../common/ShortcutTooltip.vue';
import { Button } from '../ui/button';
import { Icon } from '../ui/icon';
import IssueDisplayPopover from './IssueDisplayPopover.vue';
import IssueFilterPopover from './IssueFilterPopover.vue';

interface Props {
  currentView?: SavedView;
  project?: Project;
  isTriage: boolean;
  visibleIssues: Issue[];
  triageSelection: Issue[];
  filters: ReturnType<typeof useIssueFilterModels>;
  options: ReturnType<typeof useIssueFilterOptions>;
}

const { currentView, project, isTriage, visibleIssues, triageSelection, filters, options } =
  defineProps<Props>();
const emit = defineEmits<{ decision: [value: 'accept' | 'decline']; editView: [] }>();

const {
  advancedGroups,
  dueFilter,
  emptyLabels,
  emptyMilestone,
  emptyProjectLabels,
  emptyProjectLead,
  emptyProjectMilestones,
  excludedEmptyLabels,
  excludedEmptyMilestone,
  excludedEmptyProjectLabels,
  excludedEmptyProjectLead,
  excludedEmptyProjectMilestones,
  excludedLabels,
  excludedMilestones,
  excludedPriorities,
  excludedProjectLabels,
  excludedProjectLeads,
  excludedProjectMilestones,
  excludedProjectPriorities,
  excludedProjectStatuses,
  excludedProjects,
  excludedStatuses,
  labelFilter,
  milestones,
  priorityFilter,
  projectFilter,
  projectLabels,
  projectLeads,
  projectMilestones,
  projectPriorities,
  projectStatuses,
  statusFilter,
} = filters;
const {
  activeFilterCount,
  labelOptions,
  milestoneOptions,
  projectLabelOptions,
  projectMilestoneOptions,
} = options;
const scope = defineModel<'active' | 'all'>('scope', { required: true });
const layout = defineModel<'list' | 'board'>('layout', { required: true });
const grouping = defineModel<'none' | 'status' | 'priority' | 'project'>('grouping', {
  required: true,
});
const listGrouped = defineModel<boolean>('listGrouped', { required: true });
const search = defineModel<string>('search', { required: true });
const filtersOpen = defineModel<boolean>('filtersOpen', { required: true });
const displayOpen = defineModel<boolean>('displayOpen', { required: true });
const overviewOpen = defineModel<boolean>('overviewOpen', { required: true });
const selectedTriageIds = defineModel<string[]>('selectedTriageIds', { required: true });

const issues = useIssuesStore();
const views = useViewsStore();
const workspace = useWorkspaceStore();
const projectStore = useProjectsStore();
const { showIssuePriority, showIssueProject } = storeToRefs(usePreferencesStore());
</script>

<template>
  <div class="flex min-h-11 flex-wrap items-center gap-2 px-3 py-1.5">
    <ModeControl
      v-if="!isTriage"
      v-model="scope"
      label="Issue scope"
      :items="[
        { label: 'Active', value: 'active' },
        { label: 'All', value: 'all' },
      ]"
    />
    <div v-else class="flex h-7 items-center gap-2 text-xs font-medium text-foreground-muted">
      <Icon name="i-tabler-inbox" class="size-4" />Pending review
    </div>
    <div v-if="isTriage && visibleIssues.length" class="flex items-center gap-1">
      <Button
        :label="
          selectedTriageIds.length ? 'Clear selection' : `Select ${visibleIssues.length} loaded`
        "
        variant="ghost"
        size="xs"
        @click="
          selectedTriageIds = selectedTriageIds.length
            ? []
            : visibleIssues.slice(0, 100).map((issue) => issue.id)
        "
      />
      <template v-if="triageSelection.length">
        <Button label="Accept" variant="soft" size="xs" @click="emit('decision', 'accept')" />
        <Button
          label="Decline"
          variant="soft"
          color="error"
          size="xs"
          @click="emit('decision', 'decline')"
        />
      </template>
    </div>
    <div v-if="currentView" class="flex items-center gap-1">
      <Button
        :icon="currentView.favorite ? 'i-tabler-star-filled' : 'i-tabler-star'"
        :aria-label="currentView.favorite ? 'Remove favorite' : 'Favorite view'"
        variant="ghost"
        color="info"
        @click="
          views.update(currentView.id, {
            favorite: !currentView.favorite,
          })
        "
      />
      <Button
        icon="i-tabler-pencil"
        aria-label="Edit view"
        variant="ghost"
        @click="emit('editView')"
      />
    </div>
    <div class="ml-auto flex items-center gap-1">
      <IssueFilterPopover
        v-model:open="filtersOpen"
        v-model:search="search"
        v-model:status="statusFilter"
        v-model:excluded-statuses="excludedStatuses"
        v-model:priority="priorityFilter"
        v-model:excluded-priorities="excludedPriorities"
        v-model:label="labelFilter"
        v-model:excluded-labels="excludedLabels"
        v-model:empty-labels="emptyLabels"
        v-model:excluded-empty-labels="excludedEmptyLabels"
        v-model:project="projectFilter"
        v-model:excluded-projects="excludedProjects"
        v-model:project-statuses="projectStatuses"
        v-model:excluded-project-statuses="excludedProjectStatuses"
        v-model:project-priorities="projectPriorities"
        v-model:excluded-project-priorities="excludedProjectPriorities"
        v-model:project-labels="projectLabels"
        v-model:excluded-project-labels="excludedProjectLabels"
        v-model:empty-project-labels="emptyProjectLabels"
        v-model:excluded-empty-project-labels="excludedEmptyProjectLabels"
        v-model:project-leads="projectLeads"
        v-model:excluded-project-leads="excludedProjectLeads"
        v-model:empty-project-lead="emptyProjectLead"
        v-model:excluded-empty-project-lead="excludedEmptyProjectLead"
        v-model:project-milestones="projectMilestones"
        v-model:excluded-project-milestones="excludedProjectMilestones"
        v-model:empty-project-milestones="emptyProjectMilestones"
        v-model:excluded-empty-project-milestones="excludedEmptyProjectMilestones"
        v-model:milestones="milestones"
        v-model:excluded-milestones="excludedMilestones"
        v-model:empty-milestone="emptyMilestone"
        v-model:excluded-empty-milestone="excludedEmptyMilestone"
        v-model:due="dueFilter"
        v-model:advanced-groups="advancedGroups"
        :statuses="issues.visibleStatusGroups"
        :labels="labelOptions"
        :available-project-labels="projectLabelOptions"
        :available-project-milestones="projectMilestoneOptions"
        :available-milestones="milestoneOptions"
        :actor-id="workspace.actorId"
        :actor-name="workspace.userName"
        :projects="projectStore.names"
        :show-project="!project && !isTriage"
        :active-count="activeFilterCount"
      />
      <IssueDisplayPopover
        v-model:open="displayOpen"
        v-model:layout="layout"
        v-model:board-group-by="grouping"
        v-model:grouped="listGrouped"
        v-model:show-priority="showIssuePriority"
        v-model:show-project="showIssueProject"
        :allow-additional-grouping="!!currentView"
      />
      <ShortcutTooltip label="Toggle issue overview">
        <Button
          data-issue-overview-trigger
          icon="i-tabler-layout-sidebar-right"
          aria-label="Toggle issue overview"
          variant="ghost"
          size="xs"
          :aria-pressed="overviewOpen"
          @click="overviewOpen = !overviewOpen"
        />
      </ShortcutTooltip>
    </div>
  </div>
</template>
