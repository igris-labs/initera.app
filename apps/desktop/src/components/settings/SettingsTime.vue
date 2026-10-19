<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { ref } from 'vue';
import { loadRunningTimers } from '../../lib/time-storage';
import type { TimeEntry } from '../../lib/time-storage';
import { usePreferencesStore } from '../../stores/preferences';
import { Button } from '../ui/button';
import { Dialog } from '../ui/dialog';
import { RadioGroup } from '../ui/radio-group';
import { Select } from '../ui/select';
import { Switch } from '../ui/switch';

const preferences = usePreferencesStore();
const { timerOnExit, concurrentTimers, timerOnSleep } = storeToRefs(preferences);
const error = ref('');
const concurrentDialogOpen = ref(false);
const runningTimers = ref<TimeEntry[]>([]);
const keepTimerIssue = ref('');
const timerSleepOptions = [
  { value: 'pause', label: 'Pause while asleep' },
  { value: 'include', label: 'Include sleep time' },
];

async function changeConcurrentTimers(enabled: boolean) {
  if (enabled) {
    concurrentTimers.value = true;
    return;
  }

  try {
    runningTimers.value = await loadRunningTimers();
    if (runningTimers.value.length < 2) {
      concurrentTimers.value = false;
      return;
    }

    keepTimerIssue.value = runningTimers.value[0]?.issueIdentifier ?? '';
    concurrentDialogOpen.value = true;
  } catch (cause) {
    error.value = String(cause);
  }
}

async function confirmSingleTimer() {
  preferences.disableConcurrentTimers(keepTimerIssue.value);
  await preferences.flushSaves();
  if (preferences.saveError) {
    error.value = preferences.saveError;
    concurrentTimers.value = true;
    return;
  }

  concurrentDialogOpen.value = false;
  window.dispatchEvent(new Event('timer-changed'));
}
</script>

<template>
  <p v-if="error" role="alert" class="mb-4 text-xs text-error">{{ error }}</p>
  <section aria-labelledby="timer-heading">
    <h3 id="timer-heading" class="text-sm font-medium text-foreground-highlighted">
      Time tracking
    </h3>
    <p class="mt-1 text-xs text-foreground-muted">
      Choose how timers run and what happens when the app closes or sleeps.
    </p>
    <RadioGroup
      :model-value="timerOnExit"
      :options="[
        {
          value: 'pause',
          label: 'Pause on exit',
          description: 'After exit or a crash, stop at the last durable checkpoint.',
        },
        {
          value: 'continue',
          label: 'Continue after exit',
          description: 'Include elapsed time while the app is closed, including sleep.',
        },
      ]"
      aria-label="Timer on exit"
      size="sm"
      :ui="{
        root: 'mt-3 gap-1',
        item: 'min-h-12 w-full rounded-md px-3 py-2 text-left hover:bg-muted',
      }"
      @value-change="(value: string) => (timerOnExit = value as 'pause' | 'continue')"
    >
      <template #item="{ option, isChecked }">
        <span
          class="flex size-4 shrink-0 items-center justify-center rounded-full border border-outline-strong"
          aria-hidden="true"
        >
          <span v-if="isChecked" class="size-2 rounded-full bg-primary" />
        </span>
        <span class="min-w-0 flex-1">
          <span class="block text-sm font-medium text-foreground">{{ option.label }}</span>
          <span class="block text-xs text-foreground-muted">{{ option.description }}</span>
        </span>
      </template>
    </RadioGroup>
    <div class="mt-4 flex items-center justify-between gap-3 text-xs text-foreground-muted">
      <span>Allow timers on different issues to run together</span>
      <Switch
        :model-value="concurrentTimers"
        aria-label="Concurrent timers"
        size="sm"
        @update:model-value="changeConcurrentTimers"
      />
    </div>
    <label class="mt-4 flex items-center justify-between gap-3 text-xs text-foreground-muted">
      While app is open and computer sleeps
      <Select
        v-model="timerOnSleep"
        :items="timerSleepOptions"
        aria-label="Timer on sleep"
        class="w-44 shrink-0"
        size="sm"
      />
    </label>
    <p v-if="timerOnExit === 'continue'" class="mt-2 px-3 text-xs text-foreground-dimmed">
      Closed time includes sleep. Pause the timer before leaving if you do not want that time
      counted.
    </p>
  </section>
  <Dialog v-model:open="concurrentDialogOpen" title="Choose running timer" size="sm">
    <template #body>
      <p class="mb-3 text-xs text-foreground-muted">
        Turning off concurrent timers pauses every other running timer.
      </p>
      <Select
        v-model="keepTimerIssue"
        :items="[
          { label: 'Pause all timers', value: '' },
          ...runningTimers.map((entry) => ({
            label: `Keep ${entry.issueIdentifier} running`,
            value: entry.issueIdentifier,
          })),
        ]"
        aria-label="Timer to keep"
        size="sm"
      />
    </template>
    <template #footer>
      <Button label="Cancel" variant="ghost" size="sm" @click="concurrentDialogOpen = false" />
      <Button label="Apply" size="sm" @click="confirmSingleTimer" />
    </template>
  </Dialog>
</template>
