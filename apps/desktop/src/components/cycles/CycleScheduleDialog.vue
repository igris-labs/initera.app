<script setup lang="ts">
import { parseDate } from '@internationalized/date';
import { computed, ref, shallowRef, watch } from 'vue';
import type { Cycle, CycleSettings } from '../../lib/cycle-storage';
import { formatCycleDateRange, formatCycleInstant, formatCycleTime } from '../../lib/cycle-format';
import {
  cycleBoundaryPreview,
  localDate,
  offsetDate,
  validCycleInterval,
} from '../../lib/cycle-schedule';
import { Button } from '../ui/button';
import { DatePicker } from '../ui/date-picker';
import { Dialog } from '../ui/dialog';
import { Icon } from '../ui/icon';
import { Select } from '../ui/select';
import { Input } from '../ui/input';
import CycleDialogHeader from './CycleDialogHeader.vue';
import { cycleDialogUi } from './dialog-style';

interface Props {
  saving?: boolean;
  settings?: CycleSettings | null;
  nextStart?: string;
  cycles: Cycle[];
  issues: { identifier: string; project?: string | null }[];
  projects: { id: string; name: string }[];
}

const {
  saving = false,
  settings = null,
  nextStart,
  cycles,
  issues,
  projects,
} = defineProps<Props>();
const emit = defineEmits<{ save: [settings: CycleSettings] }>();
const open = defineModel<boolean>('open', { required: true });

