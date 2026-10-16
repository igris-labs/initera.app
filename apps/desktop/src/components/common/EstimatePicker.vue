<script setup lang="ts">
import { computed } from 'vue';
import { estimateLabel, estimateOptions } from '../../lib/estimates';
import { usePreferencesStore } from '../../stores/preferences';
import type { Estimate } from '../../types/issue';
import { Select } from '../ui/select';
import type { SelectVariants } from '../ui/select';

interface Props {
  modelValue?: Estimate | null;
  variant?: SelectVariants['variant'];
  size?: SelectVariants['size'];
}

defineOptions({ inheritAttrs: false });

const { modelValue, variant = 'ghost', size = 'sm' } = defineProps<Props>();

const emit = defineEmits<{ 'update:modelValue': [value: Estimate | null] }>();
const preferences = usePreferencesStore();

const options = computed(() => {
  const choices = estimateOptions(preferences.estimateScale).map((option, index) => ({
    label: option.label,
    value: String(option.value),
    icon: 'i-tabler-hash',
    shortcut: String(index + 1),
  }));
  if (modelValue != null && !choices.some((option) => option.value === String(modelValue))) {
    choices.unshift({
      label: estimateLabel(modelValue, preferences.estimateScale),
      value: String(modelValue),
      icon: 'i-tabler-hash',
      shortcut: '',
    });
  }

  return [
    { label: 'No estimate', value: 'none', icon: 'i-tabler-minus', shortcut: '0' },
    ...choices,
  ];
});
</script>

<template>
  <Select
    v-bind="$attrs"
    :model-value="modelValue == null ? 'none' : String(modelValue)"
    :items="options"
    :variant="variant"
    :size="size"
    :display-label="modelValue == null ? 'Estimate' : undefined"
    placeholder-icon="i-tabler-hash"
    @update:model-value="
      emit('update:modelValue', $event === 'none' ? null : (Number($event) as Estimate))
    "
  />
</template>
