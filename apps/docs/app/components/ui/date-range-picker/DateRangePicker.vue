<script setup lang="ts">
import { dateRangePickerUi } from './index';
import { DateRangePicker as AkazaDateRangePicker } from 'akaza-ui';
import type { DateRangePickerProps } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const {
  size,
  variant,
  class: customClass,
  ui: customUi,
  selectOnFocus = true,
  autoAdvance = true,
  ...forwardedProps
} = defineProps<
  DateRangePickerProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, selectOnFocus, autoAdvance }));

const ui = computed(() => dateRangePickerUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaDateRangePicker v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaDateRangePicker>
</template>
