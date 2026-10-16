<script setup lang="ts">
import { contextMenuUi } from './index';
import { ContextMenu as AkazaContextMenu } from 'akaza-ui';
import type { ContextMenuProps } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const {
  size,
  variant,
  class: customClass,
  ui: customUi,
  closeOnSelect = true,
  restoreFocus = true,
  ...forwardedProps
} = defineProps<
  ContextMenuProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, closeOnSelect, restoreFocus }));

const ui = computed(() => contextMenuUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaContextMenu v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaContextMenu>
</template>
