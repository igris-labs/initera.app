<script setup lang="ts">
import { computed, nextTick, ref } from 'vue';
import { labelColor, priorityColor } from '../../lib/issue-presentation';
import { projectStatuses } from '../../lib/project-presentation';
import type { ProjectPriority, ProjectStatus } from '../../lib/workspace-storage';
import { priorities } from '../../types/issue';
import type { Priority } from '../../types/issue';
import { Icon } from '../ui/icon';

type Property = 'status' | 'priority' | 'labels' | 'lead' | 'milestone';
type Operator = 'include' | 'exclude';

interface Props {
  availableLabels: string[];
  availableMilestones: string[];
  actorId: string;
  actorName: string;
}

const { availableLabels, availableMilestones, actorId, actorName } = defineProps<Props>();
const noLabelsOption = '\u0000empty-project-labels';
const noLeadOption = '\u0000empty-project-lead';
const noMilestonesOption = '\u0000empty-project-milestones';

const statuses = defineModel<ProjectStatus[]>('statuses', { required: true });
const excludedStatuses = defineModel<ProjectStatus[]>('excludedStatuses', { required: true });
const selectedPriorities = defineModel<ProjectPriority[]>('priorities', { required: true });
const excludedPriorities = defineModel<ProjectPriority[]>('excludedPriorities', { required: true });
const selectedLabels = defineModel<string[]>('labels', { required: true });
const excludedLabels = defineModel<string[]>('excludedLabels', { required: true });
const emptyLabels = defineModel<boolean>('emptyLabels', { required: true });
const excludedEmptyLabels = defineModel<boolean>('excludedEmptyLabels', { required: true });
const leads = defineModel<string[]>('leads', { required: true });
const excludedLeads = defineModel<string[]>('excludedLeads', { required: true });
const emptyLead = defineModel<boolean>('emptyLead', { required: true });
const excludedEmptyLead = defineModel<boolean>('excludedEmptyLead', { required: true });
const milestones = defineModel<string[]>('milestones', { required: true });
const excludedMilestones = defineModel<string[]>('excludedMilestones', { required: true });
const emptyMilestones = defineModel<boolean>('emptyMilestones', { required: true });
const excludedEmptyMilestones = defineModel<boolean>('excludedEmptyMilestones', { required: true });

const activeProperty = ref<Property | null>(null);
const operator = ref<Operator>('include');

const properties = [
  { id: 'status', title: 'Project status', icon: 'i-tabler-circle-dashed' },
  { id: 'priority', title: 'Project priority', icon: 'i-tabler-antenna-bars-5' },
  { id: 'labels', title: 'Project labels', icon: 'i-tabler-tag' },
  { id: 'lead', title: 'Project lead', icon: 'i-tabler-user' },
  { id: 'milestone', title: 'Milestone name', icon: 'i-tabler-diamond' },
] as const;

const options = computed(() => {
  if (activeProperty.value === 'status') return projectStatuses.map((item) => item.value);
  if (activeProperty.value === 'priority') return [...priorities];
  if (activeProperty.value === 'labels') return [noLabelsOption, ...availableLabels];
  if (activeProperty.value === 'lead') return [noLeadOption, ...(actorId ? [actorId] : [])];
  if (activeProperty.value === 'milestone') return [noMilestonesOption, ...availableMilestones];
  return [];
});

function optionLabel(value: string) {
  if (value === noLabelsOption) return 'No project labels';
  if (value === noLeadOption) return 'No lead';
  if (value === noMilestonesOption) return 'No milestones';
  if (activeProperty.value === 'lead' && value === actorId) return actorName || 'You';
  return value;
}

function show(property: Property) {
  if (activeProperty.value !== property) operator.value = 'include';
  activeProperty.value = property;
}

async function focusChoices(property: Property) {
  show(property);
  await nextTick();
  document.querySelector<HTMLButtonElement>('[data-project-property-choice]')?.focus();
}

function toggle<T>(items: T[], value: T): T[] {
  return items.includes(value) ? items.filter((item) => item !== value) : [...items, value];
}

function selected(value: string) {
  if (activeProperty.value === 'status')
    return (operator.value === 'include' ? statuses.value : excludedStatuses.value).includes(
      value as ProjectStatus,
    );

  if (activeProperty.value === 'priority')
    return (
      operator.value === 'include' ? selectedPriorities.value : excludedPriorities.value
    ).includes(value as ProjectPriority);

  if (activeProperty.value === 'lead') {
    if (value === noLeadOption)
      return operator.value === 'include' ? emptyLead.value : excludedEmptyLead.value;
    return (operator.value === 'include' ? leads.value : excludedLeads.value).includes(value);
  }

  if (activeProperty.value === 'milestone') {
    if (value === noMilestonesOption)
      return operator.value === 'include' ? emptyMilestones.value : excludedEmptyMilestones.value;
    return (operator.value === 'include' ? milestones.value : excludedMilestones.value).includes(
      value,
    );
  }

  if (value === noLabelsOption)
    return operator.value === 'include' ? emptyLabels.value : excludedEmptyLabels.value;

  return (operator.value === 'include' ? selectedLabels.value : excludedLabels.value).includes(
    value,
  );
}

