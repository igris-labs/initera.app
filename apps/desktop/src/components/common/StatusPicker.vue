<script setup lang="ts">
import { computed } from 'vue';
import { projectStatuses } from '../../lib/project-presentation';
import { useIssuesStore } from '../../stores/issues';
import { Select } from '../ui/select';
import type { SelectVariants } from '../ui/select';

interface Props {
  modelValue: string;
  scope: 'issue' | 'project';
  variant?: SelectVariants['variant'];
  size?: SelectVariants['size'];
}

defineOptions({ inheritAttrs: false });

const { modelValue, scope, variant = 'ghost', size = 'sm' } = defineProps<Props>();

const emit = defineEmits<{ 'update:modelValue': [value: string] }>();

const issues = useIssuesStore();

const options = computed(() =>
  scope === 'project'
    ? projectStatuses
    : [
        ...issues.statusGroups,
        ...(issues.statusGroups.includes(modelValue) ? [] : [modelValue]),
      ].map((status, index) => ({
        label: status,
        value: status,
        icon: issues.iconFor(status),
        iconClass: issues.colorFor(status),
        shortcut: String(index + 1),
      })),
);
</script>

<template>
  <Select
    v-bind="$attrs"
    :model-value="modelValue"
    :items="options"
    :variant="variant"
    :size="size"
    @update:model-value="emit('update:modelValue', $event)"
  />
</template>
