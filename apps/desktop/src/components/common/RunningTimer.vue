<script setup lang="ts">
import { useEventListener, useIntervalFn, useNow } from '@vueuse/core';
import { onMounted, ref } from 'vue';
import { useRouter } from 'vue-router';
import { loadRunningTimers, stopTimer } from '../../lib/time-storage';
import type { TimeEntry } from '../../lib/time-storage';
import { Button } from '../ui/button';
import { Icon } from '../ui/icon';

const router = useRouter();
const now = useNow({ scheduler: (callback) => useIntervalFn(callback, 1000) });
const running = ref<TimeEntry[]>([]);
const receivedAt = ref(performance.now());
const busy = ref(false);
const error = ref('');

function elapsed(entry: TimeEntry) {
  void now.value;
  const seconds =
    entry.durationSeconds + Math.max(0, Math.floor((performance.now() - receivedAt.value) / 1000));
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const remainder = seconds % 60;
  return `${hours}:${String(minutes).padStart(2, '0')}:${String(remainder).padStart(2, '0')}`;
}

async function refresh() {
  try {
    running.value = await loadRunningTimers();
    receivedAt.value = performance.now();
    error.value = '';
  } catch (cause) {
    error.value = String(cause);
  }
}

async function pause(entry: TimeEntry) {
  busy.value = true;
  try {
    await stopTimer(entry.issueIdentifier);
    await refresh();
    error.value = '';
  } catch (cause) {
    error.value = String(cause);
  } finally {
    busy.value = false;
  }
}

onMounted(() => void refresh());
useEventListener(window, 'timer-changed', () => void refresh());
useIntervalFn(() => void refresh(), 30_000);
</script>

<template>
  <div v-if="running.length || error" class="mb-1 rounded-md bg-muted/60 px-2 py-1.5">
    <p v-if="running.length > 1" class="mb-1 text-xs text-foreground-muted">
      {{ running.length }} running timers
    </p>
    <div v-for="entry in running" :key="entry.id" class="flex min-w-0 items-center gap-1.5 text-xs">
      <Icon name="i-tabler-clock" class="size-3.5 shrink-0 text-info" />
      <button
        type="button"
        class="min-w-0 flex-1 truncate text-left text-foreground hover:text-foreground-highlighted"
        :aria-label="`Open timed issue ${entry.issueIdentifier}`"
        @click="router.push({ name: 'issue-full', params: { id: entry.issueIdentifier } })"
      >
        {{ entry.issueIdentifier }}
      </button>
      <span class="shrink-0 font-mono text-foreground-muted tabular-nums">{{
        elapsed(entry)
      }}</span>
      <Button
        icon="i-tabler-player-pause"
        aria-label="Pause running timer"
        variant="ghost"
        size="xs"
        :disabled="busy"
        @click="pause(entry)"
      />
    </div>
    <p v-if="error" class="text-xs text-danger" role="alert">{{ error }}</p>
  </div>
</template>