function choose(value: string) {
  if (activeProperty.value === 'status') {
    const status = value as ProjectStatus;
    if (operator.value === 'include') {
      statuses.value = toggle(statuses.value, status);
      excludedStatuses.value = excludedStatuses.value.filter((item) => item !== status);
    } else {
      excludedStatuses.value = toggle(excludedStatuses.value, status);
      statuses.value = statuses.value.filter((item) => item !== status);
    }

    return;
  }

  if (activeProperty.value === 'priority') {
    const priority = value as ProjectPriority;
    if (operator.value === 'include') {
      selectedPriorities.value = toggle(selectedPriorities.value, priority);
      excludedPriorities.value = excludedPriorities.value.filter((item) => item !== priority);
    } else {
      excludedPriorities.value = toggle(excludedPriorities.value, priority);
      selectedPriorities.value = selectedPriorities.value.filter((item) => item !== priority);
    }

    return;
  }

  if (activeProperty.value === 'lead') {
    if (value === noLeadOption) {
      if (operator.value === 'include') {
        emptyLead.value = !emptyLead.value;
        excludedEmptyLead.value = false;
      } else {
        excludedEmptyLead.value = !excludedEmptyLead.value;
        emptyLead.value = false;
      }
    } else if (operator.value === 'include') {
      leads.value = toggle(leads.value, value);
      excludedLeads.value = excludedLeads.value.filter((item) => item !== value);
    } else {
      excludedLeads.value = toggle(excludedLeads.value, value);
      leads.value = leads.value.filter((item) => item !== value);
    }

    return;
  }

  if (activeProperty.value === 'milestone') {
    if (value === noMilestonesOption) {
      if (operator.value === 'include') {
        emptyMilestones.value = !emptyMilestones.value;
        excludedEmptyMilestones.value = false;
      } else {
        excludedEmptyMilestones.value = !excludedEmptyMilestones.value;
        emptyMilestones.value = false;
      }
    } else if (operator.value === 'include') {
      milestones.value = toggle(milestones.value, value);
      excludedMilestones.value = excludedMilestones.value.filter((item) => item !== value);
    } else {
      excludedMilestones.value = toggle(excludedMilestones.value, value);
      milestones.value = milestones.value.filter((item) => item !== value);
    }

    return;
  }

  if (value === noLabelsOption) {
    if (operator.value === 'include') {
      emptyLabels.value = !emptyLabels.value;
      excludedEmptyLabels.value = false;
    } else {
      excludedEmptyLabels.value = !excludedEmptyLabels.value;
      emptyLabels.value = false;
    }

    return;
  }

  if (operator.value === 'include') {
    selectedLabels.value = toggle(selectedLabels.value, value);
    excludedLabels.value = excludedLabels.value.filter((item) => item !== value);
  } else {
    excludedLabels.value = toggle(excludedLabels.value, value);
    selectedLabels.value = selectedLabels.value.filter((item) => item !== value);
  }
}
</script>

<template>
  <div class="relative">
    <div v-for="property in properties" :key="property.id" class="relative">
      <button
        type="button"
        :data-filter-option="property.id"
        :aria-expanded="activeProperty === property.id"
        class="flex min-h-8 w-full items-center gap-2 rounded-md px-2 text-left text-xs text-foreground hover:bg-accented aria-expanded:bg-accented"
        @pointerenter="show(property.id)"
        @focus="show(property.id)"
        @click="show(property.id)"
        @keydown.left.prevent="focusChoices(property.id)"
        @keydown.enter.prevent="focusChoices(property.id)"
        @keydown.space.prevent="focusChoices(property.id)"
      >
        <Icon :name="property.icon" class="size-4 text-foreground-muted" />
        <span class="flex-1">{{ property.title }}</span>
        <Icon name="i-tabler-chevron-right" class="size-3 text-foreground-dimmed" />
      </button>

      <div
        v-if="activeProperty === property.id"
        role="menu"
        :aria-label="`${property.title} options`"
        class="absolute top-0 right-[calc(100%+0.125rem)] z-50 w-52 rounded-lg border border-outline bg-elevated p-1 shadow-lg before:absolute before:top-0 before:-right-1 before:h-full before:w-1"
      >
        <div class="mb-1 flex gap-1 border-b border-outline px-1 pb-1">
          <button
            v-for="choice in ['include', 'exclude'] as const"
            :key="choice"
            type="button"
            :aria-pressed="operator === choice"
            class="rounded-md px-2 py-1 text-xs capitalize text-foreground-muted hover:bg-accented aria-pressed:bg-muted aria-pressed:text-foreground"
            @click="operator = choice"
          >
            {{ choice }}
          </button>
        </div>

        <div class="max-h-56 overflow-y-auto">
          <button
            v-for="value in options"
            :key="value"
            type="button"
            role="menuitemcheckbox"
            data-project-property-choice
            :aria-checked="selected(value)"
            class="flex min-h-8 w-full items-center gap-2 rounded-md px-2 text-left text-xs text-foreground hover:bg-accented aria-checked:bg-muted"
            @click="choose(value)"
          >
            <Icon
              v-if="property.id === 'status'"
              :name="projectStatuses.find((item) => item.value === value)!.icon"
              :class="['size-4', projectStatuses.find((item) => item.value === value)!.iconClass]"
            />
            <Icon
              v-else-if="property.id === 'priority'"
              name="i-tabler-antenna-bars-5"
              :class="['size-4', priorityColor[value as Priority]]"
            />
            <Icon
              v-else-if="property.id === 'lead'"
              name="i-tabler-user"
              class="size-4 text-foreground-muted"
            />
            <Icon
              v-else-if="property.id === 'milestone'"
              name="i-tabler-diamond"
              class="size-4 text-foreground-muted"
            />
            <span
              v-else-if="property.id === 'labels' && value !== noLabelsOption"
              class="size-2 shrink-0 rounded-full"
              :style="{ backgroundColor: labelColor[value] ?? 'var(--label-gray)' }"
            />
            <span class="min-w-0 flex-1 truncate">{{ optionLabel(value) }}</span>
            <Icon v-if="selected(value)" name="i-tabler-check" class="size-4 text-info" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
