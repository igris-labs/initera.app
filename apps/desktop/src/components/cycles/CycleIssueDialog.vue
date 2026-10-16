<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import type { Cycle } from '../../lib/cycle-storage';
import type { Issue } from '../../types/issue';
import { listIssueRecords, searchIssueIds } from '../../lib/issue-storage';
import { usesNativeStorage } from '../../lib/workspace-storage';
import { Button } from '../ui/button';
import { Dialog } from '../ui/dialog';
import { Icon } from '../ui/icon';
import { Input } from '../ui/input';
import CycleDialogHeader from './CycleDialogHeader.vue';
import { cycleDialogUi } from './dialog-style';

interface Props {
  cycle: Cycle | null;
  issues: Issue[];
  projects: { id: string; name: string }[];
}

const { cycle, issues, projects } = defineProps<Props>();
const emit = defineEmits<{ toggle: [identifier: string] }>();
const open = defineModel<boolean>('open', { required: true });

const search = ref('');
const nativeResults = ref<Issue[]>([]);
let searchRevision = 0;
const searchInput = ref<{ inputRef?: HTMLInputElement } | null>(null);
const matches = computed(() =>
  (usesNativeStorage ? nativeResults.value : issues)
    .filter((issue) => !issue.triage)
    .filter((issue) => {
      if (!cycle?.projectIds?.length) return true;
      const project = projects.find((item) => item.name === issue.project);
      return project && cycle.projectIds.includes(project.id);
    })
    .filter((issue) =>
      `${issue.id} ${issue.title}`.toLowerCase().includes(search.value.toLowerCase()),
    )
    .slice(0, 60),
);
const title = computed(() => `Issues in ${cycle?.title ?? 'cycle'}`);

watch(open, (value) => {
  if (value) {
    search.value = '';
    nextTick(() => searchInput.value?.inputRef?.focus());
  }
});

watch(
  [open, search, () => cycle?.issueIds],
  async ([visible, term]) => {
    if (!usesNativeStorage || !visible) return;
    const revision = ++searchRevision;
    try {
      const ids = term.trim() ? await searchIssueIds(term) : (cycle?.issueIds ?? []);
      const records = await listIssueRecords(ids.slice(0, 60));
      if (revision === searchRevision) nativeResults.value = records;
    } catch {
      if (revision === searchRevision) nativeResults.value = [];
    }
  },
  { immediate: true },
);
</script>

<template>
  <Dialog
    v-model:open="open"
    :title="title"
    description="Search and select issues for this cycle."
    size="lg"
    class="workbench-surface"
    :ui="cycleDialogUi"
  >
    <template #header="{ close, titleId }">
      <CycleDialogHeader
        :title="title"
        label="Add issues"
        :title-id="titleId"
        :close="close"
        :cycle="cycle?.title"
      />
    </template>
    <template #body>
      <Input
        ref="searchInput"
        v-model="search"
        icon="i-tabler-search"
        aria-label="Search cycle issues"
        placeholder="Search issues..."
        variant="bare"
        size="md"
        :ui="{ root: 'border-0 bg-transparent text-sm focus-visible:outline-0' }"
      />
      <div class="mt-3 max-h-80 min-h-28 overflow-y-auto border-t border-outline py-1">
        <button
          v-for="issue in matches"
          :key="issue.id"
          type="button"
          class="flex min-h-9 w-full items-center gap-3 rounded-lg px-2 text-left text-xs hover:bg-accented focus-visible:bg-accented focus-visible:outline-none"
          :aria-pressed="cycle?.issueIds.includes(issue.id)"
          @click="emit('toggle', issue.id)"
        >
          <span class="w-20 shrink-0 font-mono text-foreground-muted">{{ issue.id }}</span>
          <span class="min-w-0 flex-1 truncate text-foreground-highlighted">{{ issue.title }}</span>
          <Icon
            v-if="cycle?.issueIds.includes(issue.id)"
            name="i-tabler-check"
            class="size-4 shrink-0 text-info"
          />
        </button>
        <p v-if="!matches.length" class="px-2 py-5 text-center text-xs text-foreground-muted">
          No matching issues
        </p>
      </div>
    </template>
    <template #footer>
      <span class="mr-auto text-xs text-foreground-muted"
        >{{ cycle?.issueIds.length ?? 0 }} selected</span
      >
      <Button label="Done" size="sm" @click="open = false" />
    </template>
  </Dialog>
</template>
