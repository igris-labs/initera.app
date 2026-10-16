<script setup lang="ts">
import { useEventListener, useIntervalFn, useNow } from '@vueuse/core';
import { computed, ref, watch } from 'vue';
import {
  addTime,
  deleteTime,
  loadTime,
  startTimer,
  stopTimer,
  updateTime,
} from '../../lib/time-storage';
import type { ManualTimeInput, TimeEntry, TimeSummary } from '../../lib/time-storage';
import { ApplicationError } from '../../lib/native-invoke';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Select } from '../ui/select';
import { Dialog } from '../ui/dialog';

interface Props {
  issueId: string;
}

const { issueId } = defineProps<Props>();
const now = useNow({ scheduler: (callback) => useIntervalFn(callback, 1000) });
const summary = ref<TimeSummary | null>(null);
const summaryReceivedAt = ref(performance.now());
const busy = ref(false);
const error = ref('');
const hours = ref('');
const minutes = ref('');
const secondsPart = ref('');
const note = ref('');
const entryDate = ref('');
const entryStartTime = ref('');
const timeZone = ref(Intl.DateTimeFormat().resolvedOptions().timeZone);
const ambiguous = ref<'earlier' | 'later' | ''>('');
const manualOpen = ref(false);
const editingEntryId = ref<string | null>(null);
const overlapOpen = ref(false);
const switchOpen = ref(false);

const runningHere = computed(() =>
  summary.value?.entries.some((entry) => entry.stoppedAt === null),
);
const total = computed(() => {
  if (!summary.value) return 0;
  void now.value;
  const active = summary.value.entries.find((entry) => entry.stoppedAt === null);
  return (
    summary.value.entries.reduce(
      (sum, entry) => sum + (entry.stoppedAt === null ? 0 : entry.durationSeconds),
      0,
    ) +
    (active
      ? active.durationSeconds +
        Math.max(0, Math.floor((performance.now() - summaryReceivedAt.value) / 1000))
      : 0)
  );
});

function duration(seconds: number) {
  const value = Math.max(0, Math.floor(seconds));
  return `${Math.floor(value / 3600)}h ${String(Math.floor((value % 3600) / 60)).padStart(2, '0')}m`;
}

async function refresh(identifier: string) {
  try {
    const result = await loadTime(identifier);
    if (identifier === issueId) {
      summary.value = result;
      summaryReceivedAt.value = performance.now();
    }

    error.value = '';
  } catch (cause) {
    error.value = String(cause);
  }
}

watch(
  () => issueId,
  (identifier) => {
    summary.value = null;
    resetForm();
    void refresh(identifier);
  },
  { immediate: true },
);

useEventListener(window, 'timer-changed', () => void refresh(issueId));

async function mutate(action: () => Promise<TimeSummary>) {
  busy.value = true;
  error.value = '';
  try {
    summary.value = await action();
    summaryReceivedAt.value = performance.now();
    return true;
  } catch (cause) {
    error.value = String(cause);
    if (cause instanceof ApplicationError && cause.code === 'CONFIRMATION_REQUIRED') {
      if (cause.message.includes('overlap')) overlapOpen.value = true;
      else switchOpen.value = true;
    }

    return false;
  } finally {
    busy.value = false;
  }
}

function resetForm() {
  hours.value = '';
  minutes.value = '';
  secondsPart.value = '';
  note.value = '';
  const defaultStart = new Date(Date.now() - 60 * 60 * 1000);
  entryDate.value = `${defaultStart.getFullYear()}-${String(defaultStart.getMonth() + 1).padStart(2, '0')}-${String(defaultStart.getDate()).padStart(2, '0')}`;
  entryStartTime.value = `${String(defaultStart.getHours()).padStart(2, '0')}:${String(defaultStart.getMinutes()).padStart(2, '0')}`;
  timeZone.value = Intl.DateTimeFormat().resolvedOptions().timeZone;
  ambiguous.value = '';
  editingEntryId.value = null;
  manualOpen.value = false;
}

