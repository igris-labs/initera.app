<script setup lang="ts">
import { LayoutGroup, motion } from 'motion-v';
import Sortable from 'sortablejs';
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import type { Issue, Status } from '../../types/issue';
import { useIssuesStore } from '../../stores/issues';
import IssueStatusIcon from './IssueStatusIcon.vue';
import IssueListRow from './IssueListRow.vue';
import IssueTableHeader from './IssueTableHeader.vue';
import { visibleIssueColumns } from './issue-table-columns';
import type { IssueTableColumn, IssueTableWidths } from './issue-table-columns';
import { Icon } from '../ui/icon';

interface Props {
  items: Issue[];
  grouped: boolean;
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

const collapsed = ref<Status[]>([]);
const root = ref<HTMLElement | null>(null);

const sortables: Sortable[] = [];
let suppressClickUntil = 0;

const groups = computed(() => {
  if (!props.grouped) return [{ name: 'Issues', items: props.items }];
  return issues.visibleStatusGroups
    .map((name) => ({
      name,
      items: props.items.filter((issue) => issue.status === name),
    }))
    .filter((group) => props.showEmptyGroups || group.items.length);
});
const tableColumns = computed(() =>
  visibleIssueColumns(props.columns, props.showPriority, props.showProject),
);

function toggleGroup(status: Status) {
  collapsed.value = collapsed.value.includes(status)
    ? collapsed.value.filter((item) => item !== status)
    : [...collapsed.value, status];
}

function onDrop(event: Sortable.SortableEvent) {
  const original = (
    event as Sortable.SortableEvent & {
      originalEvent?: MouseEvent | TouchEvent;
    }
  ).originalEvent;

  const pointer = original && 'changedTouches' in original ? original.changedTouches[0] : original;
  suppressClickUntil = performance.now() + 200;

  const id = (event.item as HTMLElement).dataset.sortId;
  const hovered =
    pointer && Number.isFinite(pointer.clientX)
      ? document
          .elementFromPoint(pointer.clientX, pointer.clientY)
          ?.closest<HTMLElement>('[data-list-group]')
      : null;
  const destination = hovered ?? (event.to as HTMLElement);
  const value = destination.dataset.listGroup;

  if (!id || !value) return;
  event.item.remove();
  event.from.insertBefore(event.item, event.from.children[event.oldIndex ?? 0] ?? null);

  const index =
    destination === event.to || !pointer
      ? (event.newDraggableIndex ?? 0)
      : [...destination.children].findIndex(
          (child) => pointer.clientY < child.getBoundingClientRect().top + child.clientHeight / 2,
        );

  const targetIndex = index < 0 ? destination.children.length : index;

  if (props.grouped) emit('move', id, 'status', value, targetIndex);
  else emit('reorder', id, targetIndex);
}

function setupSortables() {
  sortables.splice(0).forEach((item) => item.destroy());
  const easing = getComputedStyle(document.documentElement)
    .getPropertyValue('--ipex-ease-enter')
    .trim();
  root.value?.querySelectorAll<HTMLElement>('[data-list-group]').forEach((element) => {
    sortables.push(
      Sortable.create(element, {
        group: 'issues-list',
        draggable: '[data-sort-id]',
        delay: 140,
        touchStartThreshold: 4,
        forceFallback: true,
        fallbackOnBody: true,
        fallbackTolerance: 3,
        animation: 120,
        easing,
        ghostClass: 'opacity-40',
        chosenClass: 'ring-1',
        onEnd: onDrop,
      }),
    );
  });
}

onMounted(setupSortables);

watch(
  () =>
    `${props.grouped}:${props.items.length}:${groups.value.map((group) => group.name).join('|')}`,
  async () => {
    await nextTick();
    setupSortables();
  },
);

onBeforeUnmount(() => sortables.splice(0).forEach((item) => item.destroy()));

function openIssue(id: string) {
  if (performance.now() >= suppressClickUntil) selectedId.value = id;
}

async function moveWithKeyboard(event: KeyboardEvent, issue: Issue) {
  if (!event.altKey && (event.key === 'ArrowUp' || event.key === 'ArrowDown')) {
    const buttons = [...(root.value?.querySelectorAll<HTMLButtonElement>('[data-issue-id]') ?? [])];
    const index = buttons.findIndex((button) => button.dataset.issueId === issue.id);
    const next = buttons[index + (event.key === 'ArrowDown' ? 1 : -1)];
    if (next?.dataset.issueId) {
      event.preventDefault();
      selectedId.value = next.dataset.issueId;
      next.focus();
    }

    return;
  }

  if (!event.altKey || !event.key.startsWith('Arrow')) return;

  if (!props.grouped && (event.key === 'ArrowLeft' || event.key === 'ArrowRight')) return;

  const group = groups.value.find((item) => item.items.some((entry) => entry.id === issue.id));

  if (!group) return;

  const itemIndex = group.items.findIndex((item) => item.id === issue.id);
  const statusIndex = issues.statusGroups.indexOf(issue.status);
  const nextStatus =
    event.key === 'ArrowLeft' || event.key === 'ArrowRight'
      ? issues.statusGroups[statusIndex + (event.key === 'ArrowRight' ? 1 : -1)]
      : issue.status;

  if (props.grouped && !nextStatus) return;

  const targetIndex =
    nextStatus === issue.status || !props.grouped
      ? itemIndex + (event.key === 'ArrowDown' ? 1 : -1)
      : (groups.value.find((item) => item.name === nextStatus)?.items.length ?? 0);

  if (
    (nextStatus === issue.status || !props.grouped) &&
    (targetIndex < 0 || targetIndex >= group.items.length)
  )
    return;

  event.preventDefault();

  if (props.grouped) {
    collapsed.value = collapsed.value.filter((status) => status !== nextStatus);
    emit('move', issue.id, 'status', nextStatus!, targetIndex);
  } else emit('reorder', issue.id, targetIndex);

  await nextTick();

  root.value?.querySelector<HTMLButtonElement>(`[data-issue-id="${issue.id}"]`)?.focus();
}
</script>

<template>
  <div
    v-if="!items.length"
    class="flex min-h-48 flex-col items-center justify-center gap-2 px-4 py-8 text-sm text-foreground-muted"
  >
    <Icon name="i-tabler-search-off" class="size-5" />
    <p>No issues found</p>
  </div>
  <div v-else ref="root" :class="['min-w-0 overflow-x-auto', grouped ? 'space-y-2 px-2 pt-1' : '']">
    <IssueTableHeader
      :columns="tableColumns"
      :widths="columnWidths"
      :sort-field="sortField"
      :sort-direction="sortDirection"
      @resize="(column, width) => emit('resizeColumn', column, width)"
      @sort="emit('sortColumn', $event)"
    />
    <LayoutGroup>
      <section v-for="group in groups" :key="group.name" :aria-label="`${group.name} issues`">
        <h2 v-if="grouped">
          <button
            type="button"
            :aria-expanded="!collapsed.includes(group.name as Status)"
            class="flex h-8 w-full items-center gap-2 rounded-md bg-muted/65 px-2 text-left text-xs font-medium text-foreground-highlighted transition-colors duration-150 ease-out hover:bg-muted focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-inverted"
            @click="toggleGroup(group.name as Status)"
          >
            <Icon
              :name="
                collapsed.includes(group.name as Status)
                  ? 'i-tabler-chevron-right'
                  : 'i-tabler-chevron-down'
              "
              class="mr-1 size-3 text-foreground-dimmed"
            />
            <IssueStatusIcon :status="group.name as Status" />
            <span>{{ group.name }}</span>
            <span class="font-normal tabular-nums text-foreground-muted">
              {{ group.items.length }}
            </span>
          </button>
        </h2>
        <div
          :class="[
            'issue-group-body',
            {
              'is-collapsed': grouped && collapsed.includes(group.name as Status),
            },
          ]"
          :inert="grouped && collapsed.includes(group.name as Status)"
          :aria-hidden="grouped && collapsed.includes(group.name as Status)"
        >
          <div class="min-h-0 overflow-clip">
            <ul
              :data-list-group="group.name"
              aria-label="Issues"
              :class="grouped ? 'mt-1 space-y-0.5' : ''"
            >
              <motion.li
                v-for="issue in group.items"
                :key="issue.id"
                :data-sort-id="issue.id"
                layout="position"
                :layout-id="`list-${issue.id}`"
                :initial="false"
                :transition="{
                  layout: { type: 'spring', stiffness: 520, damping: 44 },
                }"
              >
                <IssueListRow
                  :issue="issue"
                  :selected="selectedId === issue.id"
                  :show-priority="showPriority"
                  :show-project="showProject"
                  :columns="columns"
                  :column-widths="columnWidths"
                  :density="density"
                  :bulk-selectable="bulkSelectable"
                  :bulk-selected="bulkSelectedIds?.includes(issue.id)"
                  @open="openIssue(issue.id)"
                  @move="moveWithKeyboard($event, issue)"
                  @toggle-bulk="emit('toggleBulk', issue.id)"
                />
              </motion.li>
            </ul>
          </div>
        </div>
      </section>
    </LayoutGroup>
  </div>
</template>
