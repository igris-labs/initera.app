<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import IssueDetail from '../../../components/issues/IssueDetail.vue';
import { useIssuesStore } from '../../../stores/issues';
import { notifications } from '../../../lib/notifications';
import { collectDescendantIds } from '../../../lib/issue-tree';
import { getIssueDescendantCount, listIssueRecords } from '../../../lib/issue-storage';
import type { ChildStrategy } from '../../../lib/issue-storage';
import { usesNativeStorage } from '../../../lib/workspace-storage';
import IssueHierarchyActionDialog from '../../../components/issues/IssueHierarchyActionDialog.vue';

const route = useRoute();
const router = useRouter();
const issues = useIssuesStore();

const issue = computed(() => issues.items.find((item) => item.id === route.params.id));
const archiveDialogOpen = ref(false);
const archiveDescendants = ref<number>();

watch(
  [() => route.params.id, () => issues.ready],
  async ([identifier, ready]) => {
    if (!usesNativeStorage || !ready || typeof identifier !== 'string' || issue.value) return;
    issues.cacheRecords(await listIssueRecords([identifier]));
  },
  { immediate: true },
);

function returnToPreview() {
  const from = route.query.from;
  if (issue.value?.triage) {
    router.push({ path: '/triage', query: { issue: route.params.id } });
    return;
  }

  if (typeof from === 'string' && from.startsWith('/triage')) {
    router.push({ name: 'issue', params: { id: route.params.id }, query: { from: '/issues' } });
    return;
  }

  if (typeof from === 'string' && from.startsWith('/') && !from.startsWith('//')) {
    router.push(from);
    return;
  }

  router.push({ name: 'issue', params: { id: route.params.id }, query: { from: '/issues' } });
}

async function archiveCurrent() {
  if (!issue.value) return;
  archiveDescendants.value = usesNativeStorage
    ? await getIssueDescendantCount(issue.value.id)
    : collectDescendantIds(issues.items, issue.value.id).size - 1;
  if (archiveDescendants.value > 0) {
    archiveDialogOpen.value = true;
    return;
  }

  void confirmArchive('all');
}

async function confirmArchive(strategy: ChildStrategy, targetId?: string) {
  if (!issue.value) return;
  try {
    const id = issue.value.id;
    await issues.archive(id, strategy, targetId);
    notifications.add({ title: 'Issue archived', description: id, type: 'success' });
    await router.push('/issues');
  } catch (error) {
    notifications.add({
      title: 'Could not archive issue',
      description: String(error),
      type: 'error',
    });
  }
}
</script>

<template>
  <IssueDetail
    v-if="issue"
    :issue="issue"
    expanded
    @close="returnToPreview"
    @update="issues.update(issue.id, $event)"
    @archive="archiveCurrent"
    @moved="
      (_previousId, nextId) =>
        router.replace({ name: 'issue-full', params: { id: nextId }, query: route.query })
    "
  />
  <div v-else class="flex flex-1 flex-col items-center justify-center gap-3 text-foreground-muted">
    <p>Issue not found</p>
    <button type="button" class="text-info hover:underline" @click="router.push('/issues')">
      Back to issues
    </button>
  </div>
  <IssueHierarchyActionDialog
    v-model:open="archiveDialogOpen"
    :issue="issue ?? null"
    :descendants="archiveDescendants"
    action="archive"
    @confirm="confirmArchive"
  />
</template>
