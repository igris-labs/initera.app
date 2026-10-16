<script setup lang="ts">
import { computed, ref } from 'vue';
import { useIssuesStore } from '../../stores/issues';
import { notifications } from '../../lib/notifications';
import { Button } from '../../components/ui/button';
import { Icon } from '../../components/ui/icon';
import IssueHierarchyActionDialog from '../../components/issues/IssueHierarchyActionDialog.vue';
import { getIssueDescendantCount } from '../../lib/issue-storage';
import type { ChildStrategy } from '../../lib/issue-storage';
import { usesNativeStorage } from '../../lib/workspace-storage';
import type { Issue } from '../../types/issue';

const issues = useIssuesStore();

const restoring = ref<string | null>(null);
const deleteDialogOpen = ref(false);
const deleting = ref<Issue | null>(null);
const deleteDescendants = ref<number>();

async function prepareDelete(issue: Issue) {
  deleting.value = issue;
  deleteDescendants.value = usesNativeStorage ? await getIssueDescendantCount(issue.id) : undefined;
  deleteDialogOpen.value = true;
}

const archived = computed(() =>
  [...issues.archivedItems].sort((a, b) => (b.archivedAt ?? '').localeCompare(a.archivedAt ?? '')),
);

async function restore(id: string) {
  restoring.value = id;
  try {
    await issues.restore(id);
    notifications.add({ title: 'Issue restored', description: id, type: 'success' });
  } catch (error) {
    notifications.add({
      title: 'Could not restore issue',
      description: String(error),
      type: 'error',
    });
  } finally {
    restoring.value = null;
  }
}

async function deleteIssue(strategy: ChildStrategy, targetId?: string) {
  if (!deleting.value) return;
  const id = deleting.value.id;
  try {
    await issues.deleteArchived(id, strategy, targetId);
    notifications.add({ title: 'Issue deleted', description: id, type: 'success' });
  } catch (error) {
    notifications.add({
      title: 'Could not delete issue',
      description: String(error),
      type: 'error',
    });
  } finally {
    deleting.value = null;
  }
}
</script>

<template>
  <main class="min-h-0 flex-1 overflow-auto bg-default" aria-label="Archived issues">
    <header class="flex min-h-11 items-center gap-2 border-b border-outline px-4">
      <Icon name="i-tabler-archive" class="size-4 text-foreground-muted" />
      <h1 class="text-sm font-medium">Archived issues</h1>
      <span class="text-xs tabular-nums text-foreground-dimmed">{{ archived.length }}</span>
    </header>
    <div
      v-if="!archived.length"
      class="flex min-h-48 flex-col items-center justify-center gap-2 text-sm text-foreground-muted"
    >
      <Icon name="i-tabler-archive" class="size-5" />
      <p>No archived issues</p>
    </div>
    <ul v-else class="divide-y divide-outline">
      <li
        v-for="issue in archived"
        :key="issue.id"
        class="flex min-h-11 items-center gap-3 px-4 text-sm"
      >
        <span class="w-19 shrink-0 font-mono text-xs text-foreground-muted">{{ issue.id }}</span>
        <span class="min-w-0 flex-1 truncate">{{ issue.title }}</span>
        <span class="hidden text-xs text-foreground-muted sm:block">{{ issue.project }}</span>
        <Button
          :label="issue.archiveRootId === issue.id ? 'Restore' : 'Restore as top-level'"
          icon="i-tabler-restore"
          variant="ghost"
          size="xs"
          :aria-label="
            issue.archiveRootId === issue.id
              ? `Restore ${issue.id}`
              : `Restore ${issue.id} as top-level`
          "
          :disabled="restoring !== null"
          @click="restore(issue.id)"
        />
        <Button
          label="Delete"
          icon="i-tabler-trash"
          variant="ghost"
          color="error"
          size="xs"
          :aria-label="`Delete ${issue.id}`"
          @click="prepareDelete(issue)"
        />
      </li>
    </ul>
    <div v-if="issues.archivedHasMore" class="flex justify-center p-3">
      <Button
        label="Load more archived issues"
        variant="ghost"
        size="sm"
        @click="issues.loadMoreArchived()"
      />
    </div>
    <IssueHierarchyActionDialog
      v-model:open="deleteDialogOpen"
      :issue="deleting"
      :descendants="deleteDescendants"
      action="delete"
      @confirm="deleteIssue"
    />
  </main>
</template>
