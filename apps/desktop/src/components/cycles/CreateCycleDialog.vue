<script setup lang="ts">
import { parseDate } from '@internationalized/date';
import type { CalendarDate } from '@internationalized/date';
import { nextTick, ref, watch } from 'vue';
import type { Cycle, CycleTiming } from '../../lib/cycle-storage';
import { formatCycleDateRange, formatCycleTime } from '../../lib/cycle-format';
import { Button } from '../ui/button';
import { DatePicker } from '../ui/date-picker';
import { Dialog } from '../ui/dialog';
import { Icon } from '../ui/icon';
import { Input } from '../ui/input';
import { Textarea } from '../ui/textarea';
import { Select } from '../ui/select';
import CycleDialogHeader from './CycleDialogHeader.vue';
import { cycleDialogUi } from './dialog-style';

interface Props {
  saving?: boolean;
  cycle?: Cycle | null;
  cycles?: Cycle[];
  projects: { id: string; name: string }[];
}

const { saving = false, cycle = null, cycles = [], projects } = defineProps<Props>();
const emit = defineEmits<{
  create: [
    title: string,
    startDate: string,
    endDate: string,
    projectIds: string[],
    timing: CycleTiming,
  ];
  update: [cycle: Cycle];
}>();
const open = defineModel<boolean>('open', { required: true });

