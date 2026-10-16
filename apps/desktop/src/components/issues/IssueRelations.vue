<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { watchDebounced } from '@vueuse/core';
import { listIssueRecords, searchIssueIds } from '../../lib/issue-storage';
import { usesNativeStorage } from '../../lib/workspace-storage';
import { useIssuesStore } from '../../stores/issues';
import type { IssueRelation, IssueRelationKind } from '../../types/issue';
import { Button } from '../ui/button';
import { Dialog } from '../ui/dialog';
import { Icon } from '../ui/icon';
import { Input } from '../ui/input';
import { Listbox } from '../ui/listbox';

interface Props {
  issueId: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{ openIssue: [id: string] }>();

const issues = useIssuesStore();

const relations = computed(() => issues.relationsByIssue[props.issueId] ?? []);

const ancestorIds = computed(() => {
  const ids: string[] = [];
  const seen = new Set([props.issueId]);
  let parentId = issues.items.find((item) => item.id === props.issueId)?.parentId;

  while (parentId && !seen.has(parentId)) {
    ids.push(parentId);
    seen.add(parentId);
    parentId = issues.items.find((item) => item.id === parentId)?.parentId;
  }

  return ids;
});

const inheritedRelations = computed(() =>
  ancestorIds.value.flatMap((ancestorId) =>
    (issues.relationsByIssue[ancestorId] ?? [])
      .filter(
        (relation) =>
          relation.kind === 'blocks' && relationOther(relation, ancestorId) !== props.issueId,
      )
      .map((relation) => ({ relation, ancestorId })),
  ),
);

const relationKind = ref<IssueRelationKind>('related');
const relationDialogOpen = ref(false);
const relationSearch = ref('');
const saving = ref(false);
const error = ref('');
const searchInput = ref<{ inputRef?: HTMLInputElement } | null>(null);
const activeIndex = ref(0);
const searchedIssues = ref<typeof issues.items>([]);

const relationKinds = [
  { label: 'Related to', value: 'related', icon: 'i-tabler-link' },
  { label: 'Blocks', value: 'blocks', icon: 'i-tabler-hand-stop' },
  { label: 'Blocked by', value: 'blocked_by', icon: 'i-tabler-hand-stop' },
  { label: 'Duplicates', value: 'duplicates', icon: 'i-tabler-copy' },
];

const relationTargets = computed(() =>
  (usesNativeStorage && relationSearch.value.trim() ? searchedIssues.value : issues.items)
    .filter((item) => item.id !== props.issueId)
    .filter((item) =>
      `${item.id} ${item.title}`.toLowerCase().includes(relationSearch.value.trim().toLowerCase()),
    )
    .slice(0, 40),
);
const relationOptions = computed(() =>
  relationTargets.value.map((target) => ({
    value: target.id,
    label: `${target.id} ${target.title}`,
  })),
);

function selectTarget(
  value: string | number | Record<string, unknown> | (string | number | Record<string, unknown>)[],
) {
  if (typeof value === 'string' && !saving.value) void addRelation(value);
}

function highlightTarget(option?: { value?: unknown }) {
  const index = relationTargets.value.findIndex((target) => target.id === option?.value);
  if (index >= 0) activeIndex.value = index;
}

let searchRevision = 0;
watchDebounced(
  relationSearch,
  async (value) => {
    const revision = ++searchRevision;
    searchedIssues.value = [];
    if (!usesNativeStorage || !value.trim()) return;

    try {
      const ids = await searchIssueIds(value);
      const records = await listIssueRecords(ids);
      if (revision === searchRevision && value === relationSearch.value)
        searchedIssues.value = records;
    } catch (cause) {
      if (revision === searchRevision) error.value = String(cause);
    }
  },
  { debounce: 150 },
);

watch(
  () => [props.issueId, ...ancestorIds.value],
  (ids) => {
    error.value = '';
    void Promise.all(ids.map((id) => issues.loadRelations(id))).catch((cause) => {
      error.value = String(cause);
    });
  },
  { immediate: true },
);

watch(relationDialogOpen, async (open) => {
  if (!open) return;
  relationSearch.value = '';
  activeIndex.value = 0;
  await nextTick();
  searchInput.value?.inputRef?.focus();
});

watch(relationSearch, () => (activeIndex.value = 0));

function relationName(relation: IssueRelation) {
  if (relation.kind === 'blocks')
    return relation.sourceId === props.issueId ? 'Blocks' : 'Blocked by';
  return relation.kind === 'duplicates' ? 'Duplicates' : 'Related to';
}

function relationOther(relation: IssueRelation, issueId = props.issueId) {
  return relation.sourceId === issueId ? relation.targetId : relation.sourceId;
}

function openRelationDialog(kind: IssueRelationKind) {
  relationKind.value = kind;
  error.value = '';
  relationDialogOpen.value = true;
}

function onSearchKeydown(event: KeyboardEvent) {
  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault();
    const length = relationTargets.value.length;
    if (!length) return;
    activeIndex.value =
      (activeIndex.value + (event.key === 'ArrowDown' ? 1 : -1) + length) % length;
  } else if (event.key === 'Enter') {
    event.preventDefault();
    const target = relationTargets.value[activeIndex.value];
    if (target && !saving.value) void addRelation(target.id);
  }
}

async function addRelation(targetId: string) {
  saving.value = true;
  error.value = '';
  try {
    await issues.linkIssue(props.issueId, targetId, relationKind.value);
    relationDialogOpen.value = false;
  } catch (cause) {
    error.value = String(cause);
  } finally {
    saving.value = false;
  }
}

