<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useEventListener, useIntervalFn } from '@vueuse/core';
import { RouterLink } from 'vue-router';
import { useCyclesStore } from '../../stores/cycles';
import { useIssuesStore } from '../../stores/issues';
import { useProjectsStore } from '../../stores/projects';
import { notifications } from '../../lib/notifications';
import { localDate } from '../../lib/cycle-schedule';
import { formatCycleDateRange } from '../../lib/cycle-format';
import type { Cycle, CycleSettings, CycleTiming } from '../../lib/cycle-storage';
import { usesNativeStorage } from '../../lib/workspace-storage';
import CycleScheduleDialog from '../../components/cycles/CycleScheduleDialog.vue';
import CreateCycleDialog from '../../components/cycles/CreateCycleDialog.vue';
import CycleIssueDialog from '../../components/cycles/CycleIssueDialog.vue';
import CompleteCycleDialog from '../../components/cycles/CompleteCycleDialog.vue';
import { Button } from '../../components/ui/button';
import { Icon } from '../../components/ui/icon';
import { Select } from '../../components/ui/select';

const cycles = useCyclesStore();
const issues = useIssuesStore();
const projects = useProjectsStore();
const projectOptions = computed(() => projects.items.map(({ id, name }) => ({ id, name })));

const createOpen = ref(false);
const editOpen = ref(false);
const scheduleOpen = ref(false);
const assignOpen = ref(false);
const completeOpen = ref(false);
const selected = ref<Cycle | null>(null);
const saving = ref(false);
const selectedSeriesId = ref('all');
const scheduleTarget = ref<CycleSettings | null>(null);
const selectedSeries = computed(
  () => cycles.series.find((series) => series.id === selectedSeriesId.value) ?? null,
);
const visibleCycles = computed(() =>
  cycles.items.filter(
    (cycle) => selectedSeriesId.value === 'all' || cycle.seriesId === selectedSeriesId.value,
  ),
);

const activeCycles = computed(() =>
  visibleCycles.value.filter((cycle) => cycles.statusOf(cycle) === 'active'),
);
const currentCycle = computed(() => activeCycles.value[0]);
const pendingCycles = computed(() =>
  visibleCycles.value.filter(
    (cycle) => cycle.status !== 'completed' && cycle.transitionResult === 'pending',
  ),
);
const upcomingCycles = computed(() =>
  visibleCycles.value.filter(
    (cycle) =>
      cycles.statusOf(cycle) === 'planned' &&
      !pendingCycles.value.some((pending) => pending.id === cycle.id),
  ),
);
const nextSeriesStart = computed(
  () =>
    cycles.items
      .filter(
        (cycle) =>
          cycle.seriesId === scheduleTarget.value?.id &&
          cycles.statusOf(cycle) === 'planned' &&
          cycle.startDate > localDate(),
      )
      .sort((a, b) => a.startDate.localeCompare(b.startDate))[0]?.startDate,
);
const pastCycles = computed(() =>
  visibleCycles.value.filter((cycle) => cycles.statusOf(cycle) === 'completed'),
);
const orderedCycles = computed(() => [
  ...activeCycles.value,
  ...[...pendingCycles.value].sort((a, b) => a.startDate.localeCompare(b.startDate)),
  ...[...upcomingCycles.value].sort((a, b) => a.startDate.localeCompare(b.startDate)),
  ...[...pastCycles.value].sort((a, b) => b.startDate.localeCompare(a.startDate)),
]);

onMounted(() => void cycles.bootstrap());
useIntervalFn(() => {
  if (usesNativeStorage) void cycles.refresh();
}, 30_000);
useIntervalFn(() => {
  if (cycles.reconcileProgress.length) void cycles.refresh();
}, 1_000);
useEventListener(window, 'create-cycle-command', () => (createOpen.value = true));

function issueFor(id: string) {
  return issues.items.find((issue) => issue.id === id);
}

function completedCount(cycle: Cycle) {
  if (cycle.status === 'completed' && cycle.completedCount != null) return cycle.completedCount;
  return cycle.issueIds.filter((id) => {
    const issue = issueFor(id);
    return issue && issues.categoryFor(issue.status) === 'Completed';
  }).length;
}

