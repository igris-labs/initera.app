<script setup lang="ts">
import type { Issue } from '../../types/issue';
import { priorityColor } from '../../lib/issue-presentation';
import IssueStatusIcon from './IssueStatusIcon.vue';
import { Badge } from '../ui/badge';
import { Checkbox } from '../ui/checkbox';
import { Icon } from '../ui/icon';
import { computed } from 'vue';
import { issueTableTemplate, visibleIssueColumns } from './issue-table-columns';
import type { IssueTableWidths } from './issue-table-columns';

interface Props {
  issue: Issue;
  selected: boolean;
  showPriority: boolean;
  showProject: boolean;
  columns?: string[];
  columnWidths?: IssueTableWidths;
  density?: 'compact' | 'default';
  bulkSelectable?: boolean;
  bulkSelected?: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  open: [];
  move: [event: KeyboardEvent];
  toggleBulk: [];
}>();
const visibleColumns = computed(() =>
  visibleIssueColumns(props.columns, props.showPriority, props.showProject),
);
const gridTemplate = computed(() => issueTableTemplate(visibleColumns.value, props.columnWidths));
function trackedTime(seconds: number) {
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m`;
  return `${Math.floor(minutes / 60)}h ${minutes % 60}m`;
}
</script>

<template>
  <div v-if="bulkSelectable" class="relative flex min-w-0 items-center">
    <Checkbox
      class="absolute top-2.5 left-3 z-10 size-4 shrink-0 accent-primary"
      :model-value="!!bulkSelected"
      :aria-label="`Select ${issue.id} for triage`"
      @update:model-value="emit('toggleBulk')"
    />
  </div>
  <button
    type="button"
    :data-issue-id="issue.id"
    :aria-label="`Open ${issue.id}: ${issue.title}; ${issue.status}; ${issue.priority}`"
    aria-keyshortcuts="Alt+ArrowUp Alt+ArrowDown Alt+ArrowLeft Alt+ArrowRight"
    :class="[
      'grid min-w-max w-full cursor-default items-center rounded-md text-left text-(length:--ui-sidebar-text) transition-colors duration-150 ease-out focus-visible:relative focus-visible:z-10 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-inverted max-md:min-h-11',
      density === 'compact' ? 'min-h-7 py-1' : 'min-h-9 py-1.5',
      selected ? 'bg-muted/45 text-foreground-highlighted' : 'hover:bg-muted/35',
      bulkSelectable ? 'pl-10' : '',
    ]"
    :style="{ gridTemplateColumns: gridTemplate }"
    @click="emit('open')"
    @keydown="emit('move', $event)"
  >
    <span
      v-for="column in visibleColumns"
      :key="column"
      class="flex min-w-0 items-center gap-1.5 overflow-hidden px-3 text-xs text-foreground-muted"
    >
      <template v-if="column === 'identifier'"
        ><span class="text-data truncate">{{ issue.id }}</span></template
      >
      <template v-else-if="column === 'title'"
        ><span class="truncate font-medium text-foreground-highlighted">{{
          issue.title
        }}</span></template
      >
      <template v-else-if="column === 'status'"
        ><IssueStatusIcon :status="issue.status" /><span class="truncate">{{
          issue.status
        }}</span></template
      >
      <template v-else-if="column === 'priority'"
        ><Icon
          name="i-tabler-antenna-bars-5"
          :class="['size-3.5 shrink-0', priorityColor[issue.priority]]"
        /><span class="truncate">{{ issue.priority }}</span></template
      >
      <template v-else-if="column === 'labels'"
        ><Badge
          v-for="label in issue.labels.slice(0, 3)"
          :key="label"
          :label="label"
          dot
          size="xs"
          variant="outline"
        /><span v-if="issue.labels.length > 3">+{{ issue.labels.length - 3 }}</span></template
      >
      <template v-else-if="column === 'project'"
        ><span class="truncate">{{ issue.project }}</span></template
      >
      <template v-else-if="column === 'node'"
        ><span class="truncate">{{ issue.nodeName }}</span></template
      >
      <template v-else-if="column === 'milestone'"
        ><span class="truncate">{{ issue.milestoneName }}</span></template
      >
      <template v-else-if="column === 'cycle'"
        ><span class="truncate">{{ issue.cycleName }}</span></template
      >
      <template v-else-if="column === 'estimate'">{{ issue.estimate ?? '—' }}</template>
      <template v-else-if="column === 'dueDate'">{{ issue.due ?? '—' }}</template>
      <template v-else-if="column === 'trackedTime'">{{
        issue.trackedSeconds ? trackedTime(issue.trackedSeconds) : '—'
      }}</template>
      <template v-else-if="column === 'updatedAt'">{{ issue.updated }}</template>
    </span>
  </button>
</template>
