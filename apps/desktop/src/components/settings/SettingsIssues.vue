<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { ref } from 'vue';
import { estimateScales } from '../../lib/estimates';
import type { EstimateScale } from '../../lib/estimates';
import { notifications } from '../../lib/notifications';
import { getProjectMovePreference, setProjectMovePreference } from '../../lib/workspace-options';
import type { ProjectMovePreference } from '../../lib/workspace-options';
import { usesNativeStorage } from '../../lib/workspace-storage';
import { usePreferencesStore } from '../../stores/preferences';
import { RadioGroup } from '../ui/radio-group';
import { Select } from '../ui/select';
import { Switch } from '../ui/switch';

const preferences = usePreferencesStore();
const { groupIssuesByStatus, showIssuePriority, showIssueProject, estimateScale } =
  storeToRefs(preferences);
const projectMovePreference = ref<ProjectMovePreference>('ask');
const error = ref('');

const projectMoveOptions = [
  { value: 'ask', label: 'Ask every time' },
  { value: 'keep', label: 'Keep current identifier' },
  { value: 'regenerate', label: 'Use new project identifier' },
];

if (usesNativeStorage) {
  void getProjectMovePreference()
    .then((value) => {
      projectMovePreference.value = value;
    })
    .catch((cause) => {
      error.value = String(cause);
    });
}

async function updateProjectMovePreference(value: string) {
  const preference = value as ProjectMovePreference;
  try {
    await setProjectMovePreference(preference);
    projectMovePreference.value = preference;
    notifications.add({ title: 'Move preference saved', type: 'success' });
  } catch (cause) {
    error.value = String(cause);
  }
}
</script>

<template>
  <p v-if="error" role="alert" class="mb-4 text-xs text-error">{{ error }}</p>
  <div class="space-y-10">
    <section aria-labelledby="issue-preferences-heading">
      <h3 id="issue-preferences-heading" class="text-sm font-medium text-foreground-highlighted">
        Issue moves
      </h3>
      <label class="mt-3 flex items-center justify-between gap-3 text-xs text-foreground-muted">
        Identifier when moving to another project
        <Select
          :model-value="projectMovePreference"
          :items="projectMoveOptions"
          aria-label="Issue identifier move preference"
          class="w-48 shrink-0"
          size="sm"
          @update:model-value="updateProjectMovePreference"
        />
      </label>
    </section>
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
        Choose the scale used when estimating work in this workspace. Existing values are kept when
        you switch scales.
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
  </div>
</template>