const name = ref('Main');
const duration = ref('14');
const interval = ref('2');
const intervalUnit = ref<'days' | 'weeks'>('weeks');
const rolloverPolicy = ref<CycleSettings['rolloverPolicy']>('ask');
const timeZone = ref(Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC');
const startLocalTime = ref('09:00');
const timeZoneOptions = [
  { label: 'UTC', value: 'UTC' },
  ...Intl.supportedValuesOf('timeZone').map((value) => ({
    label: value.replaceAll('_', ' '),
    value,
  })),
];
const projectIds = ref<string[]>([]);
const start = shallowRef(parseDate(offsetDate(localDate(), 1)));

watch(open, (value) => {
  if (!value) return;
  name.value = settings?.name ?? 'New series';
  duration.value = String(settings?.durationDays ?? 14);
  const intervalDays = settings?.intervalDays ?? 14;
  intervalUnit.value = intervalDays % 7 === 0 ? 'weeks' : 'days';
  interval.value = String(intervalUnit.value === 'weeks' ? intervalDays / 7 : intervalDays);
  rolloverPolicy.value = settings?.rolloverPolicy ?? 'ask';
  timeZone.value = settings?.timeZone || Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC';
  startLocalTime.value = settings?.startLocalTime || '09:00';
  projectIds.value = settings?.projectIds?.length
    ? [...settings.projectIds]
    : projects.map((project) => project.id);
  start.value = parseDate(
    settings ? (nextStart ?? offsetDate(localDate(), 1)) : offsetDate(localDate(), 1),
  );
});

function toggleProject(id: string) {
  projectIds.value = projectIds.value.includes(id)
    ? projectIds.value.filter((item) => item !== id)
    : [...projectIds.value, id];
}

const intervalOptions = [
  { label: 'Days', value: 'days' },
  { label: 'Weeks', value: 'weeks' },
];
const intervalDays = computed(
  () => Number(interval.value) * (intervalUnit.value === 'weeks' ? 7 : 1),
);
const validInterval = computed(
  () =>
    Number.isInteger(intervalDays.value) &&
    validCycleInterval(start.value.toString(), Number(duration.value), intervalDays.value),
);
const futureCycles = computed(() =>
  cycles
    .filter(
      (cycle) =>
        cycle.seriesId === settings?.id &&
        cycle.status !== 'completed' &&
        cycle.startDate > localDate(),
    )
    .sort((a, b) => (a.scheduleIndex ?? 0) - (b.scheduleIndex ?? 0)),
);
const firstIndex = computed(() => {
  if (!settings) return 0;
  const locked = cycles
    .filter(
      (cycle) =>
        cycle.seriesId === settings.id &&
        (cycle.startDate <= localDate() || cycle.status === 'completed'),
    )
    .sort((a, b) => (b.scheduleIndex ?? 0) - (a.scheduleIndex ?? 0))[0];
  return (locked?.scheduleIndex ?? -1) + 1;
});
const affectedIssues = computed(() => {
  if (!settings || !projectIds.value.length) return [];
  const projectByName = new Map(projects.map((project) => [project.name, project.id]));
  const issueByIdentifier = new Map(issues.map((issue) => [issue.identifier, issue]));
  return [
    ...new Set(
      futureCycles.value
        .flatMap((cycle) => cycle.issueIds)
        .filter((identifier) => {
          const project = issueByIdentifier.get(identifier)?.project;
          return !project || !projectIds.value.includes(projectByName.get(project) ?? '');
        }),
    ),
  ];
});
const schedulePreview = computed(() => {
  if (!validInterval.value || !startLocalTime.value || !timeZone.value)
    return { rows: [], error: '' };

  try {
    const ordinals = futureCycles.value
      .slice(0, 3)
      .map((cycle) => cycle.scheduleIndex ?? firstIndex.value);
    while (ordinals.length < 3)
      ordinals.push(Math.max(firstIndex.value, (ordinals.at(-1) ?? firstIndex.value - 1) + 1));

    const rows = ordinals.map((ordinal) => {
      const nextStart = offsetDate(
        start.value.toString(),
        (ordinal - firstIndex.value) * intervalDays.value,
      );
      const nextEnd = offsetDate(nextStart, Number(duration.value) - 1);
      const boundary = cycleBoundaryPreview(nextStart, startLocalTime.value, timeZone.value);
      const current = futureCycles.value.find((cycle) => cycle.scheduleIndex === ordinal);

      return { ordinal, nextStart, nextEnd, boundary, current };
    });
    return { rows, error: '' };
  } catch (error) {
    return { rows: [], error: String(error) };
  }
});
const rolloverOptions = [
  { label: 'Ask me', value: 'ask' },
  { label: 'Move unfinished work', value: 'automatic' },
  { label: 'Leave work in the old cycle', value: 'leave' },
];
const startDay = computed(() =>
  new Intl.DateTimeFormat(undefined, { weekday: 'long', timeZone: 'UTC' }).format(
    new Date(`${start.value.toString()}T00:00:00Z`),
  ),
);

function save() {
  if (
    !name.value.trim() ||
    !Number.isInteger(Number(duration.value)) ||
    Number(duration.value) < 1 ||
    Number(duration.value) > 56 ||
    !validInterval.value ||
    !schedulePreview.value.rows.length ||
    affectedIssues.value.length > 0
  )
    return;
  emit('save', {
    id: settings?.id ?? '',
    version: settings?.version,
    name: name.value.trim(),
    durationDays: Number(duration.value),
    intervalDays: intervalDays.value,
    enabled: settings?.enabled ?? true,
    startDate: start.value.toString(),
    rolloverPolicy: rolloverPolicy.value,
    projectIds: projectIds.value,
    timeZone: timeZone.value,
    startLocalTime: startLocalTime.value,
  });
}
</script>

<template>
  <Dialog
    v-model:open="open"
    :title="settings ? 'Edit cycle schedule' : 'Set up cycles'"
    description="Choose a repeating cycle schedule for this workspace."
    size="lg"
    class="workbench-surface"
    :ui="cycleDialogUi"
  >
    <template #header="{ close, titleId }">
      <CycleDialogHeader
        :title="settings ? 'Edit cycle schedule' : 'Set up cycles'"
        :title-id="titleId"
        :close="close"
      />
    </template>
    <template #body>
      <div
        class="mb-4 flex size-9 items-center justify-center rounded-lg bg-muted text-foreground-muted"
      >
        <Icon name="i-tabler-calendar-repeat" class="size-5" />
      </div>
      <h3 class="text-lg font-semibold text-foreground-highlighted">A predictable rhythm</h3>
      <p class="mt-1 max-w-lg text-sm leading-5 text-foreground-muted">
        Upcoming cycles are created automatically. Choose what happens to unfinished work when a
        cycle ends.
      </p>
      <div class="mt-5 divide-y divide-outline border-t border-outline">
        <div class="flex min-h-12 items-center justify-between gap-4 py-2">
          <span class="text-sm text-foreground">Series name</span>
          <Input v-model="name" aria-label="Cycle series name" size="sm" class="w-48" />
        </div>
        <div class="flex min-h-12 items-center justify-between gap-4 py-2">
          <span class="text-sm text-foreground">Each cycle lasts (days)</span>
          <Input
            v-model="duration"
            type="number"
            min="1"
            max="56"
            aria-label="Cycle duration"
            size="sm"
            class="w-20"
          />
        </div>
        <div class="flex min-h-12 items-center justify-between gap-4 py-2">
          <span class="text-sm text-foreground">Repeat every</span>
          <div class="flex items-center gap-2">
            <Input
              v-model="interval"
              type="number"
              min="1"
              aria-label="Recurrence interval"
              size="sm"
              class="w-20"
            />
            <Select
              v-model="intervalUnit"
              :items="intervalOptions"
              aria-label="Recurrence unit"
              variant="pill"
              size="sm"
            />
          </div>
        </div>
        <div class="flex min-h-12 items-center justify-between gap-4 py-2">
          <div>
            <p class="text-sm text-foreground">
              {{ settings ? 'Next cycle starts' : 'First cycle starts' }}
            </p>
            <p class="text-xs text-foreground-muted">Starts {{ startDay }}</p>
          </div>
          <DatePicker
            v-model="start"
            label="Start date"
            calendar-label="Choose first cycle start date"
          />
        </div>
        <div class="flex min-h-12 items-center justify-between gap-4 py-2">
          <span class="text-sm text-foreground">Start time</span>
          <div class="text-right">
            <Input
              v-model="startLocalTime"
              type="time"
              aria-label="Cycle start time"
              size="sm"
              class="w-32"
            />
            <p v-if="startLocalTime" class="mt-1 text-xs text-foreground-muted">
              {{ formatCycleTime(startLocalTime) }}
            </p>
          </div>
        </div>
        <div class="flex min-h-12 items-center justify-between gap-4 py-2">
          <span class="text-sm text-foreground">Time zone</span>
          <Select
            v-model="timeZone"
            :items="timeZoneOptions"
            aria-label="Cycle time zone"
            searchable
            size="sm"
            class="w-56"
          />
        </div>
        <div class="flex min-h-12 items-center justify-between gap-4 py-2">
          <span class="text-sm text-foreground">Unfinished work</span>
          <Select
            v-model="rolloverPolicy"
            :items="rolloverOptions"
            aria-label="Cycle rollover policy"
            variant="pill"
            size="sm"
          />
        </div>
        <div v-if="projects.length" class="py-3">
          <p class="mb-2 text-sm text-foreground">Projects in these cycles</p>
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
      </div>

      <section class="mt-5 border-t border-outline pt-4" aria-label="Schedule preview">
        <div class="flex items-baseline justify-between gap-3">
          <h4 class="text-sm font-medium text-foreground">Schedule preview</h4>
          <span class="text-xs text-foreground-muted">{{ timeZone }}</span>
        </div>
        <p class="mt-1 text-xs text-foreground-muted">
          First three upcoming cycles. Existing dates stay unchanged until you save.
        </p>
        <p v-if="schedulePreview.error" class="mt-3 text-xs text-error" role="alert">
          {{ schedulePreview.error }}
        </p>
        <div v-else class="mt-3 divide-y divide-outline rounded-lg border border-outline">
          <div
            v-for="row in schedulePreview.rows"
            :key="row.ordinal"
            class="flex items-start justify-between gap-4 px-3 py-2 text-xs"
          >
            <div>
              <p class="font-medium text-foreground">Cycle {{ row.ordinal + 1 }}</p>
              <p v-if="row.current" class="mt-1 text-foreground-muted">
                Current: {{ formatCycleDateRange(row.current.startDate, row.current.endDate) }} ·
                {{ formatCycleTime(settings?.startLocalTime || '09:00') }}
              </p>
            </div>
            <div class="text-right">
              <p class="text-foreground">
                {{ formatCycleDateRange(row.nextStart, row.nextEnd) }} ·
                {{ formatCycleTime(row.boundary.localTime) }}
              </p>
              <p class="mt-1 text-foreground-muted">
                {{ formatCycleInstant(row.boundary.instant) }}
                <span v-if="row.boundary.resolution">· {{ row.boundary.resolution }}</span>
              </p>
            </div>
          </div>
        </div>
        <p v-if="affectedIssues.length" class="mt-3 text-xs text-error" role="alert">
          {{ affectedIssues.length }} assigned
          {{ affectedIssues.length === 1 ? 'issue is' : 'issues are' }}
          outside the selected projects. Move them before saving:
          {{ affectedIssues.join(', ') }}
        </p>
      </section>
    </template>
    <template #footer>
      <span class="mr-auto text-xs text-foreground-muted">12 upcoming cycles</span>
      <Button
        :label="settings ? 'Save schedule' : 'Start cycles'"
        size="sm"
        :disabled="
          saving ||
          !name.trim() ||
          !Number(duration) ||
          Number(duration) > 56 ||
          !validInterval ||
          !schedulePreview.rows.length ||
          affectedIssues.length > 0 ||
          (projects.length > 0 && projectIds.length === 0)
        "
        @click="save"
      />
    </template>
  </Dialog>
</template>