async function removeRelation(id: string) {
  error.value = '';
  try {
    await issues.unlinkIssue(props.issueId, id);
  } catch (cause) {
    error.value = String(cause);
  }
}
</script>

<template>
  <div class="mt-5 border-t border-outline pt-3" role="list" aria-label="Issue relations">
    <h3 class="mb-2 px-1 text-xs font-medium text-foreground-muted">Relations</h3>
    <div
      v-for="relation in relations"
      :key="relation.id"
      class="flex min-h-8 items-center gap-2 rounded-md px-1 text-xs"
      role="listitem"
    >
      <span class="w-20 shrink-0 text-foreground-muted">{{ relationName(relation) }}</span>
      <button
        type="button"
        class="min-w-0 flex-1 truncate text-left text-foreground hover:underline"
        @click="emit('openIssue', relationOther(relation))"
      >
        {{ relationOther(relation) }} ·
        {{
          issues.items.find((item) => item.id === relationOther(relation))?.title ??
          'Archived issue'
        }}
      </button>
      <Button
        icon="i-tabler-x"
        :aria-label="`Remove relation to ${relationOther(relation)}`"
        variant="ghost"
        size="xs"
        @click="removeRelation(relation.id)"
      />
    </div>

    <div
      v-for="{ relation, ancestorId } in inheritedRelations"
      :key="`${ancestorId}-${relation.id}`"
      class="flex min-h-8 items-center gap-2 rounded-md px-1 text-xs"
      role="listitem"
    >
      <span class="w-20 shrink-0 text-foreground-muted">{{
        relation.sourceId === ancestorId ? 'Blocks' : 'Blocked by'
      }}</span>
      <button
        type="button"
        class="min-w-0 flex-1 truncate text-left text-foreground hover:underline"
        @click="emit('openIssue', relationOther(relation, ancestorId))"
      >
        {{ relationOther(relation, ancestorId) }} ·
        {{
          issues.items.find((item) => item.id === relationOther(relation, ancestorId))?.title ??
          'Archived issue'
        }}
      </button>
      <span class="shrink-0 text-foreground-dimmed" :title="`Inherited from ${ancestorId}`"
        >via {{ ancestorId }}</span
      >
    </div>

    <div class="mt-2 flex flex-wrap items-center gap-1.5">
      <Button
        v-for="kind in relationKinds"
        :key="kind.value"
        :label="kind.label"
        :icon="kind.icon"
        :data-issue-relation-add="kind.value === 'related' ? '' : undefined"
        variant="ghost"
        size="xs"
        @click="openRelationDialog(kind.value as IssueRelationKind)"
      />
    </div>
    <p v-if="error" role="alert" class="px-1 text-xs text-error">{{ error }}</p>
  </div>

  <Dialog
    v-model:open="relationDialogOpen"
    title="Link issue"
    size="md"
    class="workbench-surface"
    :ui="{ header: 'sr-only', body: 'p-0', footer: 'hidden' }"
  >
    <template #body>
      <div class="flex items-center gap-2 px-3 pt-3">
        <span class="rounded-md bg-accented px-2 py-1 text-[11px] text-foreground-muted"
          >{{ relationKinds.find((kind) => kind.value === relationKind)?.label }} ·
          {{ issueId }}</span
        >
      </div>
      <Input
        ref="searchInput"
        v-model="relationSearch"
        icon="i-tabler-search"
        aria-label="Search issues for relation"
        placeholder="Search issues..."
        size="sm"
        class="w-full px-2 py-2"
        :ui="{ root: 'border-0 bg-transparent shadow-none focus:ring-0' }"
        role="combobox"
        aria-autocomplete="list"
        aria-controls="relation-results-content"
        :aria-activedescendant="
          relationTargets[activeIndex]
            ? `relation-results-content-option-${activeIndex}`
            : undefined
        "
        @keydown="onSearchKeydown"
      />
      <Listbox
        id="relation-results"
        aria-label="Matching issues"
        :options="relationOptions"
        :model-value="relationTargets[activeIndex]?.id ?? ''"
        :disabled="saving"
        :ui="{
          root: 'w-full',
          content:
            'max-h-[min(22rem,50vh)] min-w-0 overflow-y-auto rounded-none border-x-0 border-b-0 border-t border-outline bg-transparent p-1.5 shadow-none',
          option:
            'min-h-8 px-2 text-xs data-akaza-highlighted:bg-accented data-[state=checked]:bg-accented',
          empty: 'px-3 py-6 text-center text-xs text-foreground-muted',
        }"
        @value-change="selectTarget"
        @highlight-change="highlightTarget"
      >
        <template #option="{ value }">
          <Icon
            :name="
              issues.iconFor(
                relationTargets.find((target) => target.id === value)?.status ?? 'backlog',
              )
            "
            class="size-4 shrink-0 text-foreground-muted"
          />
          <span class="shrink-0 font-mono text-foreground-dimmed">{{ value }}</span>
          <span class="min-w-0 flex-1 truncate">{{
            relationTargets.find((target) => target.id === value)?.title
          }}</span>
          <span class="max-w-24 truncate text-foreground-dimmed">{{
            relationTargets.find((target) => target.id === value)?.project
          }}</span>
        </template>
        <template #empty>No matching issues</template>
      </Listbox>
      <p v-if="error" role="alert" class="border-t border-outline px-3 py-2 text-xs text-error">
        {{ error }}
      </p>
      <div
        class="flex items-center justify-between border-t border-outline px-3 py-2 text-xs text-foreground-muted"
      >
        <span>↑↓ Navigate <span class="mx-1">·</span> ↵ Link</span>
        <span><kbd class="font-medium">Esc</kbd> Close</span>
      </div>
    </template>
  </Dialog>
</template>
