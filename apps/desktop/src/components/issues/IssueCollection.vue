<script setup lang="ts">
import { motion } from 'motion-v';
import { useEventListener } from '@vueuse/core';
import { storeToRefs } from 'pinia';
import { computed, defineAsyncComponent, ref } from 'vue';
import { useRoute } from 'vue-router';
import type { StatusColor } from '../../lib/issue-presentation';
import type { WorkflowCategory } from '../../types/workflow';
import { useIssuesStore } from '../../stores/issues';
import { useIssueTableSettings } from '../../composables/useIssueTableSettings';
import { useIssueFilterModels } from '../../composables/useIssueFilterModels';
import { useIssueFilterOptions } from '../../composables/useIssueFilterOptions';
import { useIssueCollectionQuery } from '../../composables/useIssueCollectionQuery';
import { useIssueCollectionActions } from '../../composables/useIssueCollectionActions';
import { usePreferencesStore } from '../../stores/preferences';
import { useUiStore } from '../../stores/ui';
import { useViewsStore } from '../../stores/views';
import type { Issue, Project } from '../../types/issue';
import { usesNativeStorage } from '../../lib/workspace-storage';
import IssueActiveFilterChips from './IssueActiveFilterChips.vue';
import IssueCollectionToolbar from './IssueCollectionToolbar.vue';
import IssueBoard from './IssueBoard.vue';
import IssueContextMenu from './IssueContextMenu.vue';
import IssueHierarchyActionDialog from './IssueHierarchyActionDialog.vue';
import IssueTriageDecisionDialog from './IssueTriageDecisionDialog.vue';
import IssueList from './IssueList.vue';
import IssueVirtualList from './IssueVirtualList.vue';
import IssueOverviewPanel from './IssueOverviewPanel.vue';
import ViewSettingsDialog from '../views/ViewSettingsDialog.vue';
import { Button } from '../ui/button';

interface Props {
  title: string;
  description: string;
  triage?: boolean;
  project?: Project;
  viewId?: string;
}

const props = defineProps<Props>();

const IssueDetail = defineAsyncComponent(() => import('./IssueDetail.vue'));

const route = useRoute();
const issues = useIssuesStore();
const ui = useUiStore();
const views = useViewsStore();

const {
  issueScope,
  issueLayout,
  boardGroupBy,
  groupIssuesByStatus,
  showIssuePriority,
  showIssueProject,
} = storeToRefs(usePreferencesStore());

const currentView = computed(() => views.items.find((view) => view.id === props.viewId));
const viewColumns = computed(() => currentView.value?.columns);
const issueTableSettings = useIssueTableSettings(currentView);
const { tableWidths, tableSortField, tableSortDirection, resizeTableColumn, sortTableColumn } =
  issueTableSettings;

const viewDensity = computed(() => currentView.value?.density ?? 'default');
const showEmptyGroups = computed(() => currentView.value?.showEmptyGroups ?? true);

const scope = computed<'active' | 'all'>({
  get: () => currentView.value?.scope ?? issueScope.value,
  set: (value) =>
    currentView.value
      ? views.update(currentView.value.id, { scope: value })
      : (issueScope.value = value),
});

const layout = computed<'list' | 'board'>({
  get: () => currentView.value?.layout ?? issueLayout.value,
  set: (value) =>
    currentView.value
      ? views.update(currentView.value.id, { layout: value })
      : (issueLayout.value = value),
});

const grouping = computed<'none' | 'status' | 'priority' | 'project'>({
  get: () => currentView.value?.groupBy ?? boardGroupBy.value,
  set: (value) =>
    currentView.value
      ? views.update(currentView.value.id, { groupBy: value })
      : (boardGroupBy.value = value === 'status' || value === 'priority' ? value : 'status'),
});

const listGrouped = computed({
  get: () => (currentView.value ? grouping.value !== 'none' : groupIssuesByStatus.value),
  set: (value: boolean) => {
    if (currentView.value) grouping.value = value ? 'status' : 'none';
    else groupIssuesByStatus.value = value;
  },
});

const search = ref('');
const filtersOpen = ref(false);
const displayOpen = ref(false);
const overviewOpen = ref(false);
const editViewOpen = ref(false);
const useIssueFilterModelsResult = useIssueFilterModels(currentView, scope, search);
const { statusFilter, excludedStatuses, projectFilter, clearFilters } = useIssueFilterModelsResult;

const issueFilterOptionsResult = useIssueFilterOptions(
  useIssueFilterModelsResult,
  currentView,
  computed(() => props.project),
  search,
);
const { unavailableLabels, invalidTreeValues } = issueFilterOptionsResult;

