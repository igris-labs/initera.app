<script setup lang="ts">
import { menuUi } from './index';
import { Menu as AkazaMenu } from 'akaza-ui';
import type { MenuProps } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const {
  size,
  variant,
  class: customClass,
  ui: customUi,
  closeOnSelect = true,
  ...forwardedProps
} = defineProps<
  MenuProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, closeOnSelect }));

const ui = computed(() => menuUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaMenu v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaMenu>
</template>