function progress(cycle: Cycle) {
  const total =
    cycle.status === 'completed'
      ? (cycle.totalCount ?? cycle.issueIds.length)
      : cycle.issueIds.length;
  return total ? Math.round((completedCount(cycle) / total) * 100) : 0;
}

async function update(cycle: Cycle) {
  saving.value = true;
  try {
    await cycles.update(cycle);
    editOpen.value = false;
    notifications.add({ title: 'Cycle saved', type: 'success' });
  } catch (error) {
    notifications.add({ title: 'Could not save cycle', description: String(error), type: 'error' });
  } finally {
    saving.value = false;
  }
}

async function saveSchedule(settings: CycleSettings) {
  saving.value = true;
  try {
    await cycles.configure(settings);
    selectedSeriesId.value = cycles.settings?.id ?? 'all';
    scheduleOpen.value = false;
    notifications.add({
      title: 'Cycles scheduled',
      description: 'Upcoming cycles are ready.',
      type: 'success',
    });
  } catch (error) {
    notifications.add({
      title: 'Could not set up cycles',
      description: String(error),
      type: 'error',
    });
  } finally {
    saving.value = false;
  }
}

async function toggleSeries() {
  if (!selectedSeries.value) return;
  const enabled = !selectedSeries.value.enabled;
  try {
    await cycles.setSeriesEnabled(selectedSeries.value.id, enabled);
    notifications.add({
      title: enabled ? 'Cycle scheduling enabled' : 'Cycle scheduling disabled',
      type: 'success',
    });
  } catch (error) {
    notifications.add({
      title: 'Could not change cycle series',
      description: String(error),
      type: 'error',
    });
  }
}

async function create(
  title: string,
  startDate: string,
  endDate: string,
  projectIds: string[],
  timing: CycleTiming,
) {
  saving.value = true;
  try {
    await cycles.create(title, startDate, endDate, projectIds, timing);
    createOpen.value = false;
    notifications.add({ title: 'Cycle created', type: 'success' });
  } catch (error) {
    notifications.add({
      title: 'Could not create cycle',
      description: String(error),
      type: 'error',
    });
  } finally {
    saving.value = false;
  }
}

async function toggleIssue(id: string) {
  if (!selected.value) return;
  try {
    await cycles.setIssue(selected.value.id, id, !selected.value.issueIds.includes(id));
  } catch (error) {
    notifications.add({
      title: 'Could not update cycle',
      description: String(error),
      type: 'error',
    });
  }
}

