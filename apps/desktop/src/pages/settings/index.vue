<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { nextTick, ref, watch } from 'vue';
import { useRoute } from 'vue-router';
import { useThemeChange } from '../../composables/useThemeChange';
import SettingsBackups from '../../components/settings/SettingsBackups.vue';
import SettingsAbout from '../../components/settings/SettingsAbout.vue';
import SettingsLabels from '../../components/settings/SettingsLabels.vue';
import SettingsWorkflow from '../../components/settings/SettingsWorkflow.vue';
import { usePreferencesStore } from '../../stores/preferences';
import { useWorkspaceStore } from '../../stores/workspace';
import { usesNativeStorage } from '../../lib/workspace-storage';
import { notifications } from '../../lib/notifications';
import { loadRunningTimers } from '../../lib/time-storage';
import type { TimeEntry } from '../../lib/time-storage';
import { getProjectMovePreference, setProjectMovePreference } from '../../lib/workspace-options';
import type { ProjectMovePreference } from '../../lib/workspace-options';
import type { ThemePreference } from '../../types/preferences';
import { estimateScales } from '../../lib/estimates';
import type { EstimateScale } from '../../lib/estimates';
import { Button } from '../../components/ui/button';
import { Input } from '../../components/ui/input';
import { Dialog } from '../../components/ui/dialog';
import { RadioGroup } from '../../components/ui/radio-group';
import { Switch } from '../../components/ui/switch';
import { Select } from '../../components/ui/select';

const preferencesStore = usePreferencesStore();
const {
  groupIssuesByStatus,
  showIssuePriority,
  showIssueProject,
  estimateScale,
  timerOnExit,
  concurrentTimers,
  timerOnSleep,
} = storeToRefs(preferencesStore);
const { theme, changeTheme } = useThemeChange();
const workspace = useWorkspaceStore();
const route = useRoute();

watch(
  () => route.hash,
  async (hash) => {
    if (hash !== '#labels' && hash !== '#workflow') return;
    await nextTick();
    document.getElementById(hash.slice(1))?.scrollIntoView({ block: 'start' });
  },
  { immediate: true, flush: 'post' },
);

const userName = ref(workspace.userName);
const workspaceName = ref(workspace.workspaceName);
const settingsError = ref('');
const projectMovePreference = ref<ProjectMovePreference>('ask');
const concurrentDialogOpen = ref(false);
const runningTimers = ref<TimeEntry[]>([]);
const keepTimerIssue = ref('');

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
    settingsError.value = String(cause);
  }
}

async function confirmSingleTimer() {
  preferencesStore.disableConcurrentTimers(keepTimerIssue.value);
  await preferencesStore.flushSaves();
  if (preferencesStore.saveError) {
    settingsError.value = preferencesStore.saveError;
    concurrentTimers.value = true;
    return;
  }

  concurrentDialogOpen.value = false;
  window.dispatchEvent(new Event('timer-changed'));
}

if (usesNativeStorage) {
  void getProjectMovePreference().then((value) => {
    projectMovePreference.value = value;
  });
}

async function updateProjectMovePreference(value: string) {
  const preference = value as ProjectMovePreference;
  try {
    await setProjectMovePreference(preference);
    projectMovePreference.value = preference;
    notifications.add({ title: 'Move preference saved', type: 'success' });
  } catch (cause) {
    settingsError.value = String(cause);
  }
}

async function saveIdentity() {
  if (!userName.value.trim() || !workspaceName.value.trim()) return;
  try {
    await workspace.saveIdentity(userName.value, workspaceName.value);
  } catch {
    // Leave the edited values in place so the user can retry.
  }
}

const appearances: { value: ThemePreference; label: string; description: string }[] = [
  { value: 'system', label: 'System', description: 'Follow your device appearance' },
  { value: 'light', label: 'Light', description: 'Use the light workspace palette' },
  { value: 'dark', label: 'Dark', description: 'Use the dark workspace palette' },
];

function updateAppearance(value: string, details: { event?: Event }) {
  changeTheme(value as ThemePreference, details.event?.target as Element | undefined);
}
</script>

