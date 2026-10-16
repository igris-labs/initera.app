<script setup lang="ts">
import { navigationMenuUi } from './index';
import { NavigationMenu as AkazaNavigationMenu } from 'akaza-ui';
import type { NavigationMenuProps } from 'akaza-ui';
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
  NavigationMenuProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, loop }));

const ui = computed(() => navigationMenuUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaNavigationMenu v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaNavigationMenu>
</template>