async function complete(nextCycleId?: string, selectedIdentifiers?: string[]) {
  if (!selected.value) return;
  saving.value = true;
  try {
    await cycles.complete(selected.value.id, nextCycleId, selectedIdentifiers);
    completeOpen.value = false;
    notifications.add({ title: 'Cycle completed', type: 'success' });
  } catch (error) {
    notifications.add({
      title: 'Could not complete cycle',
      description: String(error),
      type: 'error',
    });
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <main class="min-h-0 flex-1 overflow-auto bg-default" aria-label="Cycles">
    <header class="flex min-h-11 items-center justify-between gap-3 border-b border-outline px-4">
      <div class="flex items-center gap-2">
        <Icon name="i-tabler-calendar-time" class="size-4 text-foreground-muted" />
        <h1 class="text-sm font-medium">Cycles</h1>
      </div>
      <div class="flex items-center gap-2">
        <Select
          v-if="cycles.series.length"
          v-model="selectedSeriesId"
          :items="[
            { label: 'All cycles', value: 'all' },
            ...cycles.series.map((series) => ({ label: series.name, value: series.id })),
          ]"
          aria-label="Cycle series"
          size="xs"
        />
        <Button
          v-if="selectedSeries"
          label="Edit series"
          icon="i-tabler-calendar-repeat"
          variant="ghost"
          size="xs"
          @click="
            scheduleTarget = selectedSeries;
            scheduleOpen = true;
          "
        />
        <Button
          v-if="selectedSeries"
          :label="selectedSeries.enabled ? 'Disable series' : 'Enable series'"
          variant="ghost"
          size="xs"
          @click="toggleSeries"
        />
        <Button
          label="New series"
          icon="i-tabler-calendar-plus"
          variant="ghost"
          size="xs"
          @click="
            scheduleTarget = null;
            scheduleOpen = true;
          "
        />
        <Button label="New cycle" icon="i-tabler-plus" size="xs" @click="createOpen = true" />
      </div>
    </header>

    <p v-if="cycles.error" class="px-4 py-3 text-sm text-error" role="alert">{{ cycles.error }}</p>
    <div
      v-if="cycles.reconcileProgress.length"
      class="border-b border-outline px-4 py-2 text-xs text-foreground-muted"
      role="status"
    >
      Catching up cycles:
      <span v-for="item in cycles.reconcileProgress" :key="item.seriesId">
        {{ cycles.series.find((series) => series.id === item.seriesId)?.name ?? 'Series' }}
        {{ item.processed }} of {{ item.total }}
      </span>
    </div>
    <div
      v-else-if="!cycles.items.length"
      class="flex min-h-80 flex-col items-center justify-center gap-3 px-6 text-center"
    >
      <span
        class="flex size-10 items-center justify-center rounded-lg bg-muted text-foreground-muted"
        ><Icon name="i-tabler-calendar-repeat" class="size-5"
      /></span>
      <div>
        <h2 class="text-sm font-medium text-foreground-highlighted">Plan work in cycles</h2>
        <p class="mt-1 max-w-sm text-xs leading-5 text-foreground-muted">
          Choose a cadence and decide how unfinished work moves between cycles.
        </p>
      </div>
      <div class="flex items-center gap-2">
        <Button
          label="Set up cycles"
          size="sm"
          @click="
            scheduleTarget = null;
            scheduleOpen = true;
          "
        />
        <Button label="Create one cycle" variant="ghost" size="sm" @click="createOpen = true" />
      </div>
    </div>
    <div v-else class="mx-auto max-w-4xl px-4 pb-12">
      <div
        v-if="selectedSeries"
        class="flex items-center gap-2 border-b border-outline py-3 text-xs text-foreground-muted"
      >
        <Icon name="i-tabler-calendar-repeat" class="size-4" />
        <span>{{ selectedSeries.name }} · {{ selectedSeries.durationDays }}-day cycles</span>
        <span aria-hidden="true">·</span>
        <span>Every {{ selectedSeries.intervalDays }} days</span>
        <span aria-hidden="true">·</span>
        <span>{{ selectedSeries.enabled ? '12 upcoming' : 'Scheduling disabled' }}</span>
        <span aria-hidden="true">·</span>
        <span>{{ selectedSeries.projectIds?.length || projectOptions.length }} projects</span>
        <span aria-hidden="true">·</span>
        <span>{{
          selectedSeries.rolloverPolicy === 'ask'
            ? 'Ask before rollover'
            : selectedSeries.rolloverPolicy === 'leave'
              ? 'Leave unfinished work'
              : 'Move unfinished work'
        }}</span>
      </div>
      <section v-for="cycle in orderedCycles" :key="cycle.id" class="border-b border-outline py-4">
        <p
          v-if="
            cycle.id === currentCycle?.id ||
            pendingCycles.some((pending) => pending.id === cycle.id) ||
            cycle.id === upcomingCycles.at(-1)?.id ||
            cycle.id === pastCycles[0]?.id
          "
          class="mb-3 text-xs font-medium text-foreground-muted"
        >
          {{
            cycle.id === currentCycle?.id
              ? 'Current'
              : pendingCycles.some((pending) => pending.id === cycle.id)
                ? 'Needs rollover decision'
                : cycle.id === upcomingCycles.at(-1)?.id
                  ? 'Upcoming'
                  : 'Previous'
          }}
        </p>
        <div class="flex flex-wrap items-center justify-between gap-2">
          <div class="flex min-w-0 items-center gap-3">
            <Icon name="i-tabler-calendar-time" class="size-4 shrink-0 text-foreground-muted" />
            <div class="min-w-0">
              <h2 class="truncate text-sm font-medium">{{ cycle.title }}</h2>
              <p
                v-if="cycle.description"
                class="mt-0.5 max-w-xl truncate text-xs text-foreground-muted"
              >
                {{ cycle.description }}
              </p>
              <p class="mt-0.5 text-xs text-foreground-muted">
                {{ formatCycleDateRange(cycle.startDate, cycle.endDate) }} ·
                {{ cycle.issueIds.length }}
                {{ cycle.issueIds.length === 1 ? 'issue' : 'issues' }}
                <template v-if="selectedSeriesId === 'all' && cycle.seriesId">
                  ·
                  {{
                    cycles.series.find((series) => series.id === cycle.seriesId)?.name ?? 'Series'
                  }}
                </template>
              </p>
              <p v-if="cycle.projectIds?.length" class="mt-0.5 text-xs text-foreground-muted">
                {{
                  cycle.projectIds
                    .map(
                      (id) =>
                        projectOptions.find((project) => project.id === id)?.name ??
                        'Deleted project',
                    )
                    .join(', ')
                }}
              </p>
            </div>
            <span
              class="rounded-md bg-muted px-2 py-0.5 text-xs capitalize text-foreground-muted"
              >{{
                pendingCycles.some((pending) => pending.id === cycle.id)
                  ? 'Decision needed'
                  : cycles.statusOf(cycle)
              }}</span
            >
          </div>
          <div v-if="cycles.statusOf(cycle) !== 'completed'" class="flex items-center gap-1">
            <Button
              label="Edit"
              icon="i-tabler-pencil"
              variant="ghost"
              size="xs"
              @click="
                selected = cycle;
                editOpen = true;
              "
            />
            <Button
              label="Add issues"
              icon="i-tabler-plus"
              variant="ghost"
              size="xs"
              @click="
                selected = cycle;
                assignOpen = true;
              "
            />
            <Button
              v-if="!cycle.scheduled || pendingCycles.some((pending) => pending.id === cycle.id)"
              label="Complete"
              variant="ghost"
              size="xs"
              @click="
                selected = cycle;
                completeOpen = true;
              "
            />
          </div>
        </div>
        <div class="mt-3 ml-7 flex items-center gap-3 text-xs text-foreground-muted">
          <div
            class="h-1.5 min-w-0 flex-1 overflow-hidden rounded-full bg-muted"
            role="progressbar"
            :aria-valuenow="progress(cycle)"
            aria-valuemin="0"
            aria-valuemax="100"
            :aria-label="`${cycle.title} progress`"
          >
            <div
              class="h-full rounded-full bg-info transition-[width] duration-200"
              :style="{ width: `${progress(cycle)}%` }"
            />
          </div>
          <span class="w-20 text-right tabular-nums"
            >{{ completedCount(cycle) }}/{{
              cycle.status === 'completed'
                ? (cycle.totalCount ?? cycle.issueIds.length)
                : cycle.issueIds.length
            }}
            done</span
          >
        </div>
        <ul v-if="cycle.issueIds.length" class="mt-3 ml-7 space-y-1">
          <li
            v-for="id in cycle.issueIds"
            :key="id"
            class="flex min-h-7 items-center gap-2 text-xs"
          >
            <RouterLink
              v-if="issueFor(id)"
              :to="{ name: 'issue-full', params: { id } }"
              class="flex min-w-0 flex-1 items-center gap-2 rounded-md px-1 py-1 hover:bg-accented"
            >
              <span class="w-20 shrink-0 font-mono text-foreground-muted">{{ id }}</span>
              <span class="min-w-0 flex-1 truncate">{{ issueFor(id)?.title }}</span>
            </RouterLink>
            <span v-else class="text-foreground-muted">{{ id }} · Archived issue</span>
          </li>
        </ul>
      </section>
    </div>

    <CycleScheduleDialog
      v-model:open="scheduleOpen"
      :saving="saving"
      :settings="scheduleTarget"
      :cycles="cycles.items"
      :issues="issues.items.map(({ id, project }) => ({ identifier: id, project }))"
      :projects="projectOptions"
      :next-start="nextSeriesStart"
      @save="saveSchedule"
    />
    <CreateCycleDialog
      v-model:open="createOpen"
      :saving="saving"
      :projects="projectOptions"
      @create="create"
    />
    <CreateCycleDialog
      v-model:open="editOpen"
      :cycle="selected"
      :cycles="cycles.items"
      :saving="saving"
      :projects="projectOptions"
      @update="update"
    />
    <CycleIssueDialog
      v-model:open="assignOpen"
      :cycle="selected"
      :issues="issues.items"
      :projects="projectOptions"
      @toggle="toggleIssue"
    />
    <CompleteCycleDialog
      v-model:open="completeOpen"
      :cycle="selected"
      :cycles="cycles.items"
      :saving="saving"
      @complete="complete"
    />
  </main>
</template>
