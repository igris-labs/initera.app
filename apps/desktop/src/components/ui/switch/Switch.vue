<script setup lang="ts">
import { Switch as AkazaSwitch } from 'akaza-ui';
import { computed } from 'vue';
import { cn } from '../../../lib/utils';
import { switchDefaults, switchVariants } from '.';
import type { SwitchVariants } from '.';

interface Props {
  modelValue: boolean;
  ariaLabel?: string;
  size?: SwitchVariants['size'];
  variant?: SwitchVariants['variant'];
  class?: string;
}

const { modelValue, ariaLabel, size, variant, class: customClass } = defineProps<Props>();

const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>();

const ui = computed(() => ({
  wrapper: cn('inline-flex', customClass),
  root: switchVariants({ size, variant }),
  thumb: cn(
    'rounded-full bg-default transition-transform duration-150 ease-out',
    (size ?? switchDefaults.size) === 'md'
      ? 'size-4 data-[akaza-state=checked]:translate-x-4'
      : 'size-3 data-[akaza-state=checked]:translate-x-4',
  ),
}));
</script>

<template>
  <AkazaSwitch
    :model-value="modelValue"
    :aria-label="ariaLabel"
    :ui="ui"
    @update:model-value="emit('update:modelValue', Boolean($event))"
  />
</template>