function editEntry(entry: TimeEntry) {
  hours.value = String(Math.floor(entry.durationSeconds / 3600));
  minutes.value = String(Math.floor((entry.durationSeconds % 3600) / 60));
  secondsPart.value = String(entry.durationSeconds % 60);
  note.value = entry.note;
  timeZone.value = entry.timeZone ?? Intl.DateTimeFormat().resolvedOptions().timeZone;
  const parts = new Intl.DateTimeFormat('en-CA', {
    timeZone: timeZone.value,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    hourCycle: 'h23',
  }).formatToParts(entry.startedAt * 1000);
  const part = (name: string) => parts.find((item) => item.type === name)?.value ?? '';
  entryDate.value = `${part('year')}-${part('month')}-${part('day')}`;
  entryStartTime.value = `${part('hour')}:${part('minute')}`;
  editingEntryId.value = entry.id;
  manualOpen.value = true;
}

async function saveManual(allowOverlap = false) {
  const enteredHours = Number(hours.value || 0);
  const enteredMinutes = Number(minutes.value || 0);
  const enteredSeconds = Number(secondsPart.value || 0);
  const seconds = enteredHours * 3600 + enteredMinutes * 60 + enteredSeconds;
  if (
    ![enteredHours, enteredMinutes, enteredSeconds].every(Number.isInteger) ||
    seconds <= 0 ||
    seconds > 31_536_000 ||
    enteredHours < 0 ||
    enteredMinutes < 0 ||
    enteredMinutes > 59 ||
    enteredSeconds < 0 ||
    enteredSeconds > 59
  ) {
    error.value = 'Enter a positive duration.';
    return;
  }

  const when: ManualTimeInput = {
    date: entryDate.value,
    startTime: entryStartTime.value,
    timeZone: timeZone.value,
    ...(ambiguous.value ? { ambiguous: ambiguous.value } : {}),
    allowOverlap,
  };
  const saved = await mutate(() =>
    editingEntryId.value
      ? updateTime(
          issueId,
          editingEntryId.value,
          seconds,
          note.value,
          summary.value?.entries.find((entry) => entry.id === editingEntryId.value)?.version ?? 0,
          when,
        )
      : addTime(issueId, seconds, note.value, when),
  );
  if (saved) {
    overlapOpen.value = false;
    resetForm();
  }
}
</script>

