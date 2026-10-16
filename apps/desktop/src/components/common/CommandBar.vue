<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { useRoute } from 'vue-router';
import { shortcutHint } from '../../lib/app-shortcuts';
import {
  listIssueRecords,
  searchComments,
  searchIssueIds,
  searchPlanningNodes,
} from '../../lib/issue-storage';
import type { PlanningNodeSearchResult } from '../../lib/issue-storage';
import { loadProject, searchProjectIds, usesNativeStorage } from '../../lib/workspace-storage';
import { findCommands } from '../../lib/command-search';
import type { Command } from '../../lib/command-search';
import type { CommentSearchResult, Issue } from '../../types/issue';
import type { ProjectRecord } from '../../lib/workspace-storage';
import { useCyclesStore } from '../../stores/cycles';
import { useUiStore } from '../../stores/ui';
import { useCommandCatalog } from './useCommandCatalog';
import { Dialog } from '../ui/dialog';
import { Icon } from '../ui/icon';
import { Input } from '../ui/input';
import { Listbox } from '../ui/listbox';

const open = defineModel<boolean>('open', { required: true });

const route = useRoute();
const ui = useUiStore();
const cycles = useCyclesStore();

const query = ref('');
const nativeIssueMatches = ref<{ query: string; ids: string[] } | null>(null);
const nativeProjectMatches = ref<{ query: string; ids: string[] } | null>(null);
const nativeCommentMatches = ref<{ query: string; items: CommentSearchResult[] } | null>(null);
const nativeIssueRecords = ref<Issue[]>([]);
const nativeProjectRecords = ref<ProjectRecord[]>([]);
const nativePlanningMatches = ref<PlanningNodeSearchResult[]>([]);
const activeIndex = ref(0);
const input = ref<{ inputRef?: HTMLInputElement } | null>(null);

const { commands, mode, projectName, issueName, commentCommand } = useCommandCatalog(open, {
  nativeIssueRecords,
  nativeProjectRecords,
  nativePlanningMatches,
});

const results = computed(() => {
  const localResults = findCommands(commands.value, query.value, {
    mode: mode.value,
    routeName: String(route.name),
    projectName: projectName.value,
    issueName: issueName.value,
  });
  const trimmed = query.value.trim();
  let ordered = localResults;

  if (nativeIssueMatches.value?.query === trimmed && ['all', 'issues'].includes(mode.value)) {
    const issueCommands = new Map(
      commands.value
        .filter((command) => command.section === 'Issues')
        .map((command) => [command.id, command]),
    );
    const matched = nativeIssueMatches.value.ids
      .map((id) => issueCommands.get(`issue-${id}`))
      .filter((command): command is Command => Boolean(command));
    const seen = new Set(matched.map((command) => command.id));
    ordered = [
      ...ordered.filter((command) => command.section !== 'Issues'),
      ...matched,
      ...ordered.filter((command) => command.section === 'Issues' && !seen.has(command.id)),
    ];
  }

  if (nativeProjectMatches.value?.query === trimmed && ['all', 'projects'].includes(mode.value)) {
    const projectCommands = new Map(
      commands.value
        .filter((command) => command.section === 'Projects')
        .map((command) => [command.id, command]),
    );
    const matched = nativeProjectMatches.value.ids
      .map((id) => projectCommands.get(`project-${id}`))
      .filter((command): command is Command => Boolean(command));
    const seen = new Set(matched.map((command) => command.id));
    ordered = [
      ...ordered.filter((command) => command.section !== 'Projects'),
      ...matched,
      ...ordered.filter((command) => command.section === 'Projects' && !seen.has(command.id)),
    ];
  }

  if (nativeCommentMatches.value?.query === trimmed && ['all', 'issues'].includes(mode.value)) {
    const commentCommands = nativeCommentMatches.value.items.map(commentCommand);
    ordered = [...ordered.slice(0, 25), ...commentCommands, ...ordered.slice(25)];
  }

  if (trimmed && ['all', 'projects'].includes(mode.value) && nativePlanningMatches.value.length) {
    const nodeCommands = new Map(
      commands.value
        .filter((command) => command.section === 'Project structure')
        .map((command) => [command.id, command]),
    );
    ordered = [
      ...ordered.filter((command) => command.section !== 'Project structure'),
      ...nativePlanningMatches.value.flatMap((node) => {
        const command = nodeCommands.get(`node-${node.id}`);
        return command ? [command] : [];
      }),
    ];
  }

  return ordered.slice(0, 40);
});

