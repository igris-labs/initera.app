<script setup lang="ts">
import { Popover as AkazaPopover } from 'akaza-ui';
import { computed } from 'vue';
import { cn } from '../../../lib/utils';
import { popoverVariants } from '.';
import type { PopoverVariants } from '.';

interface Props {
  class?: string;
  variant?: PopoverVariants['variant'];
  size?: PopoverVariants['size'];
  side?: 'top' | 'bottom' | 'left' | 'right';
  align?: 'start' | 'center' | 'end';
  teleport?: string | false;
}

const {
  side = 'bottom',
  align = 'end',
  teleport = false,
  variant,
  size,
  class: customClass,
} = defineProps<Props>();

const open = defineModel<boolean>('open', { default: false });

const ui = computed(() => ({
  root: 'inline-flex',
  content: cn(popoverVariants({ variant, size }), customClass),
}));
</script>

<template>
  <AkazaPopover v-model="open" :side="side" :align="align" :teleport="teleport" :ui="ui">
    <template #trigger="{ toggle, triggerProps }"
      ><slot name="trigger" :toggle="toggle" :trigger-props="triggerProps"
    /></template>
    <template #content><slot name="content" /></template>
  </AkazaPopover>
</template>