<template>
  <section class="mt-7 border-t border-outline pt-4" aria-label="Time tracking">
    <Dialog v-model:open="switchOpen" title="Switch timer" size="sm">
      <template #body>
        <p class="text-xs text-foreground-muted">
          Pause the current timer and start tracking this issue?
        </p>
      </template>
      <template #footer>
        <Button label="Cancel" variant="ghost" size="sm" @click="switchOpen = false" />
        <Button
          label="Switch timer"
          size="sm"
          @click="
            mutate(() => startTimer(issueId, true)).then((saved) => {
              if (saved) switchOpen = false;
            })
          "
        />
      </template>
    </Dialog>
    <Dialog v-model:open="overlapOpen" title="Overlapping time" size="sm">
      <template #body>
        <p class="text-xs text-foreground-muted">
          This entry overlaps existing tracked time. Totals will include both entries.
        </p>
      </template>
      <template #footer>
        <Button label="Cancel" variant="ghost" size="sm" @click="overlapOpen = false" />
        <Button label="Save anyway" size="sm" @click="saveManual(true)" />
      </template>
    </Dialog>
    <div class="flex items-center justify-between gap-2 px-1">
      <h3 class="text-xs font-medium text-foreground-muted">Time tracked</h3>
      <span class="text-xs tabular-nums text-foreground">{{ duration(total) }}</span>
    </div>
    <p v-if="summary?.running && !runningHere" class="mt-2 px-1 text-xs text-foreground-muted">
      Timer running on {{ summary.running.issueIdentifier }}.
    </p>
    <div class="mt-2 flex flex-wrap items-center gap-1">
      <Button
        v-if="runningHere"
        label="Pause timer"
        icon="i-tabler-player-pause"
        variant="ghost"
        size="xs"
        :disabled="busy"
        @click="mutate(() => stopTimer(issueId))"
      />
      <Button
        v-else
        label="Start timer"
        icon="i-tabler-player-play"
        variant="ghost"
        size="xs"
        :disabled="busy"
        @click="mutate(() => startTimer(issueId))"
      />
      <Button
        label="Add time"
        icon="i-tabler-plus"
        variant="ghost"
        size="xs"
        @click="manualOpen ? resetForm() : (manualOpen = true)"
      />
    </div>
    <form
      v-if="manualOpen"
      class="mt-2 space-y-2 rounded-lg border border-outline p-2"
      :aria-label="editingEntryId ? 'Edit time entry' : 'Add time entry'"
      @submit.prevent="saveManual()"
    >
      <div class="flex gap-2">
        <Input v-model="entryDate" type="date" aria-label="Entry date" size="sm" />
        <Input v-model="entryStartTime" type="time" aria-label="Start time" size="sm" />
      </div>
      <p class="text-xs text-foreground-muted">Timezone: {{ timeZone }}</p>
      <Select
        v-model="ambiguous"
        :items="[
          { label: 'Daylight saving: automatic', value: '' },
          { label: 'Earlier occurrence', value: 'earlier' },
          { label: 'Later occurrence', value: 'later' },
        ]"
        aria-label="Daylight saving overlap"
        size="sm"
      />
      <div class="flex gap-2">
        <Input
          v-model="hours"
          type="number"
          min="0"
          max="8760"
          placeholder="Hours"
          aria-label="Hours"
          size="sm"
        />
        <Input
          v-model="minutes"
          type="number"
          min="0"
          max="59"
          placeholder="Minutes"
          aria-label="Minutes"
          size="sm"
        />
        <Input
          v-model="secondsPart"
          type="number"
          min="0"
          max="59"
          placeholder="Seconds"
          aria-label="Seconds"
          size="sm"
        />
      </div>
      <Input
        v-model="note"
        placeholder="Note (optional)"
        aria-label="Time entry note"
        maxlength="500"
        size="sm"
      />
      <div class="flex items-center gap-1">
        <Button
          :label="editingEntryId ? 'Save changes' : 'Save time'"
          type="submit"
          size="xs"
          :disabled="busy"
        />
        <Button label="Cancel" variant="ghost" size="xs" @click="resetForm" />
      </div>
    </form>
    <p v-if="error" class="mt-2 text-xs text-error" role="alert">{{ error }}</p>
    <ul v-if="summary?.entries.length" class="mt-2 divide-y divide-outline">
      <li
        v-for="entry in summary.entries"
        :key="entry.id"
        class="flex items-center justify-between gap-2 py-1.5 text-xs"
      >
        <span class="min-w-0 truncate text-foreground-muted">{{
          entry.note ||
          (entry.stoppedAt === null
            ? 'Running'
            : new Date(entry.startedAt * 1000).toLocaleDateString())
        }}</span>
        <span class="shrink-0 tabular-nums">{{
          entry.stoppedAt === null ? 'Live' : duration(entry.durationSeconds)
        }}</span>
        <Button
          v-if="entry.stoppedAt !== null"
          icon="i-tabler-pencil"
          aria-label="Edit time entry"
          variant="ghost"
          size="xs"
          :disabled="busy"
          @click="editEntry(entry)"
        />
        <Button
          v-if="entry.stoppedAt !== null"
          icon="i-tabler-trash"
          aria-label="Delete time entry"
          variant="ghost"
          size="xs"
          :disabled="busy"
          @click="mutate(() => deleteTime(issueId, entry.id, entry.version))"
        />
      </li>
    </ul>
  </section>
</template>