const sections = computed(() => [...new Set(results.value.map((item) => item.section))]);
const resultOptions = computed(() =>
  sections.value.flatMap((section) => [
    ...(section === projectName.value || section === issueName.value
      ? []
      : [{ type: 'label' as const, label: section }]),
    ...results.value
      .filter((command) => command.section === section)
      .map((command) => ({ value: command.id, label: command.label })),
  ]),
);

function selectResult(
  value: string | number | Record<string, unknown> | (string | number | Record<string, unknown>)[],
) {
  const command = results.value.find((item) => item.id === value);
  if (command) void run(command);
}

function highlightResult(option?: { value?: unknown }) {
  const index = results.value.findIndex((item) => item.id === option?.value);
  if (index >= 0) activeIndex.value = index;
}

function resultFor(value: unknown) {
  return results.value.find((command) => command.id === value);
}

function resultShortcut(value: unknown) {
  const shortcut = resultFor(value)?.shortcut;
  return shortcut ? shortcutHint(shortcut) : '';
}

async function run(command: Command) {
  const keepOpen = [
    'keyboard-shortcuts',
    'open-command-bar',
    'search-workspace',
    'open-issue',
    'switch-workspace',
  ].includes(command.id);
  if (!keepOpen) {
    input.value?.inputRef?.blur();
    open.value = false;
  }

  query.value = '';
  await nextTick();
  // Let Akaza restore focus before the command opens another dialog or route.
  // Otherwise the closing command bar can reclaim focus from the new surface.
  if (!keepOpen) await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
  await command.run();
  if (keepOpen) await nextTick(() => input.value?.inputRef?.focus());
}

function onInputKeydown(event: KeyboardEvent) {
  if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
    event.preventDefault();
    open.value = false;
  } else if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault();
    activeIndex.value =
      (activeIndex.value + (event.key === 'ArrowDown' ? 1 : -1) + results.value.length) %
      (results.value.length || 1);
  } else if (event.key === 'Enter' && results.value[activeIndex.value]) {
    event.preventDefault();
    void run(results.value[activeIndex.value]);
  } else if (event.key === 'Backspace' && !query.value && mode.value !== 'all') {
    ui.commandBarMode = 'all';
  }
}

watch([open, mode], async ([value]) => {
  if (value) {
    void cycles.bootstrap();
    await nextTick();
    requestAnimationFrame(() => {
      if (open.value) input.value?.inputRef?.focus();
    });
  } else {
    query.value = '';
    ui.commandBarMode = 'all';
  }
});

watch([query, mode], () => {
  activeIndex.value = 0;
});

// Ignore native search responses that finish after a newer query has started.
let searchRevision = 0;

watch([query, mode, open], async ([text, currentMode, isOpen]) => {
  const revision = ++searchRevision;
  const trimmed = text.trim();
  nativeIssueMatches.value = null;
  nativeProjectMatches.value = null;
  nativeCommentMatches.value = null;
  nativeIssueRecords.value = [];
  nativeProjectRecords.value = [];
  nativePlanningMatches.value = [];
  if (!usesNativeStorage || !isOpen || !trimmed) return;

  const [issueResult, projectResult, commentResult, planningResult] = await Promise.allSettled([
    ['all', 'issues'].includes(currentMode) ? searchIssueIds(trimmed) : Promise.resolve([]),
    ['all', 'projects'].includes(currentMode) ? searchProjectIds(trimmed) : Promise.resolve([]),
    ['all', 'issues'].includes(currentMode) ? searchComments(trimmed) : Promise.resolve([]),
    ['all', 'projects'].includes(currentMode) ? searchPlanningNodes(trimmed) : Promise.resolve([]),
  ]);
  if (revision !== searchRevision) return;
  if (issueResult.status === 'fulfilled') {
    nativeIssueMatches.value = { query: trimmed, ids: issueResult.value };
    try {
      const records = await listIssueRecords(issueResult.value);
      if (revision === searchRevision) nativeIssueRecords.value = records;
    } catch {
      // Cached matches remain available when a record changes during search.
    }
  }

  if (projectResult.status === 'fulfilled') {
    nativeProjectMatches.value = { query: trimmed, ids: projectResult.value };
    const records = await Promise.allSettled(projectResult.value.map((id) => loadProject(id)));
    if (revision === searchRevision)
      nativeProjectRecords.value = records.flatMap((result) =>
        result.status === 'fulfilled' ? [result.value] : [],
      );
  }

  if (commentResult.status === 'fulfilled')
    nativeCommentMatches.value = { query: trimmed, items: commentResult.value };
  if (planningResult.status === 'fulfilled' && revision === searchRevision)
    nativePlanningMatches.value = planningResult.value;
});
</script>

