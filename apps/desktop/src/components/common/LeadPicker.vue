<script setup lang="ts">
import { computed } from 'vue';
import { useWorkspaceStore } from '../../stores/workspace';
import { Select } from '../ui/select';
import type { SelectVariants } from '../ui/select';

interface Props {
  modelValue: string | null;
  variant?: SelectVariants['variant'];
  size?: SelectVariants['size'];
}

defineOptions({ inheritAttrs: false });

const { modelValue, variant = 'ghost', size = 'sm' } = defineProps<Props>();
const emit = defineEmits<{ 'update:modelValue': [value: string | null] }>();
const workspace = useWorkspaceStore();

const options = computed(() => [
  { label: 'No lead', value: 'none', icon: 'i-tabler-circle' },
  { label: workspace.userName || 'You', value: workspace.actorId, icon: 'i-tabler-user' },
]);
</script>

<template>
  <Select
    v-bind="$attrs"
    :model-value="modelValue ?? 'none'"
    :items="options"
    :variant="variant"
    :size="size"
    :display-label="modelValue ? undefined : 'Set lead'"
    :placeholder-icon="modelValue ? undefined : 'i-tabler-user'"
    @update:model-value="emit('update:modelValue', $event === 'none' ? null : $event)"
  />
</template>
