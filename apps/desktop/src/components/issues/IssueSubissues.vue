<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { listIssueChildren } from '../../lib/issue-storage';
import { usesNativeStorage } from '../../lib/workspace-storage';
import { useIssuesStore } from '../../stores/issues';
import type { Issue } from '../../types/issue';
import { Button } from '../ui/button';

interface Props {
  issue: Issue;
}

const { issue } = defineProps<Props>();
const emit = defineEmits<{ add: []; open: [id: string] }>();
const issues = useIssuesStore();
const nativeChildren = ref<Issue[]>([]);
const childCount = ref(0);
const childCursor = ref<string | null>(null);
const loadingChildren = ref(false);
let childRevision = 0;

const subIssues = computed(() => {
  const cached = issues.items.filter((item) => item.parentId === issue.id);
  if (!usesNativeStorage) return cached;
  return [...new Map([...nativeChildren.value, ...cached].map((item) => [item.id, item])).values()];
});

async function loadChildren(more = false) {
  if (!usesNativeStorage || (more && loadingChildren.value)) return;
  const revision = childRevision;
  loadingChildren.value = true;
  try {
    const page = await listIssueChildren(
      issue.id,
      more ? (childCursor.value ?? undefined) : undefined,
    );
    if (revision !== childRevision) return;
    nativeChildren.value = more ? [...nativeChildren.value, ...page.items] : page.items;
    childCount.value = page.totalCount;
    childCursor.value = page.nextCursor;
  } finally {
    loadingChildren.value = false;
  }
}

watch(
  () => issue.id,
  () => {
    childRevision++;
    nativeChildren.value = [];
    childCount.value = 0;
    childCursor.value = null;
    void loadChildren();
  },
  { immediate: true },
);
</script>

<template>
  <div class="mt-5 border-t border-outline pt-3">
    <div class="mb-2 flex items-center justify-between gap-2">
      <h3 class="text-xs font-medium text-foreground-muted">
        Sub-issues
        <span v-if="subIssues.length">{{
          usesNativeStorage ? Math.max(childCount, subIssues.length) : subIssues.length
        }}</span>
      </h3>
      <Button
        label="Add sub-issue"
        aria-label="Add sub-issue"
        icon="i-tabler-plus"
        variant="ghost"
        size="xs"
        @click="emit('add')"
      />
    </div>
    <button
      v-for="child in subIssues"
      :key="child.id"
      type="button"
      class="flex min-h-8 w-full items-center gap-2 rounded-md px-1 text-left text-xs hover:bg-accented"
      @click="emit('open', child.id)"
    >
      <span class="shrink-0 text-foreground-muted">{{ child.id }}</span>
      <span class="min-w-0 truncate text-foreground">{{ child.title }}</span>
    </button>
    <Button
      v-if="usesNativeStorage && nativeChildren.length < childCount"
      label="Load more sub-issues"
      variant="ghost"
      size="xs"
      :disabled="loadingChildren"
      @click="loadChildren(true)"
    />
  </div>
</template>
