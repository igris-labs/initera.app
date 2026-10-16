import { watchDebounced } from '@vueuse/core';
import { computed, ref, watch } from 'vue';
import type { ComputedRef } from 'vue';
import { listIssueRecords, searchIssueIds } from '../lib/issue-storage';
import { collectDescendantIds } from '../lib/issue-tree';
import { usesNativeStorage } from '../lib/workspace-storage';
import { useIssuesStore } from '../stores/issues';
import type { Issue } from '../types/issue';

export function useIssueParentPicker(issue: ComputedRef<Issue>) {
  const issues = useIssuesStore();
  const parentSearch = ref('');
  const parentError = ref('');
  const nativeParentRecords = ref<Issue[]>([]);
  let searchRevision = 0;

  watchDebounced(
    parentSearch,
    async (value) => {
      const revision = ++searchRevision;
      if (!usesNativeStorage || !value.trim()) {
        nativeParentRecords.value = [];
        return;
      }

      try {
        const ids = await searchIssueIds(value.trim());
        const records = await listIssueRecords(ids.slice(0, 40));
        if (revision === searchRevision) nativeParentRecords.value = records;
      } catch {
        if (revision === searchRevision) nativeParentRecords.value = [];
      }
    },
    { debounce: 150 },
  );

  watch(
    () => issue.value.parentId,
    async (id) => {
      if (!usesNativeStorage || !id || issues.items.some((item) => item.id === id)) return;
      nativeParentRecords.value = [...nativeParentRecords.value, ...(await listIssueRecords([id]))];
    },
    { immediate: true },
  );

  const parentIssue = computed(() =>
    [...issues.items, ...nativeParentRecords.value].find(
      (item) => item.id === issue.value.parentId,
    ),
  );

  const parentItems = computed(() => {
    const descendants = collectDescendantIds(issues.items, issue.value.id);
    return [
      { label: 'No parent', value: 'none', icon: 'i-tabler-minus' },
      ...[
        ...new Map(
          [...issues.items, ...nativeParentRecords.value].map((item) => [item.id, item]),
        ).values(),
      ]
        .filter((item) => !descendants.has(item.id))
        .map((item) => ({
          label: `${item.id} · ${item.title}`,
          value: item.id,
          icon: 'i-tabler-git-branch',
        })),
    ];
  });

  async function changeParent(value: string) {
    parentError.value = '';
    try {
      await issues.save(issue.value.id, { parentId: value === 'none' ? null : value });
    } catch (error) {
      parentError.value = String(error);
    }
  }

  return { parentSearch, parentError, parentIssue, parentItems, changeParent };
}
