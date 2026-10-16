import { computed, nextTick, ref } from 'vue';
import type { ComputedRef } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { getIssueDescendantCount } from '../lib/issue-storage';
import type { ChildStrategy } from '../lib/issue-storage';
import { collectDescendantIds } from '../lib/issue-tree';
import { notifications } from '../lib/notifications';
import { usesNativeStorage } from '../lib/workspace-storage';
import { useIssuesStore } from '../stores/issues';
import type { Issue } from '../types/issue';

interface Options {
  selectedIssue: ComputedRef<Issue | null>;
  triagePage: ComputedRef<boolean>;
  refreshNativeQuery: () => void | Promise<void>;
  showCompletedStatus: (status: string) => void;
}

export function useIssueCollectionActions(options: Options) {
  const { selectedIssue, triagePage, refreshNativeQuery, showCompletedStatus } = options;
  const issues = useIssuesStore();
  const route = useRoute();
  const router = useRouter();
  const contextIssueId = ref<string | null>(null);
  const selectedTriageIds = ref<string[]>([]);
  const triageDialogOpen = ref(false);
  const triageDecision = ref<'accept' | 'decline'>('accept');
  const archiveDialogOpen = ref(false);
  const pendingArchive = ref<Issue | null>(null);
  const archiveDescendants = ref<number>();

  const contextIssue = computed(
    () => issues.items.find((issue) => issue.id === contextIssueId.value) ?? null,
  );

  const triageSelection = computed(() =>
    issues.items.filter((issue) => issue.triage && selectedTriageIds.value.includes(issue.id)),
  );

  function toggleTriageSelection(id: string) {
    selectedTriageIds.value = selectedTriageIds.value.includes(id)
      ? selectedTriageIds.value.filter((item) => item !== id)
      : [...selectedTriageIds.value, id];
  }

  function openTriageDecision(decision: 'accept' | 'decline', issue?: Issue) {
    if (issue) selectedTriageIds.value = [issue.id];
    if (!triageSelection.value.length) return;
    triageDecision.value = decision;
    triageDialogOpen.value = true;
  }

  function afterTriageDecision() {
    selectedTriageIds.value = [];
    if (usesNativeStorage) void refreshNativeQuery();
  }

  const selectedId = computed<string | null>({
    get: () => selectedIssue.value?.id ?? null,
    set: (id) => {
      if (!id) return;
      if (route.name === 'issues' || route.name === 'issue') {
        router.push({
          name: 'issue',
          params: { id },
          query: {
            from: route.name === 'issue' ? String(route.query.from || '/issues') : route.fullPath,
          },
        });
      } else {
        router.push({ path: route.path, query: { ...route.query, issue: id } });
      }
    },
  });

  function closeDetail() {
    const id = selectedId.value;
    const from =
      route.name === 'issue'
        ? typeof route.query.from === 'string' && route.query.from.startsWith('/')
          ? route.query.from
          : '/issues'
        : route.path;
    router
      .push(from)
      .then(() =>
        nextTick(() =>
          document.querySelector<HTMLButtonElement>(`[data-issue-id="${id}"]`)?.focus(),
        ),
      );
  }

  function expandDetail() {
    if (!selectedIssue.value) return;
    router.push({
      name: 'issue-full',
      params: { id: selectedIssue.value.id },
      query: { from: route.fullPath },
    });
  }

  function updateIssue(id: string, patch: Partial<Issue>) {
    const issue = issues.items.find((item) => item.id === id);
    if (issue?.triage && patch.triage === false) {
      openTriageDecision(patch.status === 'Canceled' ? 'decline' : 'accept', issue);
      return;
    }

    if (patch.status) showCompletedStatus(patch.status);

    issues.update(id, patch);

    if (selectedIssue.value?.id !== id) return;

    if (patch.triage === true && !triagePage.value) {
      router.replace({ path: '/triage', query: { issue: id } });
    } else if (patch.triage === false && triagePage.value) {
      router.replace({
        name: 'issue',
        params: { id },
        query: { from: '/issues' },
      });
    }
  }

  function updateSelectedIssue(patch: Partial<Issue>) {
    if (selectedIssue.value) updateIssue(selectedIssue.value.id, patch);
  }

  function afterProjectMove(previousId: string, nextId: string) {
    if (selectedId.value !== previousId) return;
    router.replace({ name: 'issue', params: { id: nextId }, query: route.query });
  }

  async function archiveSelectedIssue(id: string) {
    const issue = issues.items.find((item) => item.id === id);
    if (!issue) return;
    pendingArchive.value = issue;
    try {
      archiveDescendants.value = usesNativeStorage
        ? await getIssueDescendantCount(id)
        : collectDescendantIds(issues.items, id).size - 1;
    } catch (error) {
      notifications.add({
        title: 'Could not inspect sub-issues',
        description: String(error),
        type: 'error',
      });
      return;
    }

    if (archiveDescendants.value > 0) {
      archiveDialogOpen.value = true;
      return;
    }

    void confirmArchive('all');
  }

  async function confirmArchive(strategy: ChildStrategy, targetId?: string) {
    const id = pendingArchive.value?.id;
    if (!id) return;
    try {
      await issues.archive(id, strategy, targetId);
      if (selectedIssue.value?.id === id) await router.push('/issues');
      notifications.add({
        title: 'Issue archived',
        description: id,
        type: 'success',
      });
    } catch (error) {
      notifications.add({
        title: 'Could not archive issue',
        description: String(error),
        type: 'error',
      });
    }

    pendingArchive.value = null;
  }

  function prepareContextMenu(event: MouseEvent | KeyboardEvent) {
    if (
      event instanceof KeyboardEvent &&
      event.key !== 'ContextMenu' &&
      !(event.shiftKey && event.key === 'F10')
    )
      return;

    const target = event.target;

    const id =
      target instanceof Element
        ? target.closest<HTMLElement>('[data-issue-id]')?.dataset.issueId
        : undefined;

    if (!id) {
      event.stopPropagation();
      return;
    }

    contextIssueId.value = id;
  }

  return {
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
  };
}