const selectedIssue = computed(
  () => issues.items.find((issue) => issue.id === (route.params.id ?? route.query.issue)) ?? null,
);

const isTriage = computed(
  () => props.triage || (route.name === 'issue' && !!selectedIssue.value?.triage),
);

function showCompletedStatus(status: string) {
  if (scope.value === 'active' && ['Completed', 'Canceled'].includes(issues.categoryFor(status)))
    scope.value = 'all';
}

const {
  nativeIssueIds,
  nativeTotalCount,
  nativeLoadedLimit,
  nativeLoadingMore,
  nativeQueryError,
  nativeBoardLoading,
  projectGroupNames,
  nativeBoardCounts,
  refreshNativeQuery,
  loadMoreNativeBoard,
  loadMoreNativeIssues,
  visibleIssues,
} = useIssueCollectionQuery({
  currentView,
  filters: useIssueFilterModelsResult,
  table: issueTableSettings,
  search,
  project: computed(() => props.project),
  scope,
  isTriage,
  layout,
  grouping,
  unavailableLabels,
  invalidTreeValues,
});

const {
  contextIssue,
  selectedTriageIds,
  triageDialogOpen,
  triageDecision,
  archiveDialogOpen,
  pendingArchive,
  archiveDescendants,
  triageSelection,
  toggleTriageSelection,
  openTriageDecision,
  afterTriageDecision,
  selectedId,
  closeDetail,
  expandDetail,
  updateIssue,
  updateSelectedIssue,
  afterProjectMove,
  archiveSelectedIssue,
  confirmArchive,
  prepareContextMenu,
} = useIssueCollectionActions({
  selectedIssue,
  triagePage: computed(() => !!props.triage),
  refreshNativeQuery,
  showCompletedStatus,
});

function moveIssue(id: string, property: 'status' | 'priority', value: string, index: number) {
  if (property === 'status') showCompletedStatus(value);
  issues.moveOnBoard(id, property, value, index);
}

function createInGroup(property: 'status' | 'priority', value: string) {
  const matchingProject = visibleIssues.value.find((issue) => issue[property] === value)?.project;

  const fallbackProject = visibleIssues.value.find(
    (issue) => issue.project !== 'Unassigned',
  )?.project;

  ui.createIssueDefaults = {
    status: property === 'status' ? value : issues.defaultStatus,
    priority: property === 'priority' ? (value as Issue['priority']) : 'No priority',
    project: isTriage.value
      ? 'Unassigned'
      : (props.project ??
        projectFilter.value[0] ??
        matchingProject ??
        fallbackProject ??
        'Unassigned'),
  };

  if (property === 'status') showCompletedStatus(value);

  ui.createIssueOpen = true;
}

function addStatusGroup(
  name: string,
  icon: string,
  color: StatusColor,
  category: WorkflowCategory,
  description: string,
) {
  issues.addStatusGroup(name, { icon, color }, category, description);
}

function editStatusGroup(
  name: string,
  nextName: string,
  icon: string,
  color: StatusColor,
  category: WorkflowCategory,
  description: string,
) {
  if (!issues.customizeStatusGroup(name, nextName, { icon, color }, category, description)) return;

  if (statusFilter.value.includes(name))
    statusFilter.value = statusFilter.value.map((status) => (status === name ? nextName : status));
  if (excludedStatuses.value.includes(name))
    excludedStatuses.value = excludedStatuses.value.map((status) =>
      status === name ? nextName : status,
    );

  views.items.forEach((view) => {
    if (view.statuses?.includes(name))
      views.update(view.id, {
        statuses: view.statuses.map((status) => (status === name ? nextName : status)),
      });
    else if (view.status === name) views.update(view.id, { status: nextName });
    if (view.excludedStatuses?.includes(name))
      views.update(view.id, {
        excludedStatuses: view.excludedStatuses.map((status) =>
          status === name ? nextName : status,
        ),
      });
  });
}

function reorderIssue(id: string, index: number) {
  issues.reorder(
    id,
    index,
    visibleIssues.value.map((issue) => issue.id),
  );
}

function onKeydown(event: KeyboardEvent) {
  if (ui.createIssueOpen || event.defaultPrevented || event.key !== 'Escape') return;

  const target = event.target;
  if (
    target instanceof Element &&
    target.closest('[role="menu"], [role="listbox"], [role="dialog"]')
  )
    return;

  if (selectedIssue.value) {
    event.preventDefault();
    closeDetail();
  } else if (overviewOpen.value) {
    event.preventDefault();
    overviewOpen.value = false;
  }
}

