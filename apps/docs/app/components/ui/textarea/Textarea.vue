<script setup lang="ts">
import { computed } from 'vue';
import { cn } from '../../../lib/utils';
import { textareaVariants } from '.';
import type { TextareaVariants } from '.';

interface Props {
  modelValue?: string;
  size?: TextareaVariants['size'];
  variant?: TextareaVariants['variant'];
  class?: string;
}

const { modelValue = '', size, variant, class: customClass } = defineProps<Props>();

const emit = defineEmits<{ 'update:modelValue': [value: string] }>();

const classes = computed(() => cn(textareaVariants({ size, variant }), customClass));
</script>

<template>
  <textarea
    :value="modelValue"
    :class="classes"
    @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
  />
</template>
