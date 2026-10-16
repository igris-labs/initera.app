<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue';
import { labelColor, priorityColor } from '../../lib/issue-presentation';
import type { ProjectPriority, ProjectStatus } from '../../lib/workspace-storage';
import { priorities } from '../../types/issue';
import type { DueFilter, IssueFilterGroup, Priority, Project } from '../../types/issue';
import { Button } from '../ui/button';
import { Icon } from '../ui/icon';
import { Input } from '../ui/input';
import { Popover } from '../ui/popover';
import IssueStatusIcon from './IssueStatusIcon.vue';
import ProjectPropertyFilterMenu from './ProjectPropertyFilterMenu.vue';
import AdvancedFilterMenu from './AdvancedFilterMenu.vue';
import ShortcutTooltip from '../common/ShortcutTooltip.vue';

type Category =
  | 'status'
  | 'priority'
  | 'labels'
  | 'milestone'
  | 'project'
  | 'projectProperties'
  | 'due'
  | 'advanced';

interface Props {
  statuses: string[];
  labels: string[];
  availableProjectLabels: string[];
  availableProjectMilestones: string[];
  availableMilestones: { id: string; label: string }[];
  actorId: string;
  actorName: string;
  projects: Project[];
  showProject: boolean;
  activeCount: number;
}

const props = defineProps<Props>();
const noLabelsOption = '\u0000empty-labels';
const noMilestoneOption = '\u0000empty-milestone';

const open = defineModel<boolean>('open', { required: true });
const search = defineModel<string>('search', { required: true });
const status = defineModel<string[]>('status', { required: true });
const excludedStatuses = defineModel<string[]>('excludedStatuses', { required: true });
const priority = defineModel<Priority[]>('priority', { required: true });
const excludedPriorities = defineModel<Priority[]>('excludedPriorities', { required: true });
const label = defineModel<string[]>('label', { required: true });
const excludedLabels = defineModel<string[]>('excludedLabels', { required: true });
const emptyLabels = defineModel<boolean>('emptyLabels', { required: true });
const excludedEmptyLabels = defineModel<boolean>('excludedEmptyLabels', { required: true });
const milestones = defineModel<string[]>('milestones', { required: true });
const excludedMilestones = defineModel<string[]>('excludedMilestones', { required: true });
const emptyMilestone = defineModel<boolean>('emptyMilestone', { required: true });
const excludedEmptyMilestone = defineModel<boolean>('excludedEmptyMilestone', { required: true });
const project = defineModel<Project[]>('project', { required: true });
const excludedProjects = defineModel<Project[]>('excludedProjects', { required: true });
const selectedProjectStatuses = defineModel<ProjectStatus[]>('projectStatuses', { required: true });
const excludedProjectStatuses = defineModel<ProjectStatus[]>('excludedProjectStatuses', {
  required: true,
});
const selectedProjectPriorities = defineModel<ProjectPriority[]>('projectPriorities', {
  required: true,
});
const excludedProjectPriorities = defineModel<ProjectPriority[]>('excludedProjectPriorities', {
  required: true,
});
const selectedProjectLabels = defineModel<string[]>('projectLabels', { required: true });
const excludedProjectLabels = defineModel<string[]>('excludedProjectLabels', { required: true });
const emptyProjectLabels = defineModel<boolean>('emptyProjectLabels', { required: true });
const excludedEmptyProjectLabels = defineModel<boolean>('excludedEmptyProjectLabels', {
  required: true,
});
const projectLeads = defineModel<string[]>('projectLeads', { required: true });
const excludedProjectLeads = defineModel<string[]>('excludedProjectLeads', { required: true });
const emptyProjectLead = defineModel<boolean>('emptyProjectLead', { required: true });
const excludedEmptyProjectLead = defineModel<boolean>('excludedEmptyProjectLead', {
  required: true,
});
const projectMilestones = defineModel<string[]>('projectMilestones', { required: true });
const excludedProjectMilestones = defineModel<string[]>('excludedProjectMilestones', {
  required: true,
});
const emptyProjectMilestones = defineModel<boolean>('emptyProjectMilestones', { required: true });
const excludedEmptyProjectMilestones = defineModel<boolean>('excludedEmptyProjectMilestones', {
  required: true,
});
const due = defineModel<DueFilter>('due', { required: true });
const advancedGroups = defineModel<IssueFilterGroup[]>('advancedGroups', { required: true });
const category = ref<Category | null>(null);
const operator = ref<'include' | 'exclude'>('include');
const searchInput = ref<{ inputRef?: HTMLInputElement } | null>(null);

