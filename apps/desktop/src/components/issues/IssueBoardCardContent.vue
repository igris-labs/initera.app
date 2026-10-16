<script setup lang="ts">
import { priorityColor } from '../../lib/issue-presentation';
import type { Issue } from '../../types/issue';
import { Badge } from '../ui/badge';
import { Icon } from '../ui/icon';
import IssueStatusIcon from './IssueStatusIcon.vue';

interface Props {
  issue: Issue;
  showPriority: boolean;
  showProject: boolean;
  columns?: string[];
  density?: 'compact' | 'default';
}

defineProps<Props>();

function trackedTime(seconds: number) {
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m`;
  return `${Math.floor(minutes / 60)}h ${minutes % 60}m`;
}
</script>

<template>
  <div :class="['flex items-start gap-2', density === 'compact' ? 'mb-0.5' : 'mb-1']">
    <span class="text-data text-xs text-foreground-muted">{{ issue.id }}</span>
  </div>
  <div
    class="flex w-full items-start gap-2 text-left text-(length:--ui-sidebar-text) font-medium leading-5 text-foreground-highlighted"
  >
    <IssueStatusIcon
      v-if="!columns || columns.includes('status')"
      :status="issue.status"
      size="mt-0.5 size-4 shrink-0"
    />
    <span>{{ issue.title }}</span>
  </div>
  <div
    :class="[
      'flex min-h-5 flex-wrap items-center gap-1.5',
      density === 'compact' ? 'mt-1.5' : 'mt-3',
    ]"
  >
    <Icon
      v-if="showPriority && (!columns || columns.includes('priority'))"
      name="i-tabler-antenna-bars-5"
      :class="['size-3.5', priorityColor[issue.priority]]"
      :aria-label="issue.priority"
    />
    <template v-if="!columns || columns.includes('labels')">
      <Badge
        v-for="label in issue.labels.slice(0, 2)"
        :key="label"
        :label="label"
        dot
        size="xs"
        variant="outline"
      />
    </template>
    <span
      v-if="issue.labels.length > 2 && (!columns || columns.includes('labels'))"
      class="text-xs text-foreground-muted"
    >
      +{{ issue.labels.length - 2 }}
    </span>
    <span
      v-if="showProject && (!columns || columns.includes('project'))"
      class="ml-auto max-w-28 truncate text-xs text-foreground-muted"
    >
      {{ issue.project }}
    </span>
    <span
      v-if="columns?.includes('estimate') && issue.estimate != null"
      class="text-xs text-foreground-muted"
      >{{ issue.estimate }} pts</span
    >
    <span v-if="columns?.includes('dueDate') && issue.due" class="text-xs text-foreground-muted">{{
      issue.due
    }}</span>
    <span
      v-if="columns?.includes('milestone') && issue.milestoneName"
      class="max-w-32 truncate text-xs text-foreground-muted"
      >{{ issue.milestoneName }}</span
    >
    <span
      v-if="columns?.includes('node') && issue.nodeName"
      class="max-w-32 truncate text-xs text-foreground-muted"
      >{{ issue.nodeName }}</span
    >
    <span
      v-if="columns?.includes('cycle') && issue.cycleName"
      class="max-w-32 truncate text-xs text-foreground-muted"
      >{{ issue.cycleName }}</span
    >
    <span
      v-if="columns?.includes('trackedTime') && issue.trackedSeconds"
      class="text-xs text-foreground-muted"
      >{{ trackedTime(issue.trackedSeconds) }}</span
    >
  </div>
</template>
