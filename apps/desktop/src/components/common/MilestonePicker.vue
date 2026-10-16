<script setup lang="ts">
import { computed } from 'vue';
import { useProjectsStore } from '../../stores/projects';
import { Select } from '../ui/select';

interface Props {
  project: string;
  variant?: 'pill' | 'ghost';
  size?: 'xs' | 'sm' | 'md';
}

const { project, variant = 'ghost', size = 'sm' } = defineProps<Props>();
const model = defineModel<string | null>({ default: null });
const projects = useProjectsStore();

const milestones = computed(
  () =>
    projects.items
      .find((item) => item.name === project)
      ?.milestones.filter((item) => !item.archivedAt || item.id === model.value) ?? [],
);
const options = computed(() => [
  { label: 'No milestone', value: 'none', icon: 'i-tabler-minus' },
  ...milestones.value.map((item) => ({
    label: item.title,
    value: item.id,
    icon: 'i-tabler-diamond',
  })),
]);
</script>

<template>
  <Select
    v-if="project !== 'Unassigned' && milestones.length"
    :model-value="model ?? 'none'"
    :items="options"
    aria-label="Milestone"
    :display-label="model ? undefined : 'Add milestone'"
    placeholder-icon="i-tabler-diamond"
    :variant="variant"
    :size="size"
    searchable
    search-placeholder="Find a milestone..."
    class="w-full"
    @update:model-value="model = $event === 'none' ? null : $event"
  />
</template>