let closeTimer: ReturnType<typeof setTimeout> | undefined;

const categories = computed(() => [
  {
    id: 'status' as const,
    title: 'Status',
    icon: 'i-tabler-circle-dashed',
    value: [...status.value, ...excludedStatuses.value.map((item) => `not ${item}`)].join(', '),
  },
  {
    id: 'priority' as const,
    title: 'Priority',
    icon: 'i-tabler-antenna-bars-5',
    value: [...priority.value, ...excludedPriorities.value.map((item) => `not ${item}`)].join(', '),
  },
  {
    id: 'labels' as const,
    title: 'Labels',
    icon: 'i-tabler-tag',
    value: [
      ...label.value,
      ...excludedLabels.value.map((item) => `not ${item}`),
      ...(emptyLabels.value ? ['no labels'] : []),
      ...(excludedEmptyLabels.value ? ['has labels'] : []),
    ].join(', '),
  },
  ...(props.showProject
    ? [
        {
          id: 'project' as const,
          title: 'Project',
          icon: 'i-tabler-cube',
          value: [...project.value, ...excludedProjects.value.map((item) => `not ${item}`)].join(
            ', ',
          ),
        },
      ]
    : []),
  {
    id: 'projectProperties' as const,
    title: 'Project properties',
    icon: 'i-tabler-cube',
    value: [
      ...selectedProjectStatuses.value,
      ...excludedProjectStatuses.value.map((item) => `not ${item}`),
      ...selectedProjectPriorities.value,
      ...excludedProjectPriorities.value.map((item) => `not ${item}`),
      ...selectedProjectLabels.value,
      ...excludedProjectLabels.value.map((item) => `not ${item}`),
      ...(emptyProjectLabels.value ? ['no labels'] : []),
      ...(excludedEmptyProjectLabels.value ? ['has labels'] : []),
      ...projectLeads.value.map((item) =>
        item === props.actorId ? props.actorName || 'You' : item,
      ),
      ...excludedProjectLeads.value.map(
        (item) => `not ${item === props.actorId ? props.actorName || 'You' : item}`,
      ),
      ...(emptyProjectLead.value ? ['no lead'] : []),
      ...(excludedEmptyProjectLead.value ? ['has lead'] : []),
      ...projectMilestones.value,
      ...excludedProjectMilestones.value.map((item) => `not ${item}`),
      ...(emptyProjectMilestones.value ? ['no milestones'] : []),
      ...(excludedEmptyProjectMilestones.value ? ['has milestones'] : []),
    ].join(', '),
  },
  {
    id: 'milestone' as const,
    title: 'Milestone',
    icon: 'i-tabler-diamond',
    value: [
      ...milestones.value.map(milestoneLabel),
      ...excludedMilestones.value.map((id) => `not ${milestoneLabel(id)}`),
      ...(emptyMilestone.value ? ['no milestone'] : []),
      ...(excludedEmptyMilestone.value ? ['has milestone'] : []),
    ].join(', '),
  },
  {
    id: 'due' as const,
    title: 'Due date',
    icon: 'i-tabler-calendar',
    value: due.value === 'any' ? '' : due.value === 'has' ? 'Has date' : 'No date',
  },
  {
    id: 'advanced' as const,
    title: 'Advanced filter',
    icon: 'i-tabler-filter-cog',
    value: advancedGroups.value.length ? `${advancedGroups.value.length} groups` : '',
  },
]);

function milestoneLabel(id: string) {
  return (
    props.availableMilestones.find((milestone) => milestone.id === id)?.label ?? 'Removed milestone'
  );
}

const options = computed(() => {
  switch (category.value) {
    case 'status':
      return ['All statuses', ...props.statuses];
    case 'priority':
      return ['Any priority', ...priorities];
    case 'labels':
      return ['Any label', noLabelsOption, ...props.labels];
    case 'milestone':
      return [
        'Any milestone',
        noMilestoneOption,
        ...props.availableMilestones.map((item) => item.id),
      ];
    case 'project':
      return ['Any project', ...props.projects];
    case 'due':
      return ['Any date', 'Has due date', 'No due date'];
    default:
      return [];
  }
});

