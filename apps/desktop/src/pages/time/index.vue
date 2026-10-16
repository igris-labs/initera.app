<script setup lang="ts">
import { useEventListener, useIntervalFn, useNow, watchDebounced } from '@vueuse/core';
import { computed, onMounted, ref, watch } from 'vue';
import { RouterLink } from 'vue-router';
import { loadWorkspaceTimePage } from '../../lib/time-storage';
import type { WorkspaceTimeEntry } from '../../lib/time-storage';
import { usesNativeStorage } from '../../lib/workspace-storage';
import { useWorkspaceStore } from '../../stores/workspace';
import { Button } from '../../components/ui/button';
import { Icon } from '../../components/ui/icon';
import { Input } from '../../components/ui/input';

const workspace = useWorkspaceStore();
const now = useNow({ scheduler: (callback) => useIntervalFn(callback, 1000) });
const issueFilter = ref('');
const projectFilter = ref('');
const fromDate = ref('');
const throughDate = ref('');
const entries = ref<WorkspaceTimeEntry[]>([]);
const nextCursor = ref<string | null>(null);
const totalSeconds = ref(0);
const totalCount = ref(0);
const loading = ref(false);
const error = ref('');
let request = 0;

function boundary(date: string, nextDay = false): number | undefined {
  if (!date) return undefined;
  const value = new Date(`${date}T00:00:00`);
  if (Number.isNaN(value.getTime())) return undefined;
  if (nextDay) value.setDate(value.getDate() + 1);
  return Math.floor(value.getTime() / 1000);
}

async function refresh(more = false) {
  if (!usesNativeStorage || (more && !nextCursor.value)) return;
  const revision = ++request;
  loading.value = true;
  error.value = '';
  try {
    const page = await loadWorkspaceTimePage({
      issue: issueFilter.value.trim() || undefined,
      project: projectFilter.value.trim() || undefined,
      start: boundary(fromDate.value),
      end: boundary(throughDate.value, true),
      cursor: more ? (nextCursor.value ?? undefined) : undefined,
    });
    if (revision !== request) return;
    entries.value = more ? [...entries.value, ...page.items] : page.items;
    nextCursor.value = page.nextCursor;
    totalSeconds.value = page.totalSeconds;
    totalCount.value = page.totalCount;
  } catch (cause) {
    if (revision === request) error.value = String(cause);
  } finally {
    if (revision === request) loading.value = false;
  }
}

watchDebounced([issueFilter, projectFilter, fromDate, throughDate], () => void refresh(), {
  debounce: 250,
});

watch(
  () => workspace.workspaceId,
  () => {
    entries.value = [];
    nextCursor.value = null;
    void refresh();
  },
);

onMounted(() => void refresh());
useEventListener(window, 'timer-changed', () => void refresh());

function day(entry: WorkspaceTimeEntry) {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'full',
    timeZone: entry.timeZone ?? undefined,
  }).format(new Date(entry.startedAt * 1000));
}

const groups = computed(() => {
  const result = new Map<string, WorkspaceTimeEntry[]>();
  for (const entry of entries.value) {
    const key = day(entry);
    if (!result.has(key)) result.set(key, []);
    result.get(key)!.push(entry);
  }

  return [...result];
});

function duration(seconds: number) {
  const minutes = Math.floor(Math.max(0, seconds) / 60);
  return `${Math.floor(minutes / 60)}h ${String(minutes % 60).padStart(2, '0')}m`;
}

function entrySeconds(entry: WorkspaceTimeEntry) {
  return entry.stoppedAt === null
    ? Math.max(0, Math.floor(now.value.getTime() / 1000) - entry.startedAt)
    : entry.durationSeconds;
}
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
    <header class="flex min-h-11 items-center gap-2 border-b border-outline px-3">
      <h1 class="text-sm font-medium text-foreground-highlighted">Time tracking</h1>
      <span class="text-xs tabular-nums text-foreground-dimmed">{{ totalCount }} entries</span>
      <span class="ml-auto text-xs tabular-nums text-foreground-muted">{{
        duration(totalSeconds)
      }}</span>
      <Button
        label="Refresh time"
        icon="i-tabler-refresh"
        variant="ghost"
        size="xs"
        :disabled="loading"
        @click="refresh()"
      />
    </header>

    <div class="flex flex-wrap gap-2 border-b border-outline px-3 py-2">
      <Input
        v-model="issueFilter"
        aria-label="Filter by issue identifier"
        placeholder="Issue"
        size="sm"
        class="w-32"
      />
      <Input
        v-model="projectFilter"
        aria-label="Filter by project"
        placeholder="Project"
        size="sm"
        class="w-40"
      />
      <Input v-model="fromDate" aria-label="From date" type="date" size="sm" />
      <Input v-model="throughDate" aria-label="Through date" type="date" size="sm" />
    </div>

    <p v-if="error" role="alert" class="px-4 py-2 text-xs text-error">{{ error }}</p>
    <div v-if="groups.length" class="min-h-0 flex-1 overflow-auto px-3 py-2">
      <section v-for="[date, items] in groups" :key="date" class="mb-5">
        <h2 class="border-b border-outline px-2 py-2 text-xs font-medium text-foreground-muted">
          {{ date }}
        </h2>
        <ul class="divide-y divide-outline/60">
          <li
            v-for="entry in items"
            :key="entry.id"
            class="flex min-h-11 items-center gap-3 px-2 py-2 text-xs"
          >
            <RouterLink
              :to="{ name: 'issue-full', params: { id: entry.issueIdentifier } }"
              class="min-w-0 flex-1 truncate text-foreground-highlighted hover:underline"
            >
              <span class="mr-2 font-mono text-foreground-dimmed">{{ entry.issueIdentifier }}</span
              >{{ entry.issueTitle }}
            </RouterLink>
            <span class="hidden w-32 truncate text-foreground-muted sm:block">{{
              entry.project
            }}</span>
            <span class="hidden w-36 truncate text-foreground-dimmed md:block">{{
              entry.note
            }}</span>
            <span class="shrink-0 tabular-nums text-foreground-muted">{{
              duration(entrySeconds(entry))
            }}</span>
          </li>
        </ul>
      </section>
      <Button
        v-if="nextCursor"
        label="Load more entries"
        variant="ghost"
        size="sm"
        :disabled="loading"
        @click="refresh(true)"
      />
    </div>
    <div
      v-else-if="!loading"
      class="flex flex-1 flex-col items-center justify-center gap-2 text-center"
    >
      <Icon name="i-tabler-clock" class="size-7 text-foreground-dimmed" />
      <p class="text-sm font-medium text-foreground-highlighted">No time entries found</p>
    </div>
    <p v-if="loading" class="px-4 py-3 text-xs text-foreground-muted">Loading time…</p>
  </div>
</template>
