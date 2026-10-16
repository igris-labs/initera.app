<script setup lang="ts">
import type { Project } from '../../types/issue';
import type { useIssueFilterModels } from '../../composables/useIssueFilterModels';
import type { useIssueFilterOptions } from '../../composables/useIssueFilterOptions';
import { Icon } from '../ui/icon';

interface Props {
  filters: ReturnType<typeof useIssueFilterModels>;
  options: ReturnType<typeof useIssueFilterOptions>;
  project?: Project;
}

const { filters, options, project } = defineProps<Props>();
const emit = defineEmits<{ clear: [] }>();
const search = defineModel<string>('search', { required: true });
const {
  statusFilter,
  excludedStatuses,
  priorityFilter,
  excludedPriorities,
  labelFilter,
  excludedLabels,
  emptyLabels,
  excludedEmptyLabels,
  projectFilter,
  excludedProjects,
  dueFilter,
  advancedGroups,
  projectStatuses,
  excludedProjectStatuses,
  projectPriorities,
  excludedProjectPriorities,
  projectLabels,
  excludedProjectLabels,
  emptyProjectLabels,
  excludedEmptyProjectLabels,
} = filters;
const { activeFilterCount, planningChips, removePlanningChip } = options;
</script>

<template>
  <div
    v-if="activeFilterCount"
    class="flex flex-wrap items-center gap-1 px-3 pb-2"
    aria-label="Active filters"
  >
    <button
      v-if="search.trim()"
      type="button"
      class="filter-chip"
      aria-label="Clear text search"
      @click="search = ''"
    >
      Text: {{ search }} <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-for="value in statusFilter"
      :key="`status-${value}`"
      type="button"
      class="filter-chip"
      :aria-label="`Clear ${value} status filter`"
      @click="statusFilter = statusFilter.filter((item) => item !== value)"
    >
      Status: {{ value }} <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-for="value in excludedStatuses"
      :key="`excluded-status-${value}`"
      type="button"
      class="filter-chip"
      :aria-label="`Clear excluded ${value} status`"
      @click="excludedStatuses = excludedStatuses.filter((item) => item !== value)"
    >
      Not status: {{ value }} <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-for="value in priorityFilter"
      :key="`priority-${value}`"
      type="button"
      class="filter-chip"
      :aria-label="`Clear ${value} priority filter`"
      @click="priorityFilter = priorityFilter.filter((item) => item !== value)"
    >
      Priority: {{ value }}
      <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-for="value in excludedPriorities"
      :key="`excluded-priority-${value}`"
      type="button"
      class="filter-chip"
      :aria-label="`Clear excluded ${value} priority`"
      @click="excludedPriorities = excludedPriorities.filter((item) => item !== value)"
    >
      Not priority: {{ value }} <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-for="value in labelFilter"
      :key="`label-${value}`"
      type="button"
      class="filter-chip"
      :aria-label="`Clear ${value} label filter`"
      @click="labelFilter = labelFilter.filter((item) => item !== value)"
    >
      Label: {{ value }} <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-for="value in excludedLabels"
      :key="`excluded-label-${value}`"
      type="button"
      class="filter-chip"
      :aria-label="`Clear excluded ${value} label`"
      @click="excludedLabels = excludedLabels.filter((item) => item !== value)"
    >
      Not label: {{ value }} <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-if="emptyLabels"
      type="button"
      class="filter-chip"
      aria-label="Clear no labels filter"
      @click="emptyLabels = false"
    >
      No labels <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-if="excludedEmptyLabels"
      type="button"
      class="filter-chip"
      aria-label="Clear has labels filter"
      @click="excludedEmptyLabels = false"
    >
      Has labels <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-for="value in project ? [] : projectFilter"
      :key="`project-${value}`"
      type="button"
      class="filter-chip"
      :aria-label="`Clear ${value} project filter`"
      @click="projectFilter = projectFilter.filter((item) => item !== value)"
    >
      Project: {{ value }}
      <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-for="value in project ? [] : excludedProjects"
      :key="`excluded-project-${value}`"
      type="button"
      class="filter-chip"
      :aria-label="`Clear excluded ${value} project`"
      @click="excludedProjects = excludedProjects.filter((item) => item !== value)"
    >
      Not project: {{ value }} <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-if="dueFilter !== 'any'"
      type="button"
      class="filter-chip"
      aria-label="Clear due date filter"
      @click="dueFilter = 'any'"
    >
      {{ dueFilter === 'has' ? 'Has due date' : 'No due date' }}
      <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-for="(group, index) in advancedGroups"
      :key="`advanced-${index}`"
      type="button"
      class="filter-chip"
      :aria-label="`Clear advanced filter group ${index + 1}`"
      @click="advancedGroups = advancedGroups.filter((_, position) => position !== index)"
    >
      {{ group.rules.length }}
      {{ group.mode === 'all' ? 'AND' : 'OR' }} conditions
      <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-for="value in projectStatuses"
      :key="`project-status-${value}`"
      type="button"
      class="filter-chip"
      :aria-label="`Clear ${value} project status filter`"
      @click="projectStatuses = projectStatuses.filter((item) => item !== value)"
    >
      Project status: {{ value }}
      <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-for="value in excludedProjectStatuses"
      :key="`excluded-project-status-${value}`"
      type="button"
      class="filter-chip"
      :aria-label="`Clear excluded ${value} project status filter`"
      @click="excludedProjectStatuses = excludedProjectStatuses.filter((item) => item !== value)"
    >
      Not project status: {{ value }}
      <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-for="value in projectPriorities"
      :key="`project-priority-${value}`"
      type="button"
      class="filter-chip"
      :aria-label="`Clear ${value} project priority filter`"
      @click="projectPriorities = projectPriorities.filter((item) => item !== value)"
    >
      Project priority: {{ value }}
      <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-for="value in excludedProjectPriorities"
      :key="`excluded-project-priority-${value}`"
      type="button"
      class="filter-chip"
      :aria-label="`Clear excluded ${value} project priority filter`"
      @click="
        excludedProjectPriorities = excludedProjectPriorities.filter((item) => item !== value)
      "
    >
      Not project priority: {{ value }}
      <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-for="value in projectLabels"
      :key="`project-label-${value}`"
      type="button"
      class="filter-chip"
      :aria-label="`Clear ${value} project label filter`"
      @click="projectLabels = projectLabels.filter((item) => item !== value)"
    >
      Project label: {{ value }} <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-for="value in excludedProjectLabels"
      :key="`excluded-project-label-${value}`"
      type="button"
      class="filter-chip"
      :aria-label="`Clear excluded ${value} project label filter`"
      @click="excludedProjectLabels = excludedProjectLabels.filter((item) => item !== value)"
    >
      Not project label: {{ value }}
      <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-if="emptyProjectLabels"
      type="button"
      class="filter-chip"
      aria-label="Clear no project labels filter"
      @click="emptyProjectLabels = false"
    >
      No project labels <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-if="excludedEmptyProjectLabels"
      type="button"
      class="filter-chip"
      aria-label="Clear has project labels filter"
      @click="excludedEmptyProjectLabels = false"
    >
      Has project labels <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      v-for="chip in planningChips"
      :key="`${chip.kind}-${chip.value ?? ''}`"
      type="button"
      class="filter-chip"
      :aria-label="`Clear ${chip.label.toLowerCase()} filter`"
      @click="removePlanningChip(chip)"
    >
      {{ chip.label }} <Icon name="i-tabler-x" class="size-3" />
    </button>
    <button
      type="button"
      class="rounded-md px-2 py-1 text-xs text-foreground-muted hover:bg-muted/60 hover:text-foreground-highlighted"
      @click="emit('clear')"
    >
      Clear all
    </button>
  </div>
</template>