function isSelected(value: string) {
  switch (category.value) {
    case 'status':
      return value === 'All statuses'
        ? !status.value.length && !excludedStatuses.value.length
        : (operator.value === 'include' ? status.value : excludedStatuses.value).includes(value);
    case 'priority':
      return value === 'Any priority'
        ? !priority.value.length && !excludedPriorities.value.length
        : (operator.value === 'include' ? priority.value : excludedPriorities.value).includes(
            value as Priority,
          );
    case 'labels':
      if (value === noLabelsOption)
        return operator.value === 'include' ? emptyLabels.value : excludedEmptyLabels.value;
      return value === 'Any label'
        ? !label.value.length &&
            !excludedLabels.value.length &&
            !emptyLabels.value &&
            !excludedEmptyLabels.value
        : (operator.value === 'include' ? label.value : excludedLabels.value).includes(value);
    case 'project':
      return value === 'Any project'
        ? !project.value.length && !excludedProjects.value.length
        : (operator.value === 'include' ? project.value : excludedProjects.value).includes(value);
    case 'milestone':
      if (value === noMilestoneOption)
        return operator.value === 'include' ? emptyMilestone.value : excludedEmptyMilestone.value;
      return value === 'Any milestone'
        ? !milestones.value.length &&
            !excludedMilestones.value.length &&
            !emptyMilestone.value &&
            !excludedEmptyMilestone.value
        : (operator.value === 'include' ? milestones.value : excludedMilestones.value).includes(
            value,
          );
    case 'due':
      return (
        (value === 'Any date' && due.value === 'any') ||
        (value === 'Has due date' && due.value === 'has') ||
        (value === 'No due date' && due.value === 'none')
      );
    default:
      return false;
  }
}

function toggleValue<T>(values: T[], value: T): T[] {
  return values.includes(value) ? values.filter((item) => item !== value) : [...values, value];
}

function choose(value: string) {
  switch (category.value) {
    case 'status':
      if (value === 'All statuses') {
        status.value = [];
        excludedStatuses.value = [];
      } else if (operator.value === 'include') {
        status.value = toggleValue(status.value, value);
        excludedStatuses.value = excludedStatuses.value.filter((item) => item !== value);
      } else {
        excludedStatuses.value = toggleValue(excludedStatuses.value, value);
        status.value = status.value.filter((item) => item !== value);
      }

      break;
    case 'priority':
      if (value === 'Any priority') {
        priority.value = [];
        excludedPriorities.value = [];
      } else if (operator.value === 'include') {
        priority.value = toggleValue(priority.value, value as Priority);
        excludedPriorities.value = excludedPriorities.value.filter((item) => item !== value);
      } else {
        excludedPriorities.value = toggleValue(excludedPriorities.value, value as Priority);
        priority.value = priority.value.filter((item) => item !== value);
      }

      break;
    case 'labels':
      if (value === 'Any label') {
        label.value = [];
        excludedLabels.value = [];
        emptyLabels.value = false;
        excludedEmptyLabels.value = false;
      } else if (value === noLabelsOption && operator.value === 'include') {
        emptyLabels.value = !emptyLabels.value;
        excludedEmptyLabels.value = false;
      } else if (value === noLabelsOption) {
        excludedEmptyLabels.value = !excludedEmptyLabels.value;
        emptyLabels.value = false;
      } else if (operator.value === 'include') {
        label.value = toggleValue(label.value, value);
        excludedLabels.value = excludedLabels.value.filter((item) => item !== value);
      } else {
        excludedLabels.value = toggleValue(excludedLabels.value, value);
        label.value = label.value.filter((item) => item !== value);
      }

      break;
    case 'project':
      if (value === 'Any project') {
        project.value = [];
        excludedProjects.value = [];
      } else if (operator.value === 'include') {
        project.value = toggleValue(project.value, value as Project);
        excludedProjects.value = excludedProjects.value.filter((item) => item !== value);
      } else {
        excludedProjects.value = toggleValue(excludedProjects.value, value as Project);
        project.value = project.value.filter((item) => item !== value);
      }

      break;
    case 'milestone':
      if (value === 'Any milestone') {
        milestones.value = [];
        excludedMilestones.value = [];
        emptyMilestone.value = false;
        excludedEmptyMilestone.value = false;
      } else if (value === noMilestoneOption && operator.value === 'include') {
        emptyMilestone.value = !emptyMilestone.value;
        excludedEmptyMilestone.value = false;
      } else if (value === noMilestoneOption) {
        excludedEmptyMilestone.value = !excludedEmptyMilestone.value;
        emptyMilestone.value = false;
      } else if (operator.value === 'include') {
        milestones.value = toggleValue(milestones.value, value);
        excludedMilestones.value = excludedMilestones.value.filter((item) => item !== value);
      } else {
        excludedMilestones.value = toggleValue(excludedMilestones.value, value);
        milestones.value = milestones.value.filter((item) => item !== value);
      }

      break;
    case 'due':
      due.value = value === 'Has due date' ? 'has' : value === 'No due date' ? 'none' : 'any';
      break;
  }

  if (category.value === 'due') category.value = null;
}

