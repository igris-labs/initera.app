import { useDebounceFn } from '@vueuse/core';
import { computed, nextTick, ref, watch } from 'vue';
import type { ComputedRef } from 'vue';
import { useRoute } from 'vue-router';
import { listIssueRecords, queryIssueIdPage } from '../lib/issue-storage';
import { usesNativeStorage } from '../lib/workspace-storage';
import { useIssuesStore } from '../stores/issues';
import { useProjectsStore } from '../stores/projects';
import type { Issue, IssueQuery, Priority } from '../types/issue';

export function useNativeIssuePages(
  nativeQuery: ComputedRef<IssueQuery>,
  layout: ComputedRef<'list' | 'board'>,
  grouping: ComputedRef<'none' | 'status' | 'priority' | 'project'>,
  unavailableLabels: ComputedRef<string[]>,
  invalidTreeValues: ComputedRef<string[]>,
) {
  const route = useRoute();
  const issues = useIssuesStore();
  const projectStore = useProjectsStore();

  const nativeIssueIds = ref<string[] | null>(null);
  const nativeTotalCount = ref(0);
  const nativeIssueRecords = ref<Issue[]>([]);
  const nativeLoadedLimit = ref(0);
  const nativeListCursor = ref<string | null>(null);
  const nativeLoadingMore = ref(false);
  const nativeQueryError = ref('');
  const nativeBoardPages = ref<
    Record<
      string,
      { ids: string[]; records: Issue[]; totalCount: number; nextCursor: string | null }
    >
  >({});
  const nativeBoardLoading = ref<Record<string, boolean>>({});
  const projectGroupNames = computed(() => [
    ...new Set([
      ...projectStore.items.map((item) => item.name),
      ...projectStore.inactive.map((item) => item.name),
      ...Object.keys(issues.counts?.projects ?? {}),
      'Unassigned',
    ]),
  ]);
  const boardGroupNames = computed(() =>
    grouping.value === 'status'
      ? [...issues.statusGroups, ...issues.archivedStatusGroups]
      : grouping.value === 'priority'
        ? issues.priorityGroupOrder
        : grouping.value === 'project'
          ? projectGroupNames.value
          : ['All issues'],
  );
  const nativeBoardCounts = computed(() =>
    Object.fromEntries(
      Object.entries(nativeBoardPages.value).map(([name, page]) => [name, page.totalCount]),
    ),
  );

  // Ignore page responses from a query that has since changed.
  let queryRevision = 0;

  // Caching a page updates Pinia; that update must not start the same query again.
  let cachingNativeRecords = false;

  async function cacheNativeRecords(records: Issue[]) {
    cachingNativeRecords = true;
    issues.cacheRecords(records);
    await nextTick();
    cachingNativeRecords = false;
  }

  function boardQuery(group: string): IssueQuery | null {
    const query = nativeQuery.value;
    if (grouping.value === 'none') return query;
    if (grouping.value === 'project') {
      if (query.projects.length && !query.projects.includes(group)) return null;
      return { ...query, projects: [group] };
    }

    if (grouping.value === 'status') {
      if (query.statuses.length && !query.statuses.includes(group)) return null;
      return { ...query, statuses: [group] };
    }

    if (query.priorities.length && !query.priorities.includes(group as Priority)) return null;
    return { ...query, priorities: [group as Priority] };
  }

  function applyBoardPages() {
    const pages = boardGroupNames.value.map((name) => nativeBoardPages.value[name]).filter(Boolean);
    nativeIssueIds.value = pages.flatMap((page) => page.ids);
    nativeIssueRecords.value = pages.flatMap((page) => page.records);
    nativeLoadedLimit.value = nativeIssueIds.value.length;
    nativeTotalCount.value = pages.reduce((total, page) => total + page.totalCount, 0);
  }

  async function refreshNativeQuery() {
    if (!usesNativeStorage || !issues.ready) return;

    const revision = ++queryRevision;
    nativeIssueIds.value = null;
    nativeTotalCount.value = 0;
    nativeIssueRecords.value = [];
    nativeLoadedLimit.value = 0;
    nativeListCursor.value = null;
    nativeBoardPages.value = {};

    try {
      await issues.flushSaves();
      if (unavailableLabels.value.length || invalidTreeValues.value.length) {
        nativeIssueIds.value = [];
        nativeQueryError.value = '';
        return;
      }

      if (layout.value === 'board') {
        // Page each lane separately; a busy status must not consume the entire board window.
        const groups = await Promise.all(
          boardGroupNames.value.map(async (name) => {
            const query = boardQuery(name);
            if (!query)
              return [name, { ids: [], records: [], totalCount: 0, nextCursor: null }] as const;

            const page = await queryIssueIdPage(query, 40);
            const records = await listIssueRecords(page.ids);
            return [
              name,
              { ids: page.ids, records, totalCount: page.totalCount, nextCursor: page.nextCursor },
            ] as const;
          }),
        );
        if (revision !== queryRevision) return;

        nativeBoardPages.value = Object.fromEntries(groups);
        applyBoardPages();
        nativeQueryError.value = '';
        await cacheNativeRecords(nativeIssueRecords.value);
        return;
      }

      const page = await queryIssueIdPage(nativeQuery.value);
      const records = await listIssueRecords(page.ids);

      if (revision !== queryRevision) return;

      nativeIssueRecords.value = records;
      nativeLoadedLimit.value = page.ids.length;
      nativeListCursor.value = page.nextCursor;
      nativeIssueIds.value = page.ids;
      nativeTotalCount.value = page.totalCount;
      nativeQueryError.value = '';
      await cacheNativeRecords(records);
    } catch (error) {
      if (revision !== queryRevision) return;

      nativeIssueIds.value = null;
      nativeQueryError.value = String(error);
    }
  }

  async function loadMoreNativeBoard(group: string) {
    const current = nativeBoardPages.value[group];
    const query = boardQuery(group);
    if (!current || !query || nativeBoardLoading.value[group] || !current.nextCursor) return;

    const revision = queryRevision;
    nativeBoardLoading.value[group] = true;
    try {
      const page = await queryIssueIdPage(query, 40, current.nextCursor);
      const records = await listIssueRecords(page.ids);
      if (revision !== queryRevision) return;

      nativeBoardPages.value[group] = {
        ids: [...current.ids, ...page.ids],
        records: [...current.records, ...records],
        totalCount: page.totalCount,
        nextCursor: page.nextCursor,
      };
      applyBoardPages();
      await cacheNativeRecords(records);
    } catch (error) {
      // A cursor is tied to its query and workspace revision; restart after either changes.
      if (String(error).includes('Issue page cursor does not match this view'))
        void refreshNativeQuery();
      else nativeQueryError.value = String(error);
    } finally {
      nativeBoardLoading.value[group] = false;
    }
  }

  async function loadMoreNativeIssues() {
    const ids = nativeIssueIds.value;
    if (!ids || nativeLoadingMore.value || !nativeListCursor.value) return;

    const revision = queryRevision;
    nativeLoadingMore.value = true;
    try {
      const page = await queryIssueIdPage(nativeQuery.value, 200, nativeListCursor.value);
      const records = await listIssueRecords(page.ids);
      if (revision !== queryRevision) return;
      nativeIssueIds.value = [...ids, ...page.ids];
      nativeIssueRecords.value.push(...records);
      nativeLoadedLimit.value += page.ids.length;
      nativeTotalCount.value = page.totalCount;
      nativeListCursor.value = page.nextCursor;
      await cacheNativeRecords(records);
    } catch (error) {
      // The first page rebuilds the list after a concurrent edit invalidates its cursor.
      if (String(error).includes('Issue page cursor does not match this view'))
        void refreshNativeQuery();
      else nativeQueryError.value = String(error);
    } finally {
      nativeLoadingMore.value = false;
    }
  }

  // A project edit can change the query result without changing any selected filter.
  const projectFilterRevision = computed(() =>
    projectStore.items
      .map((project) =>
        [
          project.id,
          project.name,
          project.status,
          project.priority,
          project.leadActorId ?? '',
          ...(project.labels ?? []),
          ...(project.milestones ?? []).map((milestone) => `${milestone.id}:${milestone.title}`),
        ].join('\u0000'),
      )
      .join('\u0001'),
  );

  watch(
    [
      nativeQuery,
      () => issues.ready,
      projectFilterRevision,
      layout,
      grouping,
      boardGroupNames,
      unavailableLabels,
      invalidTreeValues,
    ],
    () => void refreshNativeQuery(),
    {
      immediate: true,
    },
  );

  watch(
    [() => route.params.id ?? route.query.issue, () => issues.ready],
    async ([identifier, ready]) => {
      if (!usesNativeStorage || !ready || typeof identifier !== 'string') return;
      if (issues.items.some((issue) => issue.id === identifier)) return;
      const record = await listIssueRecords([identifier]);
      if (record.length) await cacheNativeRecords(record);
    },
    { immediate: true },
  );

  if (usesNativeStorage) {
    const refreshAfterIssueChange = useDebounceFn(() => void refreshNativeQuery(), 100);
    issues.$subscribe(() => {
      if (!cachingNativeRecords) refreshAfterIssueChange();
    });
  }

  return {
    nativeIssueIds,
    nativeTotalCount,
    nativeIssueRecords,
    nativeLoadedLimit,
    nativeLoadingMore,
    nativeQueryError,
    nativeBoardLoading,
    projectGroupNames,
    nativeBoardCounts,
    refreshNativeQuery,
    loadMoreNativeBoard,
    loadMoreNativeIssues,
  };
}