const title = ref('');
const description = ref('');
const projectIds = ref<string[]>([]);
const startDate = ref<CalendarDate>();
const endDate = ref<CalendarDate>();
const timeZone = ref(Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC');
const startLocalTime = ref('09:00');
const endLocalTime = ref('18:00');
const rolloverPolicy = ref<CycleTiming['rolloverPolicy']>('ask');
const successorId = ref('none');
const timeZoneOptions = [
  { label: 'UTC', value: 'UTC' },
  ...Intl.supportedValuesOf('timeZone').map((value) => ({
    label: value.replaceAll('_', ' '),
    value,
  })),
];
const rolloverOptions = [
  { label: 'Ask before moving', value: 'ask' },
  { label: 'Move unfinished automatically', value: 'automatic' },
  { label: 'Leave in this cycle', value: 'leave' },
];
const titleInput = ref<{ inputRef?: HTMLInputElement } | null>(null);

watch(open, (value) => {
  if (value) {
    title.value = cycle?.title ?? '';
    description.value = cycle?.description ?? '';
    projectIds.value = cycle?.projectIds?.length
      ? [...cycle.projectIds]
      : projects.map((project) => project.id);
    startDate.value = cycle ? parseDate(cycle.startDate) : undefined;
    endDate.value = cycle ? parseDate(cycle.endDate) : undefined;
    timeZone.value = cycle?.timeZone || Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC';
    startLocalTime.value = cycle?.startLocalTime || '09:00';
    endLocalTime.value = cycle?.endLocalTime || '18:00';
    rolloverPolicy.value = cycle?.rolloverPolicy || 'ask';
    successorId.value = cycle?.successorId || 'none';
    nextTick(() => titleInput.value?.inputRef?.focus());
  }
});

function toggleProject(id: string) {
  projectIds.value = projectIds.value.includes(id)
    ? projectIds.value.filter((item) => item !== id)
    : [...projectIds.value, id];
}

function submit() {
  if (!title.value.trim() || !startDate.value || !endDate.value) return;
  if (startDate.value.toString() > endDate.value.toString()) return;
  if (cycle) {
    emit('update', {
      ...cycle,
      title: title.value.trim(),
      description: description.value.trim(),
      startDate: startDate.value.toString(),
      endDate: endDate.value.toString(),
      projectIds: projectIds.value,
      timeZone: timeZone.value,
      startLocalTime: startLocalTime.value,
      endLocalTime: endLocalTime.value,
      rolloverPolicy: rolloverPolicy.value,
      successorId: successorId.value === 'none' ? null : successorId.value,
    });
  } else {
    emit(
      'create',
      title.value.trim(),
      startDate.value.toString(),
      endDate.value.toString(),
      projectIds.value,
      {
        timeZone: timeZone.value,
        startLocalTime: startLocalTime.value,
        endLocalTime: endLocalTime.value,
        rolloverPolicy: rolloverPolicy.value,
      },
    );
  }
}
</script>

<template>
  <Dialog
    v-model:open="open"
    :title="cycle ? 'Edit cycle' : 'Create cycle'"
    description="Set a timebox for a group of issues."
    size="lg"
    class="workbench-surface"
    :ui="cycleDialogUi"
  >
    <template #header="{ close, titleId }">
      <CycleDialogHeader
        :title="cycle ? 'Edit cycle' : 'Create cycle'"
        :title-id="titleId"
        :close="close"
      />
    </template>
    <template #body>
      <form
        :id="cycle ? 'edit-cycle-form' : 'create-cycle-form'"
        class="space-y-3"
        @submit.prevent="submit"
      >
        <span
          class="flex size-9 items-center justify-center rounded-lg bg-muted text-foreground-muted"
        >
          <Icon name="i-tabler-calendar-time" class="size-5" />
        </span>
        <Input
          ref="titleInput"
          v-model="title"
          aria-label="Cycle name"
          placeholder="Cycle name"
          maxlength="80"
          variant="bare"
          size="lg"
          :ui="{
            root: 'min-h-10 border-0 px-0 text-xl font-semibold placeholder:text-foreground-dimmed',
          }"
        />
        <Textarea
          v-model="description"
          aria-label="Cycle description"
          placeholder="Add a description…"
          rows="2"
          maxlength="4000"
          variant="bare"
          class="w-full resize-none text-sm"
        />
        <div v-if="projects.length" class="pt-2">
          <p class="mb-2 text-xs text-foreground-muted">Projects in this cycle</p>
          <div class="flex flex-wrap gap-2">
            <Button
              v-for="project in projects"
              :key="project.id"
              :label="project.name"
              :icon="projectIds.includes(project.id) ? 'i-tabler-check' : 'i-tabler-cube'"
              :aria-pressed="projectIds.includes(project.id)"
              variant="ghost"
              size="sm"
              @click="toggleProject(project.id)"
            />
          </div>
        </div>
        <div
          v-if="cycle?.scheduled"
          class="flex items-center gap-2 pt-1 text-xs text-foreground-muted"
        >
          <Icon name="i-tabler-calendar" class="size-3.5" />
          {{ formatCycleDateRange(cycle.startDate, cycle.endDate) }}
        </div>
        <div v-else class="flex flex-wrap items-center gap-1.5 pt-1">
          <DatePicker
            v-model="startDate"
            label="Start date"
            calendar-label="Choose cycle start date"
          />
          <Icon
            name="i-tabler-arrow-right"
            class="size-3.5 text-foreground-dimmed"
            aria-hidden="true"
          />
          <DatePicker v-model="endDate" label="End date" calendar-label="Choose cycle end date" />
        </div>
        <p v-if="cycle?.scheduled" class="text-xs text-foreground-muted">
          Scheduled cycle dates follow workspace schedule.
        </p>
        <div v-if="!cycle?.scheduled" class="grid grid-cols-2 gap-2 pt-2">
          <div>
            <label for="cycle-start-time" class="mb-1 block text-xs text-foreground-muted">
              Start time
            </label>
            <Input
              id="cycle-start-time"
              v-model="startLocalTime"
              type="time"
              aria-label="Cycle start time"
              size="sm"
            />
            <p v-if="startLocalTime" class="mt-1 text-xs text-foreground-muted">
              {{ formatCycleTime(startLocalTime) }}
            </p>
          </div>
          <div>
            <label for="cycle-end-time" class="mb-1 block text-xs text-foreground-muted">
              End time
            </label>
            <Input
              id="cycle-end-time"
              v-model="endLocalTime"
              type="time"
              aria-label="Cycle end time"
              size="sm"
            />
            <p v-if="endLocalTime" class="mt-1 text-xs text-foreground-muted">
              {{ formatCycleTime(endLocalTime) }}
            </p>
          </div>
        </div>
        <div v-if="!cycle?.scheduled" class="grid grid-cols-2 gap-2">
          <Select
            v-model="timeZone"
            :items="timeZoneOptions"
            aria-label="Cycle time zone"
            searchable
            size="sm"
          />
          <Select
            v-model="rolloverPolicy"
            :items="rolloverOptions"
            aria-label="Cycle rollover policy"
            size="sm"
          />
        </div>
        <div v-if="cycle && !cycle.scheduled" class="flex items-center justify-between gap-4 pt-2">
          <span class="text-sm text-foreground">Manual successor</span>
          <Select
            v-model="successorId"
            :items="[
              { label: 'No successor', value: 'none' },
              ...cycles
                .filter(
                  (item) =>
                    !item.scheduled &&
                    item.id !== cycle.id &&
                    item.status !== 'completed' &&
                    (item.startAt && cycle.endAt
                      ? item.startAt >= cycle.endAt
                      : item.startDate > cycle.endDate),
                )
                .map((item) => ({ label: item.title, value: item.id })),
            ]"
            aria-label="Manual cycle successor"
            size="sm"
          />
        </div>
        <Select
          v-if="cycle?.scheduled"
          v-model="rolloverPolicy"
          :items="rolloverOptions"
          aria-label="Cycle rollover policy"
          size="sm"
        />
        <p
          v-if="startDate && endDate && startDate.toString() > endDate.toString()"
          class="text-xs text-error"
          role="alert"
        >
          End date must be on or after start date.
        </p>
      </form>
    </template>
    <template #footer>
      <span class="mr-auto text-xs text-foreground-muted">Add issues after creating the cycle</span>
      <Button
        :label="cycle ? 'Save changes' : 'Create cycle'"
        type="submit"
        :form="cycle ? 'edit-cycle-form' : 'create-cycle-form'"
        size="sm"
        :disabled="
          saving ||
          !title.trim() ||
          !startDate ||
          !endDate ||
          startDate.toString() > endDate.toString() ||
          (projects.length > 0 && projectIds.length === 0)
        "
      />
    </template>
  </Dialog>
</template>
