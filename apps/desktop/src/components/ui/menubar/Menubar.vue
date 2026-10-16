<script setup lang="ts">
import { menubarUi } from './index';
import { Menubar as AkazaMenubar } from 'akaza-ui';
import type { MenubarProps } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const {
  size,
  variant,
  class: customClass,
  ui: customUi,
  loop = true,
  closeOnSelect = true,
  ...forwardedProps
} = defineProps<
  MenubarProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, loop, closeOnSelect }));

const ui = computed(() => menubarUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaMenubar v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaMenubar>
</template>
