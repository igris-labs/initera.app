<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { listIssueRecords, searchIssueIds } from '../../lib/issue-storage';
import type { ChildStrategy } from '../../lib/issue-storage';
import { usesNativeStorage } from '../../lib/workspace-storage';
import { collectDescendantIds } from '../../lib/issue-tree';
import { useIssuesStore } from '../../stores/issues';
import type { Issue } from '../../types/issue';
import { Button } from '../ui/button';
import { Dialog } from '../ui/dialog';
import { Input } from '../ui/input';
import { RadioGroup } from '../ui/radio-group';

interface Props {
  issue: Issue | null;
  action: 'archive' | 'delete';
  descendants?: number;
}

const props = defineProps<Props>();
const emit = defineEmits<{ confirm: [strategy: ChildStrategy, targetId?: string] }>();
const open = defineModel<boolean>('open', { required: true });
const issues = useIssuesStore();
const strategy = ref<ChildStrategy>('all');
const search = ref('');
const targetId = ref<string>();

const tree = computed(() => {
  if (!props.issue) return new Set<string>();
  return collectDescendantIds([...issues.items, ...issues.archivedItems], props.issue.id);
});
const childCount = computed(() => props.descendants ?? Math.max(0, tree.value.size - 1));
const searchResults = ref<Issue[]>([]);
const destinations = computed(() =>
  (usesNativeStorage && search.value.trim() ? searchResults.value : issues.items)
    .filter((item) => !tree.value.has(item.id))
    .filter((item) => `${item.id} ${item.title}`.toLowerCase().includes(search.value.toLowerCase()))
    .slice(0, 30),
);

let searchRevision = 0;
watch(search, async (value) => {
  const revision = ++searchRevision;
  if (!usesNativeStorage || !value.trim()) {
    searchResults.value = [];
    return;
  }

  try {
    const ids = await searchIssueIds(value);
    const records = await listIssueRecords(ids.slice(0, 30));
    if (revision === searchRevision) searchResults.value = records;
  } catch {
    if (revision === searchRevision) searchResults.value = [];
  }
});
const choices = computed(() => [
  {
    value: 'all',
    label: props.action === 'archive' ? 'Archive all sub-issues' : 'Delete all sub-issues',
    description: 'Apply this action to the entire issue tree.',
  },
  {
    value: 'detach',
    label: 'Make sub-issues top-level',
    description: 'Keep the sub-issues open without a parent.',
  },
  {
    value: 'move',
    label: 'Move sub-issues to another issue',
    description: 'Keep the sub-issues open under a new parent.',
  },
]);

watch(open, (value) => {
  if (!value) return;
  strategy.value = 'all';
  targetId.value = undefined;
  search.value = '';
});

function confirm() {
  if (strategy.value === 'move' && !targetId.value) return;
  emit('confirm', strategy.value, targetId.value);
  open.value = false;
}
</script>

<template>
  <Dialog
    v-model:open="open"
    :title="`${action === 'archive' ? 'Archive' : 'Delete'} ${issue?.id ?? 'issue'}`"
    :description="
      action === 'delete'
        ? 'This permanently removes the archived issue.'
        : 'Choose what happens to its sub-issues.'
    "
    size="sm"
  >
    <template #body>
      <p class="mb-3 text-sm text-foreground-muted">
        {{
          childCount
            ? `${childCount} sub-issue${childCount === 1 ? '' : 's'} will be affected.`
            : 'This issue has no sub-issues.'
        }}
      </p>
      <RadioGroup
        v-if="childCount"
        :model-value="strategy"
        :options="choices"
        aria-label="Handle sub-issues"
        size="sm"
        :ui="{
          root: 'gap-1',
          item: 'min-h-12 w-full rounded-md px-2 py-2 text-left hover:bg-accented',
        }"
        @value-change="(value: string) => (strategy = value as ChildStrategy)"
      >
        <template #item="{ option, isChecked }">
          <span
            class="mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-full border border-outline-strong"
            aria-hidden="true"
          >
            <span v-if="isChecked" class="size-2 rounded-full bg-primary" />
          </span>
          <span class="min-w-0">
            <span class="block text-sm">{{ option.label }}</span>
            <span class="block text-xs text-foreground-muted">{{ option.description }}</span>
          </span>
        </template>
      </RadioGroup>
      <div v-if="childCount && strategy === 'move'" class="mt-3">
        <Input
          v-model="search"
          icon="i-tabler-search"
          placeholder="Find destination issue..."
          aria-label="Find destination issue"
          size="sm"
        />
        <div class="mt-1 max-h-40 overflow-auto">
          <button
            v-for="target in destinations"
            :key="target.id"
            type="button"
            class="flex min-h-8 w-full items-center gap-2 rounded-md px-2 text-left text-xs hover:bg-accented"
            :class="targetId === target.id ? 'bg-accented' : ''"
            @click="targetId = target.id"
          >
            <span class="font-mono text-foreground-muted">{{ target.id }}</span>
            <span class="truncate">{{ target.title }}</span>
            <span v-if="targetId === target.id" class="ml-auto text-primary">✓</span>
          </button>
          <p v-if="!destinations.length" class="p-2 text-xs text-foreground-muted">
            No available issues
          </p>
        </div>
      </div>
    </template>
    <template #footer>
      <Button label="Cancel" variant="ghost" size="sm" @click="open = false" />
      <Button
        :label="action === 'archive' ? 'Archive issue' : 'Delete permanently'"
        variant="solid"
        :color="action === 'delete' ? 'error' : 'neutral'"
        size="sm"
        :disabled="strategy === 'move' && !targetId"
        @click="confirm"
      />
    </template>
  </Dialog>
</template>