function cancelClose() {
  if (closeTimer) clearTimeout(closeTimer);
  closeTimer = undefined;
}

function showCategory(value: Category) {
  cancelClose();
  if (category.value !== value) operator.value = 'include';
  category.value = value;
}

function scheduleClose() {
  cancelClose();
  closeTimer = setTimeout(() => {
    category.value = null;
  }, 400);
}

async function focusFirstOption(value: Category) {
  showCategory(value);
  await nextTick();
  document
    .getElementById('issue-filter-options')
    ?.querySelector<HTMLButtonElement>('[data-filter-option]')
    ?.focus();
}

function returnToCategory() {
  category.value = null;
  searchInput.value?.inputRef?.focus();
}

onBeforeUnmount(cancelClose);

watch(open, async (value) => {
  if (!value) {
    cancelClose();
    category.value = null;
    return;
  }

  await nextTick();
  searchInput.value?.inputRef?.focus();
});
</script>

<template>
  <Popover v-model:open="open" size="sm" align="end" teleport="body" class="workbench-surface w-60">
    <template #trigger="{ toggle, triggerProps }">
      <ShortcutTooltip label="Filter issues" shortcut="filter">
        <Button
          v-bind="triggerProps"
          icon="i-tabler-filter"
          aria-label="Filter issues"
          variant="ghost"
          size="xs"
          :aria-pressed="open || activeCount > 0"
          @click="toggle"
        />
      </ShortcutTooltip>
    </template>
    <template #content>
      <div class="p-1 text-xs">
        <Input
          ref="searchInput"
          v-model="search"
          icon="i-tabler-search"
          aria-label="Search issues"
          placeholder="Search issues..."
          size="sm"
          class="mb-1 w-full"
        />
        <p class="px-2 py-1 text-xs text-foreground-dimmed">Add filter</p>
        <div
          v-for="item in categories"
          :key="item.id"
          class="relative"
          @pointerleave="scheduleClose"
        >
          <button
            type="button"
            :aria-label="item.title"
            :aria-expanded="category === item.id"
            aria-haspopup="menu"
            :aria-controls="category === item.id ? 'issue-filter-options' : undefined"
            :data-filter-category="item.id"
            class="flex min-h-8 w-full items-center gap-2 rounded-md px-2 text-left text-xs text-foreground hover:bg-accented focus-visible:outline-2 focus-visible:outline-info aria-expanded:bg-accented"
            @pointerenter="showCategory(item.id)"
            @focus="showCategory(item.id)"
            @click="showCategory(item.id)"
            @keydown.left.prevent="focusFirstOption(item.id)"
            @keydown.enter.prevent="focusFirstOption(item.id)"
            @keydown.space.prevent="focusFirstOption(item.id)"
          >
            <Icon :name="item.icon" class="size-4 shrink-0 text-foreground-muted" />
            <span class="min-w-0 flex-1">{{ item.title }}</span>
            <span v-if="item.value" class="max-w-22 truncate text-foreground-muted">{{
              item.value
            }}</span>
            <Icon name="i-tabler-chevron-right" class="size-3 shrink-0 text-foreground-dimmed" />
          </button>
          <div
            v-if="category === item.id"
            id="issue-filter-options"
            role="menu"
            :aria-label="`${item.title} options`"
            class="absolute top-0 right-[calc(100%+0.125rem)] z-50 rounded-lg border border-outline bg-elevated p-1 shadow-lg before:absolute before:top-0 before:-right-1 before:h-full before:w-1"
            :class="category === 'advanced' ? 'w-84' : 'w-56'"
            @pointerenter="cancelClose"
            @keydown.esc.stop.prevent="returnToCategory"
          >
            <div
              v-if="
                category !== 'due' && category !== 'projectProperties' && category !== 'advanced'
              "
              class="mb-1 flex gap-1 border-b border-outline px-1 pb-1"
            >
              <button
                v-for="choice in ['include', 'exclude'] as const"
                :key="choice"
                type="button"
                :aria-pressed="operator === choice"
                class="rounded-md px-2 py-1 text-xs capitalize text-foreground-muted hover:bg-accented aria-pressed:bg-muted aria-pressed:text-foreground"
                @click="operator = choice"
              >
                {{ choice === 'include' ? 'Include' : 'Exclude' }}
              </button>
            </div>
            <div
              :class="
                category === 'projectProperties' || category === 'advanced'
                  ? 'overflow-visible'
                  : 'max-h-56 overflow-auto'
              "
            >
              <ProjectPropertyFilterMenu
                v-if="category === 'projectProperties'"
                v-model:statuses="selectedProjectStatuses"
                v-model:excluded-statuses="excludedProjectStatuses"
                v-model:priorities="selectedProjectPriorities"
                v-model:excluded-priorities="excludedProjectPriorities"
                v-model:labels="selectedProjectLabels"
                v-model:excluded-labels="excludedProjectLabels"
                v-model:empty-labels="emptyProjectLabels"
                v-model:excluded-empty-labels="excludedEmptyProjectLabels"
                v-model:leads="projectLeads"
                v-model:excluded-leads="excludedProjectLeads"
                v-model:empty-lead="emptyProjectLead"
                v-model:excluded-empty-lead="excludedEmptyProjectLead"
                v-model:milestones="projectMilestones"
                v-model:excluded-milestones="excludedProjectMilestones"
                v-model:empty-milestones="emptyProjectMilestones"
                v-model:excluded-empty-milestones="excludedEmptyProjectMilestones"
                :available-labels="props.availableProjectLabels"
                :available-milestones="props.availableProjectMilestones"
                :actor-id="props.actorId"
                :actor-name="props.actorName"
              />
              <AdvancedFilterMenu v-if="category === 'advanced'" v-model="advancedGroups" />
              <button
                v-for="option in category === 'projectProperties' ? [] : options"
                :key="option"
                type="button"
                :role="category === 'due' ? 'menuitemradio' : 'menuitemcheckbox'"
                :aria-checked="isSelected(option)"
                data-filter-option
                class="flex min-h-8 w-full items-center gap-2 rounded-md px-2 text-left text-xs text-foreground hover:bg-accented focus-visible:outline-2 focus-visible:outline-info aria-checked:bg-muted"
                @click="choose(option)"
              >
                <Icon
                  v-if="category === 'priority' && option !== 'Any priority'"
                  name="i-tabler-antenna-bars-5"
                  :class="['size-4', priorityColor[option as Priority]]"
                />
                <IssueStatusIcon
                  v-if="category === 'status' && option !== 'All statuses'"
                  :status="option"
                />
                <span
                  v-if="
                    category === 'labels' && option !== 'Any label' && option !== noLabelsOption
                  "
                  class="size-2 shrink-0 rounded-full"
                  :style="{ backgroundColor: labelColor[option] ?? 'var(--label-gray)' }"
                />
                <span class="min-w-0 flex-1 truncate">{{
                  option === noLabelsOption
                    ? 'No labels'
                    : option === noMilestoneOption
                      ? 'No milestone'
                      : category === 'milestone' && option !== 'Any milestone'
                        ? milestoneLabel(option)
                        : option
                }}</span>
                <Icon
                  v-if="isSelected(option)"
                  name="i-tabler-check"
                  class="size-4 shrink-0 text-info"
                />
              </button>
            </div>
          </div>
        </div>
      </div>
    </template>
  </Popover>
</template>
