<script setup lang="ts">
import { priorityColor } from '../../lib/issue-presentation';
import { priorities } from '../../types/issue';
import { Select } from '../ui/select';
import type { SelectVariants } from '../ui/select';

interface Props {
  modelValue: string;
  variant?: SelectVariants['variant'];
  size?: SelectVariants['size'];
  showEmptyLabel?: boolean;
}

defineOptions({ inheritAttrs: false });

const { modelValue, variant = 'ghost', size = 'sm', showEmptyLabel = false } = defineProps<Props>();

const emit = defineEmits<{ 'update:modelValue': [value: string] }>();

const options = priorities.map((priority, index) => ({
  label: priority,
  value: priority,
  icon: 'i-tabler-antenna-bars-5',
  iconClass: priorityColor[priority],
  shortcut: String(index + 1),
}));
</script>

<template>
  <Select
    v-bind="$attrs"
    :model-value="modelValue"
    :items="options"
    :variant="variant"
    :size="size"
    :display-label="showEmptyLabel && modelValue === 'No priority' ? 'Set priority' : undefined"
    :placeholder-icon="showEmptyLabel ? 'i-tabler-dots' : undefined"
    @update:model-value="emit('update:modelValue', $event)"
  />
</template>
