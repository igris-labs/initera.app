<script setup lang="ts">
import { comboboxUi } from './index';
import { Combobox as AkazaCombobox } from 'akaza-ui';
import type { ComboboxProps } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const {
  size,
  variant,
  class: customClass,
  ui: customUi,
  loop = true,
  filterable = true,
  resetSearchOnSelect = true,
  openOnFocus = true,
  openOnClick = true,
  openOnInput = true,
  highlightOnHover = true,
  ...forwardedProps
} = defineProps<
  ComboboxProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({
  ...forwardedProps,
  loop,
  filterable,
  resetSearchOnSelect,
  openOnFocus,
  openOnClick,
  openOnInput,
  highlightOnHover,
}));

const ui = computed(() => comboboxUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaCombobox v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaCombobox>
</template>
