import { computed, ref, watch } from 'vue';
import type { ComputedRef } from 'vue';
import { useViewsStore } from '../stores/views';
import { useWorkspaceStore } from '../stores/workspace';
import type { SavedView } from '../types/view';
import { issueTableColumns } from '../components/issues/issue-table-columns';
import type { IssueTableColumn, IssueTableWidths } from '../components/issues/issue-table-columns';

type SortField = 'title' | 'status' | 'priority' | 'dueDate' | 'updatedAt';

interface LocalTableSettings {
  widths: IssueTableWidths;
  sortField?: SortField;
  sortDirection?: 'asc' | 'desc';
}

export function useIssueTableSettings(currentView: ComputedRef<SavedView | undefined>) {
  const views = useViewsStore();
  const workspace = useWorkspaceStore();
  const localTable = ref<LocalTableSettings>({ widths: {} });

  const tableWidths = computed(() => currentView.value?.columnWidths ?? localTable.value.widths);
  const tableSortField = computed(() => currentView.value?.sortField ?? localTable.value.sortField);
  const tableSortDirection = computed(
    () => currentView.value?.sortDirection ?? localTable.value.sortDirection,
  );

  watch(
    () => workspace.workspaceId,
    (id) => {
      if (!id) return;

      try {
        localTable.value = JSON.parse(localStorage.getItem(`issue-table:${id}`) ?? '{"widths":{}}');
      } catch {
        localTable.value = { widths: {} };
      }
    },
    { immediate: true },
  );

  function saveLocalTable() {
    if (workspace.workspaceId)
      localStorage.setItem(
        `issue-table:${workspace.workspaceId}`,
        JSON.stringify(localTable.value),
      );
  }

  function resizeTableColumn(column: IssueTableColumn, width: number) {
    const config = issueTableColumns[column];
    const bounded = Math.max(config.min, Math.min(config.max, width));
    const widths = { ...tableWidths.value, [column]: bounded };

    if (currentView.value) void views.update(currentView.value.id, { columnWidths: widths });
    else {
      localTable.value = { ...localTable.value, widths };
      saveLocalTable();
    }
  }

  function sortTableColumn(column: IssueTableColumn) {
    if (!['title', 'status', 'priority', 'dueDate', 'updatedAt'].includes(column)) return;

    const field = column as SortField;
    const direction =
      tableSortField.value === field && tableSortDirection.value === 'asc' ? 'desc' : 'asc';

    if (currentView.value)
      void views.update(currentView.value.id, { sortField: field, sortDirection: direction });
    else {
      localTable.value = { ...localTable.value, sortField: field, sortDirection: direction };
      saveLocalTable();
    }
  }

  return {
    localTable,
    tableWidths,
    tableSortField,
    tableSortDirection,
    resizeTableColumn,
    sortTableColumn,
  };
}