<template>
  <Dialog
    v-model:open="open"
    title="Command bar"
    size="lg"
    class="command-bar-dialog workbench-surface max-w-2xl"
    :ui="{ header: 'sr-only', body: 'p-0', footer: 'hidden' }"
  >
    <template #body>
      <div
        v-if="projectName || issueName || mode !== 'all'"
        class="flex items-center gap-1.5 px-3 pt-3"
      >
        <span class="rounded-md bg-accented px-2 py-1 text-[11px] text-foreground-muted">
          {{
            mode === 'shortcuts'
              ? 'Keyboard shortcuts'
              : mode === 'issues'
                ? 'Issues'
                : mode === 'projects'
                  ? 'Projects'
                  : mode === 'workspaces'
                    ? 'Workspaces'
                    : projectName
                      ? `Project · ${projectName}`
                      : `Issue · ${issueName}`
          }}
        </span>
        <button
          v-if="mode !== 'all'"
          type="button"
          class="text-xs text-foreground-dimmed hover:text-foreground"
          aria-label="Clear command scope"
          @click="ui.commandBarMode = 'all'"
        >
          ×
        </button>
      </div>
      <Input
        ref="input"
        v-model="query"
        icon="i-tabler-search"
        aria-label="Search commands"
        :placeholder="
          mode === 'shortcuts'
            ? 'Search shortcuts...'
            : mode === 'issues'
              ? 'Search issues...'
              : mode === 'projects'
                ? 'Search projects...'
                : mode === 'workspaces'
                  ? 'Search workspaces...'
                  : 'Type a command or search...'
        "
        class="w-full px-2 py-2"
        :ui="{ root: 'border-0 bg-transparent shadow-none focus:ring-0' }"
        role="combobox"
        aria-autocomplete="list"
        aria-controls="command-results-content"
        :aria-activedescendant="
          results[activeIndex]
            ? `command-results-content-option-${resultOptions.findIndex((option) => 'value' in option && option.value === results[activeIndex]?.id)}`
            : undefined
        "
        @keydown="onInputKeydown"
      />
      <Listbox
        id="command-results"
        aria-label="Commands"
        :options="resultOptions"
        :model-value="results[activeIndex]?.id ?? ''"
        :ui="{
          root: 'w-full',
          content:
            'max-h-[min(26rem,55vh)] min-w-0 overflow-y-auto rounded-none border-x-0 border-b-0 border-t border-outline bg-transparent p-1.5 shadow-none',
          option:
            'min-h-8 px-2 text-xs data-akaza-highlighted:bg-accented data-[state=checked]:bg-accented',
          groupLabel: 'px-2 pt-2 pb-1 text-[11px] font-medium text-foreground-dimmed',
          empty: 'px-3 py-6 text-center text-xs text-foreground-muted',
        }"
        @value-change="selectResult"
        @highlight-change="highlightResult"
      >
        <template #option="{ value }">
          <Icon
            :name="resultFor(value)?.icon ?? 'i-tabler-search'"
            class="mr-2.5 size-4 shrink-0 text-foreground-muted"
          />
          <span
            v-if="resultFor(value)?.detail"
            class="mr-2 shrink-0 font-mono text-foreground-dimmed"
            >{{ resultFor(value)?.detail }}</span
          >
          <span class="min-w-0 flex-1 truncate">{{ resultFor(value)?.label }}</span>
          <kbd
            v-if="resultFor(value)?.shortcut"
            class="ml-auto shrink-0 rounded-md border border-outline bg-muted px-1.5 py-0.5 text-xs font-medium whitespace-nowrap text-foreground-highlighted"
            >{{ resultShortcut(value) }}</kbd
          >
        </template>
        <template #empty>No matching results</template>
      </Listbox>
      <div
        class="flex items-center justify-between border-t border-outline px-3 py-2 text-xs text-foreground-muted"
      >
        <span>↑↓ Navigate <span class="mx-2">·</span> ↵ Open</span>
        <span class="flex items-center gap-1.5"><kbd class="font-medium">Esc</kbd> Close</span>
      </div>
    </template>
  </Dialog>
</template>