<template>
  <main class="min-h-0 flex-1 overflow-auto bg-default px-5 py-8 sm:px-8">
    <div class="mx-auto max-w-2xl space-y-10">
      <header>
        <p v-if="settingsError" role="alert" class="text-xs text-error">{{ settingsError }}</p>
        <h2 class="text-xl font-semibold tracking-tight text-foreground-highlighted">Settings</h2>
        <p class="mt-1 text-sm text-foreground-muted">{{ workspace.workspaceName }} preferences</p>
      </header>
      <section aria-labelledby="workspace-heading">
        <h3 id="workspace-heading" class="text-sm font-medium text-foreground-highlighted">
          Workspace
        </h3>
        <form class="mt-3 grid gap-3 sm:grid-cols-2" @submit.prevent="saveIdentity">
          <label class="space-y-1 text-xs font-medium text-foreground-muted">
            Your name
            <Input v-model="userName" aria-label="Your name" maxlength="80" required size="md" />
          </label>
          <label class="space-y-1 text-xs font-medium text-foreground-muted">
            Workspace name
            <Input
              v-model="workspaceName"
              aria-label="Workspace name"
              maxlength="80"
              required
              size="md"
            />
          </label>
          <Button class="sm:col-span-2 sm:justify-self-start" label="Save names" type="submit" />
          <p v-if="workspace.error" class="text-xs text-danger sm:col-span-2" role="alert">
            {{ workspace.error }}
          </p>
        </form>
        <p v-if="workspace.dataRoot" class="mt-3 break-all text-xs text-foreground-muted">
          Data folder: {{ workspace.dataRoot }}
        </p>
      </section>
      <section aria-labelledby="issue-preferences-heading">
        <h3 id="issue-preferences-heading" class="text-sm font-medium text-foreground-highlighted">
          Issue moves
        </h3>
        <label class="mt-3 flex items-center justify-between gap-3 text-xs text-foreground-muted">
          Identifier when moving to another project
          <Select
            :model-value="projectMovePreference"
            :items="['ask', 'keep', 'regenerate']"
            aria-label="Issue identifier move preference"
            size="sm"
            @update:model-value="updateProjectMovePreference"
          />
        </label>
      </section>
      <SettingsLabels />
      <SettingsWorkflow />
      <section aria-labelledby="appearance-heading">
        <h3 id="appearance-heading" class="text-sm font-medium text-foreground-highlighted">
          Appearance
        </h3>
        <RadioGroup
          :model-value="theme"
          :options="appearances"
          aria-label="Appearance"
          size="sm"
          :ui="{
            root: 'mt-3 gap-1',
            item: 'min-h-12 w-full rounded-md px-3 py-2 text-left transition-colors duration-150 ease-out hover:bg-muted focus-visible:outline-2 focus-visible:outline-info',
          }"
          @value-change="updateAppearance"
        >
          <template #item="{ option, isChecked }">
            <span
              :class="[
                'flex size-4 shrink-0 items-center justify-center rounded-full border',
                isChecked ? 'border-info' : 'border-foreground-dimmed',
              ]"
              aria-hidden="true"
            >
              <span v-if="isChecked" class="size-2 rounded-full bg-info" />
            </span>
            <span class="min-w-0 flex-1">
              <span class="block text-sm font-medium text-foreground">{{ option.label }}</span>
              <span class="block text-xs text-foreground-muted">{{ option.description }}</span>
            </span>
          </template>
        </RadioGroup>
      </section>
      <SettingsBackups v-if="usesNativeStorage" />
      <section aria-labelledby="issues-heading">
        <h3 id="issues-heading" class="text-sm font-medium text-foreground-highlighted">
          Issue display
        </h3>
        <div class="mt-3 space-y-1">
          <label
            class="flex min-h-10 items-center justify-between gap-4 rounded-md px-3 text-sm text-foreground hover:bg-muted"
          >
            Group list by status
            <Switch v-model="groupIssuesByStatus" aria-label="Group list by status" size="sm" />
          </label>
          <label
            class="flex min-h-10 items-center justify-between gap-4 rounded-md px-3 text-sm text-foreground hover:bg-muted"
          >
            Show priority
            <Switch v-model="showIssuePriority" aria-label="Show priority" size="sm" />
          </label>
          <label
            class="flex min-h-10 items-center justify-between gap-4 rounded-md px-3 text-sm text-foreground hover:bg-muted"
          >
            Show project
            <Switch v-model="showIssueProject" aria-label="Show project" size="sm" />
          </label>
        </div>
      </section>
      <section aria-labelledby="estimates-heading">
        <h3 id="estimates-heading" class="text-sm font-medium text-foreground-highlighted">
          Issue estimates
        </h3>
        <p class="mt-1 text-xs text-foreground-muted">
          Choose the scale used when estimating work in this workspace. Existing values are kept
          when you switch scales.
        </p>
        <RadioGroup
          :model-value="estimateScale"
          :options="estimateScales"
          aria-label="Estimate scale"
          size="sm"
          :ui="{
            root: 'mt-3 gap-1',
            item: 'min-h-12 w-full rounded-md px-3 py-2 text-left hover:bg-muted',
          }"
          @value-change="(value: string) => (estimateScale = value as EstimateScale)"
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
        <p class="mt-2 px-3 text-xs text-foreground-dimmed">
          Time estimates use 8-hour workdays, 40-hour workweeks, and 160-hour workmonths.
        </p>
      </section>
      <section v-if="usesNativeStorage" aria-labelledby="timer-heading">
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
            :items="['pause', 'include']"
            aria-label="Timer on sleep"
            size="sm"
          />
        </label>
        <p v-if="timerOnExit === 'continue'" class="mt-2 px-3 text-xs text-foreground-dimmed">
          Closed time includes sleep. Pause the timer before leaving if you do not want that time
          counted.
        </p>
      </section>
      <SettingsAbout />
    </div>
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
  </main>
</template>
