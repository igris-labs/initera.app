<script setup lang="ts">
import { useVirtualizer } from '@tanstack/vue-virtual';
import { computed, nextTick, ref } from 'vue';
import { useIssuesStore } from '../../stores/issues';
import type { Issue, Status } from '../../types/issue';
import IssueListRow from './IssueListRow.vue';
import IssueStatusIcon from './IssueStatusIcon.vue';
import IssueTableHeader from './IssueTableHeader.vue';
import { visibleIssueColumns } from './issue-table-columns';
import type { IssueTableColumn, IssueTableWidths } from './issue-table-columns';
import { Icon } from '../ui/icon';

interface Props {
  items: Issue[];
  grouped: boolean;
  groupBy?: 'none' | 'status' | 'priority' | 'project';
  showPriority: boolean;
  showProject: boolean;
  columns?: string[];
  columnWidths?: IssueTableWidths;
  sortField?: string;
  sortDirection?: 'asc' | 'desc';
  density?: 'compact' | 'default';
  showEmptyGroups?: boolean;
  bulkSelectable?: boolean;
  bulkSelectedIds?: string[];
}

type ListRow =
  | { kind: 'header'; key: string; name: string; count: number }
  | { kind: 'issue'; key: string; issue: Issue; group: string; index: number };

const props = defineProps<Props>();

const emit = defineEmits<{
  move: [id: string, property: 'status', value: string, index: number];
  reorder: [id: string, index: number];
  toggleBulk: [id: string];
  resizeColumn: [column: IssueTableColumn, width: number];
  sortColumn: [column: IssueTableColumn];
}>();

const selectedId = defineModel<string | null>('selectedId', { required: true });
const issues = useIssuesStore();
const scrollElement = ref<HTMLElement | null>(null);
const collapsed = ref<string[]>([]);
const draggingId = ref<string | null>(null);

const groups = computed(() => {
  if (!props.grouped) return [{ name: 'Issues', items: props.items }];
  const field =
    props.groupBy === 'priority' || props.groupBy === 'project' ? props.groupBy : 'status';
  const names =
    field === 'status'
      ? issues.visibleStatusGroups
      : [...new Set(props.items.map((issue) => issue[field]))];
  return names
    .map((name) => ({
      name,
      items: props.items.filter((issue) => issue[field] === name),
    }))
    .filter((group) => props.showEmptyGroups || group.items.length);
});
const tableColumns = computed(() =>
  visibleIssueColumns(props.columns, props.showPriority, props.showProject),
);

const rows = computed<ListRow[]>(() =>
  groups.value.flatMap((group) => [
    ...(props.grouped
      ? [
          {
            kind: 'header' as const,
            key: `header-${group.name}`,
            name: group.name,
            count: group.items.length,
          },
        ]
      : []),
    ...(collapsed.value.includes(group.name)
      ? []
      : group.items.map((issue, index) => ({
          kind: 'issue' as const,
          key: `issue-${issue.id}`,
          issue,
          group: group.name,
          index,
        }))),
  ]),
);

function itemKey(index: number) {
  return rows.value[index]?.key ?? index;
}

const virtualizer = useVirtualizer(
  computed(() => ({
    count: rows.value.length,
    getScrollElement: () => scrollElement.value,
    estimateSize: (index: number) => (rows.value[index]?.kind === 'header' ? 44 : 40),
    getItemKey: itemKey,
    overscan: 8,
  })),
);

const visibleRows = computed(() =>
  virtualizer.value.getVirtualItems().flatMap((item) => {
    const row = rows.value[item.index];
    return row ? [{ item, row }] : [];
  }),
);

function toggleGroup(name: string) {
  collapsed.value = collapsed.value.includes(name)
    ? collapsed.value.filter((item) => item !== name)
    : [...collapsed.value, name];
}

function startDrag(event: DragEvent, id: string) {
  if (props.grouped && props.groupBy !== 'status') return;
  draggingId.value = id;
  if (!event.dataTransfer) return;
  event.dataTransfer.effectAllowed = 'move';
  event.dataTransfer.setData('text/plain', id);
}

function dropOn(event: DragEvent, row: ListRow) {
  if (props.grouped && props.groupBy !== 'status') return;
  event.preventDefault();
  const id = event.dataTransfer?.getData('text/plain') || draggingId.value;
  draggingId.value = null;
  if (!id) return;

  const group = row.kind === 'header' ? row.name : row.group;
  // The dragged issue leaves its old position before the destination index is calculated.
  const peers =
    groups.value.find((item) => item.name === group)?.items.filter((item) => item.id !== id) ?? [];
  const target = row.kind === 'header' ? 0 : peers.findIndex((item) => item.id === row.issue.id);
  if (target < 0) return;
  const bounds = (event.currentTarget as HTMLElement).getBoundingClientRect();
  const after = row.kind === 'issue' && event.clientY > bounds.top + bounds.height / 2;
  const index = target + Number(after);

  if (props.grouped) emit('move', id, 'status', group, index);
  else emit('reorder', id, index);
}

