<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { notifications } from '../../lib/notifications';
import { useCyclesStore } from '../../stores/cycles';
import { Select } from '../ui/select';

interface Props {
  issueId: string;
}

const { issueId } = defineProps<Props>();
const cycles = useCyclesStore();

const options = computed(() => [
  { label: 'No cycle', value: 'none', icon: 'i-tabler-minus' },
  ...cycles.items
    .filter((cycle) => cycles.statusOf(cycle) !== 'completed')
    .sort((a, b) => a.startDate.localeCompare(b.startDate))
    .map((cycle) => ({
      label: `${cycle.title} · ${cycle.startDate}`,
      value: cycle.id,
      icon: 'i-tabler-calendar-time',
    })),
]);

const selected = computed(() =>
  cycles.items.find(
    (cycle) => cycles.statusOf(cycle) !== 'completed' && cycle.issueIds.includes(issueId),
  ),
);

onMounted(() => void cycles.bootstrap());

async function changeCycle(value: string) {
  try {
    if (value === 'none' && selected.value) {
      await cycles.setIssue(selected.value.id, issueId, false);
    } else if (value !== 'none') {
      await cycles.setIssue(value, issueId, true);
    }
  } catch (error) {
    notifications.add({
      title: 'Could not change cycle',
      description: String(error),
      type: 'error',
    });
  }
}
</script>

<template>
  <Select
    :model-value="selected?.id ?? 'none'"
    :items="options"
    aria-label="Cycle"
    :display-label="selected ? undefined : 'Add to cycle'"
    placeholder-icon="i-tabler-calendar-time"
    variant="ghost"
    size="sm"
    searchable
    search-placeholder="Find a cycle..."
    class="w-full"
    @update:model-value="changeCycle"
  />
</template>
