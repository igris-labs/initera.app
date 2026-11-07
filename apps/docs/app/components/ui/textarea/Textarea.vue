<script setup lang="ts">
import { computed } from 'vue';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import { textareaDefaults, textareaVariants } from '.';
import type { TextareaVariants } from '.';

interface Props {
  modelValue?: string;
  size?: TextareaVariants['size'];
  variant?: TextareaVariants['variant'];
  class?: string;
}

const { modelValue = '', size, variant, class: customClass } = defineProps<Props>();

const emit = defineEmits<{ 'update:modelValue': [value: string] }>();

const resolvedSize = computed(
  () =>
    size ??
    uiConfig.components?.textarea?.size ??
    (['sm', 'md', 'lg'] as const).find((value) => value === uiConfig.defaults?.size) ??
    textareaDefaults.size,
);
const resolvedVariant = computed(
  () =>
    variant ??
    uiConfig.components?.textarea?.variant ??
    (['outline', 'soft', 'bare'] as const).find((value) => value === uiConfig.defaults?.variant) ??
    textareaDefaults.variant,
);
const classes = computed(() =>
  cn(textareaVariants({ size: resolvedSize.value, variant: resolvedVariant.value }), customClass),
);
</script>

<template>
  <textarea
    :value="modelValue"
    :class="classes"
    @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
  />
</template>
