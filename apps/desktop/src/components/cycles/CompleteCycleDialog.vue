<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { Cycle } from '../../lib/cycle-storage';
import { useIssuesStore } from '../../stores/issues';
import { Button } from '../ui/button';
import { Dialog } from '../ui/dialog';
import { Icon } from '../ui/icon';
import { Select } from '../ui/select';
import { Checkbox } from '../ui/checkbox';
import CycleDialogHeader from './CycleDialogHeader.vue';
import { cycleDialogUi } from './dialog-style';

interface Props {
  cycle: Cycle | null;
  cycles: Cycle[];
  saving?: boolean;
}

const { cycle, cycles, saving = false } = defineProps<Props>();
const emit = defineEmits<{ complete: [nextCycleId?: string, selectedIdentifiers?: string[]] }>();
const open = defineModel<boolean>('open', { required: true });
const issues = useIssuesStore();

const rolloverTo = ref('none');
const selectedIdentifiers = ref<string[]>([]);
const candidates = computed(() =>
  (cycle?.issueIds ?? []).filter((id) => {
    const issue = issues.items.find((item) => item.id === id);
    return (
      issue &&
      !issue.triage &&
      !['Completed', 'Canceled'].includes(issues.categoryFor(issue.status))
    );
  }),
);
const title = computed(() => `Complete ${cycle?.title ?? 'cycle'}`);
const options = computed(() => [
  { label: 'Leave unfinished issues in this cycle', value: 'none' },
  ...cycles
    .filter((item) => item.id !== cycle?.id && item.status !== 'completed')
    .map((item) => ({ label: item.title, value: item.id })),
]);

watch(open, (value) => {
  if (value) {
    rolloverTo.value = 'none';
    selectedIdentifiers.value = [...candidates.value];
  }
});

function toggleCandidate(id: string, checked: boolean) {
  selectedIdentifiers.value = checked
    ? [...selectedIdentifiers.value, id]
    : selectedIdentifiers.value.filter((item) => item !== id);
}

function complete() {
  emit(
    'complete',
    rolloverTo.value === 'none' ? undefined : rolloverTo.value,
    selectedIdentifiers.value,
  );
}
</script>

<template>
  <Dialog
    v-model:open="open"
    :title="title"
    description="Decide where unfinished issues should go."
    size="lg"
    class="workbench-surface"
    :ui="cycleDialogUi"
  >
    <template #header="{ close, titleId }">
      <CycleDialogHeader
        :title="title"
        label="Complete cycle"
        :title-id="titleId"
        :close="close"
        :cycle="cycle?.title"
      />
    </template>
    <template #body>
      <span
        class="flex size-9 items-center justify-center rounded-lg bg-muted text-foreground-muted"
      >
        <Icon name="i-tabler-circle-check" class="size-5" />
      </span>
      <h3 class="mt-3 text-lg font-semibold text-foreground-highlighted">
        Complete {{ cycle?.title }}
      </h3>
      <p class="mt-1 text-sm leading-5 text-foreground-muted">
        Completed and canceled issues stay here. Choose whether to move unfinished work to another
        cycle.
      </p>
      <div
        class="mt-5 flex flex-wrap items-center justify-between gap-3 border-t border-outline py-3"
      >
        <span class="text-sm text-foreground">Unfinished work</span>
        <Select
          v-model="rolloverTo"
          :items="options"
          aria-label="Rollover unfinished issues"
          variant="pill"
          size="sm"
        />
      </div>
      <div
        v-if="rolloverTo !== 'none' && candidates.length"
        class="max-h-48 space-y-1 overflow-auto border-t border-outline py-3"
      >
        <p class="mb-2 text-xs text-foreground-muted">Choose issues to move</p>
        <Checkbox
          v-for="id in candidates"
          :key="id"
          :model-value="selectedIdentifiers.includes(id)"
          :label="`${id} · ${issues.items.find((item) => item.id === id)?.title ?? 'Issue'}`"
          size="sm"
          @update:model-value="toggleCandidate(id, Boolean($event))"
        />
      </div>
    </template>
    <template #footer>
      <Button label="Cancel" variant="ghost" size="sm" class="mr-auto" @click="open = false" />
      <Button label="Complete cycle" size="sm" :disabled="saving" @click="complete" />
    </template>
  </Dialog>
</template>
