<script setup lang="ts">
import { toolbarUi } from './index';
import { Toolbar as AkazaToolbar } from 'akaza-ui';
import type { ToolbarProps } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const {
  size,
  variant,
  class: customClass,
  ui: customUi,
  loop = true,
  ...forwardedProps
} = defineProps<
  ToolbarProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, loop }));

const ui = computed(() => toolbarUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaToolbar v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaToolbar>
</template>