useEventListener(window, 'keydown', onKeydown);

useEventListener(window, 'issue-collection-command', (event) => {
  const name = (event as CustomEvent<string>).detail;

  if (name === 'filter' || name === 'find') filtersOpen.value = true;

  if (name === 'clearFilters') clearFilters();

  if (name === 'display') displayOpen.value = true;

  if (name === 'toggleBoard') layout.value = layout.value === 'board' ? 'list' : 'board';

  if (name === 'board') layout.value = 'board';

  if (name === 'list') layout.value = 'list';

  if (name === 'active' || name === 'all') scope.value = name;

  if (name === 'overview') overviewOpen.value = !overviewOpen.value;

  if (name === 'archive' && selectedIssue.value) void archiveSelectedIssue(selectedIssue.value.id);
});

useEventListener(document, 'pointerdown', (event) => {
  if (!(event.target instanceof Element)) return;

  if (selectedIssue.value) {
    if (
      event.target.closest(
        '[data-issue-preview], [data-issue-id], [role="listbox"], [role="menu"], [role="dialog"], .akaza-popover-content, a[href]',
      )
    )
      return;
    closeDetail();
  } else if (
    overviewOpen.value &&
    !event.target.closest(
      '[data-issue-overview], [data-issue-overview-trigger], [role="listbox"], [role="menu"], [role="dialog"], .akaza-popover-content',
    )
  ) {
    overviewOpen.value = false;
  }
});
</script>

