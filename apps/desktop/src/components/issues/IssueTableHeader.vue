<script setup lang="ts">
import { computed } from 'vue';
import { issueTableColumns, issueTableTemplate } from './issue-table-columns';
import type { IssueTableColumn, IssueTableWidths } from './issue-table-columns';

interface Props {
  columns: IssueTableColumn[];
  widths?: IssueTableWidths;
  sortField?: string;
  sortDirection?: 'asc' | 'desc';
}

const props = defineProps<Props>();
const emit = defineEmits<{
  resize: [column: IssueTableColumn, width: number];
  sort: [column: IssueTableColumn];
}>();

const template = computed(() => issueTableTemplate(props.columns, props.widths));
const sortable = new Set<IssueTableColumn>(['title', 'status', 'priority', 'dueDate', 'updatedAt']);

function resize(event: PointerEvent, column: IssueTableColumn) {
  event.preventDefault();
  const target = event.currentTarget as HTMLElement;
  const initial =
    target.parentElement?.getBoundingClientRect().width ?? issueTableColumns[column].width;
  const start = event.clientX;
  target.setPointerCapture(event.pointerId);

  function move(pointer: PointerEvent) {
    emit('resize', column, Math.round(initial + pointer.clientX - start));
  }

  function stop() {
    target.removeEventListener('pointermove', move);
    target.removeEventListener('pointerup', stop);
    target.removeEventListener('pointercancel', stop);
  }

  target.addEventListener('pointermove', move);
  target.addEventListener('pointerup', stop);
  target.addEventListener('pointercancel', stop);
}
</script>

<template>
  <div
    role="row"
    class="grid min-w-max border-b border-outline text-xs text-foreground-muted"
    :style="{ gridTemplateColumns: template }"
  >
    <div
      v-for="column in columns"
      :key="column"
      role="columnheader"
      :aria-sort="
        sortField === column ? (sortDirection === 'desc' ? 'descending' : 'ascending') : undefined
      "
      class="relative flex min-w-0 items-center px-3 py-2"
    >
      <button
        type="button"
        :aria-label="`Sort by ${issueTableColumns[column].label}`"
        class="flex min-w-0 items-center gap-1 truncate text-left hover:text-foreground disabled:cursor-default"
        :disabled="!sortable.has(column)"
        @click="emit('sort', column)"
      >
        {{ issueTableColumns[column].label }}
        <span v-if="sortField === column" aria-hidden="true">{{
          sortDirection === 'desc' ? '↓' : '↑'
        }}</span>
      </button>
      <span
        role="separator"
        tabindex="0"
        :aria-label="`Resize ${issueTableColumns[column].label} column`"
        aria-orientation="vertical"
        class="absolute top-0 right-0 h-full w-1 cursor-col-resize hover:bg-outline focus-visible:bg-outline"
        @pointerdown="resize($event, column)"
        @keydown.left.prevent="
          emit('resize', column, (widths?.[column] ?? issueTableColumns[column].width) - 16)
        "
        @keydown.right.prevent="
          emit('resize', column, (widths?.[column] ?? issueTableColumns[column].width) + 16)
        "
      />
    </div>
  </div>
</template>
