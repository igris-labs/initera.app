<script setup lang="ts">
import { drawerUi } from './index';
import { Drawer as AkazaDrawer } from 'akaza-ui';
import type { DrawerProps } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const {
  size,
  variant,
  class: customClass,
  ui: customUi,
  closeOnBackdropClick = true,
  swipeToClose = true,
  ...forwardedProps
} = defineProps<
  DrawerProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, closeOnBackdropClick, swipeToClose }));

const ui = computed(() => drawerUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaDrawer v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaDrawer>
</template>