async function moveWithKeyboard(event: KeyboardEvent, issue: Issue) {
  if (!event.altKey && (event.key === 'ArrowUp' || event.key === 'ArrowDown')) {
    const index = rows.value.findIndex((row) => row.kind === 'issue' && row.issue.id === issue.id);
    const direction = event.key === 'ArrowDown' ? 1 : -1;
    let next = index + direction;
    while (rows.value[next]?.kind === 'header') next += direction;
    const target = rows.value[next];
    if (target?.kind === 'issue') {
      event.preventDefault();
      selectedId.value = target.issue.id;
      virtualizer.value.scrollToIndex(next);
      requestAnimationFrame(() =>
        scrollElement.value
          ?.querySelector<HTMLButtonElement>(`[data-issue-id="${target.issue.id}"]`)
          ?.focus(),
      );
    }

    return;
  }

  if (props.grouped && props.groupBy !== 'status') return;
  if (!event.altKey || !event.key.startsWith('Arrow')) return;
  if (!props.grouped && ['ArrowLeft', 'ArrowRight'].includes(event.key)) return;

  const group = groups.value.find((item) => item.items.some((entry) => entry.id === issue.id));
  if (!group) return;

  const current = group.items.findIndex((item) => item.id === issue.id);
  const statusIndex = issues.statusGroups.indexOf(issue.status);
  const nextStatus = ['ArrowLeft', 'ArrowRight'].includes(event.key)
    ? issues.statusGroups[statusIndex + (event.key === 'ArrowRight' ? 1 : -1)]
    : issue.status;
  if (props.grouped && !nextStatus) return;

  const target =
    nextStatus === issue.status || !props.grouped
      ? current + (event.key === 'ArrowDown' ? 1 : -1)
      : (groups.value.find((item) => item.name === nextStatus)?.items.length ?? 0);
  if (
    (nextStatus === issue.status || !props.grouped) &&
    (target < 0 || target >= group.items.length)
  )
    return;

  event.preventDefault();
  if (props.grouped) {
    collapsed.value = collapsed.value.filter((status) => status !== nextStatus);
    emit('move', issue.id, 'status', nextStatus!, target);
  } else emit('reorder', issue.id, target);

  await nextTick();
  const rowIndex = rows.value.findIndex((row) => row.kind === 'issue' && row.issue.id === issue.id);
  if (rowIndex >= 0) virtualizer.value.scrollToIndex(rowIndex);
  requestAnimationFrame(() =>
    scrollElement.value?.querySelector<HTMLButtonElement>(`[data-issue-id="${issue.id}"]`)?.focus(),
  );
}
</script>

<template>
  <div ref="scrollElement" class="h-full min-h-0 overflow-auto" aria-label="Issue list">
    <IssueTableHeader
      v-if="items.length"
      :columns="tableColumns"
      :widths="columnWidths"
      :sort-field="sortField"
      :sort-direction="sortDirection"
      @resize="(column, width) => emit('resizeColumn', column, width)"
      @sort="emit('sortColumn', $event)"
    />
    <div
      v-if="!items.length"
      class="flex min-h-48 flex-col items-center justify-center gap-2 px-4 py-8 text-sm text-foreground-muted"
    >
      <Icon name="i-tabler-search-off" class="size-5" />
      <p>No issues found</p>
    </div>
    <div v-else class="relative mx-2" :style="{ height: `${virtualizer.getTotalSize()}px` }">
      <div
        v-for="entry in visibleRows"
        :key="String(entry.item.key)"
        class="absolute top-0 left-0 w-full"
        :style="{
          height: `${entry.item.size}px`,
          transform: `translateY(${entry.item.start}px)`,
        }"
        :draggable="entry.row.kind === 'issue' && (!grouped || groupBy === 'status')"
        @dragstart="entry.row.kind === 'issue' && startDrag($event, entry.row.issue.id)"
        @dragend="draggingId = null"
        @dragover.prevent
        @drop="dropOn($event, entry.row)"
      >
        <template v-if="entry.row.kind === 'header'">
          <button
            type="button"
            :aria-expanded="!collapsed.includes(entry.row.name)"
            class="mt-1 flex h-8 w-full items-center gap-2 rounded-md bg-muted/65 px-2 text-left text-xs font-medium text-foreground-highlighted hover:bg-muted focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-inverted"
            @click="toggleGroup(entry.row.name)"
          >
            <Icon
              :name="
                collapsed.includes(entry.row.name)
                  ? 'i-tabler-chevron-right'
                  : 'i-tabler-chevron-down'
              "
              class="mr-1 size-3 text-foreground-dimmed"
            />
            <IssueStatusIcon v-if="groupBy === 'status'" :status="entry.row.name as Status" />
            <Icon
              v-else
              :name="groupBy === 'project' ? 'i-tabler-cube' : 'i-tabler-antenna-bars-5'"
              class="size-4 text-foreground-muted"
            />
            <span>{{ entry.row.name }}</span>
            <span class="font-normal tabular-nums text-foreground-muted">{{
              entry.row.count
            }}</span>
          </button>
        </template>
        <IssueListRow
          v-else-if="entry.row.kind === 'issue'"
          :issue="entry.row.issue"
          :selected="selectedId === entry.row.issue.id"
          :show-priority="showPriority"
          :show-project="showProject"
          :columns="columns"
          :column-widths="columnWidths"
          :density="density"
          :bulk-selectable="bulkSelectable"
          :bulk-selected="bulkSelectedIds?.includes(entry.row.issue.id)"
          @open="selectedId = entry.row.issue.id"
          @move="moveWithKeyboard($event, entry.row.issue)"
          @toggle-bulk="emit('toggleBulk', entry.row.issue.id)"
        />
      </div>
    </div>
  </div>
</template>