<template>
  <div class="relative flex min-h-0 flex-1 overflow-hidden bg-workspace">
    <main
      class="flex min-w-0 flex-1 flex-col overflow-hidden bg-default"
      :aria-label="isTriage ? 'Triage' : title"
    >
      <p class="sr-only">{{ description }}</p>
      <section class="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
        <IssueCollectionToolbar
          v-model:scope="scope"
          v-model:layout="layout"
          v-model:grouping="grouping"
          v-model:list-grouped="listGrouped"
          v-model:search="search"
          v-model:filters-open="filtersOpen"
          v-model:display-open="displayOpen"
          v-model:overview-open="overviewOpen"
          v-model:selected-triage-ids="selectedTriageIds"
          :current-view="currentView"
          :project="project"
          :is-triage="isTriage"
          :visible-issues="visibleIssues"
          :triage-selection="triageSelection"
          :filters="useIssueFilterModelsResult"
          :options="issueFilterOptionsResult"
          @decision="openTriageDecision"
          @edit-view="editViewOpen = true"
        />
        <IssueActiveFilterChips
          v-model:search="search"
          :filters="useIssueFilterModelsResult"
          :options="issueFilterOptionsResult"
          :project="project"
          @clear="clearFilters"
        />
        <p v-if="unavailableLabels.length" role="alert" class="px-3 pb-2 text-xs text-warning">
          Unavailable label filter: {{ unavailableLabels.join(', ') }}. Remove or restore the label
          to show issues.
        </p>
        <p v-if="invalidTreeValues.length" role="alert" class="px-3 pb-2 text-xs text-warning">
          Unavailable filter value: {{ invalidTreeValues.join(', ') }}. Restore or replace it in the
          view settings.
        </p>
        <p v-if="nativeQueryError" role="alert" class="px-3 pb-2 text-xs text-error">
          Issue search is unavailable. Showing local results.
        </p>
        <IssueContextMenu
          :issue="contextIssue"
          @open="selectedId = $event"
          @update="updateIssue"
          @archive="archiveSelectedIssue"
          @moved="afterProjectMove"
        >
          <motion.div
            layout-scroll
            :class="[
              'min-h-0 min-w-0 flex-1',
              layout === 'list' && visibleIssues.length > 200 ? 'overflow-hidden' : 'overflow-auto',
            ]"
            @contextmenu="prepareContextMenu"
            @keydown="prepareContextMenu"
          >
            <IssueList
              v-if="
                layout === 'list' &&
                visibleIssues.length <= 200 &&
                (!currentView || grouping === 'status')
              "
              v-model:selected-id="selectedId"
              :items="visibleIssues"
              :grouped="currentView ? grouping === 'status' : groupIssuesByStatus"
              :show-priority="showIssuePriority"
              :show-project="showIssueProject && !project && !currentView?.project"
              :columns="viewColumns"
              :column-widths="tableWidths"
              :sort-field="tableSortField"
              :sort-direction="tableSortDirection"
              :density="viewDensity"
              :show-empty-groups="showEmptyGroups"
              :bulk-selectable="isTriage"
              :bulk-selected-ids="selectedTriageIds"
              @move="moveIssue"
              @reorder="reorderIssue"
              @toggle-bulk="toggleTriageSelection"
              @resize-column="resizeTableColumn"
              @sort-column="sortTableColumn"
            />
            <IssueVirtualList
              v-else-if="layout === 'list'"
              v-model:selected-id="selectedId"
              :items="visibleIssues"
              :grouped="currentView ? grouping !== 'none' : groupIssuesByStatus"
              :group-by="currentView ? grouping : groupIssuesByStatus ? 'status' : 'none'"
              :show-priority="showIssuePriority"
              :show-project="showIssueProject && !project && !currentView?.project"
              :columns="viewColumns"
              :column-widths="tableWidths"
              :sort-field="tableSortField"
              :sort-direction="tableSortDirection"
              :density="viewDensity"
              :show-empty-groups="showEmptyGroups"
              :bulk-selectable="isTriage"
              :bulk-selected-ids="selectedTriageIds"
              @move="moveIssue"
              @reorder="reorderIssue"
              @toggle-bulk="toggleTriageSelection"
              @resize-column="resizeTableColumn"
              @sort-column="sortTableColumn"
            />
            <IssueBoard
              v-else
              v-model:selected-id="selectedId"
              :items="visibleIssues"
              :group-by="grouping"
              :show-priority="showIssuePriority"
              :show-project="showIssueProject"
              :columns="viewColumns"
              :density="viewDensity"
              :status-groups="issues.statusGroups"
              :archived-status-groups="issues.archivedStatusGroups"
              :status-appearance="issues.statusAppearance"
              :status-meta="issues.statusMeta"
              :priority-groups="issues.priorityGroupOrder"
              :project-groups="projectGroupNames"
              :status-usage="issues.counts?.statuses"
              :group-counts="nativeBoardCounts"
              :loading-groups="nativeBoardLoading"
              @load-more-group="loadMoreNativeBoard"
              @move="moveIssue"
              @create="createInGroup"
              @add-group="addStatusGroup"
              @edit-group="editStatusGroup"
              @archive-group="issues.archiveStatusGroup"
              @restore-group="issues.restoreStatusGroup"
              @delete-group="issues.deleteStatusGroup"
              @reorder-group="issues.reorderGroup"
              @set-default-group="issues.setDefaultStatusGroup"
            />
          </motion.div>
        </IssueContextMenu>
        <div
          v-if="
            usesNativeStorage &&
            layout === 'list' &&
            nativeIssueIds &&
            nativeLoadedLimit < nativeTotalCount
          "
          class="flex shrink-0 items-center justify-center border-t border-outline/50 p-2"
        >
          <Button
            size="sm"
            variant="ghost"
            :disabled="nativeLoadingMore"
            @click="loadMoreNativeIssues"
          >
            {{
              nativeLoadingMore
                ? 'Loading…'
                : `Load more issues (${nativeLoadedLimit} of ${nativeTotalCount})`
            }}
          </Button>
        </div>
      </section>
      <Transition name="issue-detail">
        <div
          v-if="overviewOpen && !selectedIssue"
          data-issue-overview
          class="absolute top-13 right-2 bottom-2 z-10 w-[min(22rem,calc(100%-1rem))]"
        >
          <IssueOverviewPanel :items="visibleIssues" />
        </div>
      </Transition>
    </main>
    <Transition name="issue-detail" appear>
      <div
        v-if="selectedIssue"
        :key="selectedIssue.id"
        data-issue-preview
        class="absolute top-2 right-2 bottom-2 left-2 z-20 overflow-hidden rounded-lg border border-outline shadow-xl md:left-auto md:w-120"
      >
        <IssueDetail
          :issue="selectedIssue"
          @close="closeDetail"
          @expand="expandDetail"
          @update="updateSelectedIssue"
          @archive="archiveSelectedIssue(selectedIssue.id)"
          @moved="afterProjectMove"
        />
      </div>
    </Transition>
    <ViewSettingsDialog v-if="currentView" v-model:open="editViewOpen" :view="currentView" />
    <IssueTriageDecisionDialog
      v-model:open="triageDialogOpen"
      :issues="triageSelection"
      :decision="triageDecision"
      @completed="afterTriageDecision"
    />
    <IssueHierarchyActionDialog
      v-model:open="archiveDialogOpen"
      :issue="pendingArchive"
      :descendants="archiveDescendants"
      action="archive"
      @confirm="confirmArchive"
    />
  </div>
</template>
