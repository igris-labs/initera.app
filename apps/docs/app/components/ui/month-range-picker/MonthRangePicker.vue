<script setup lang="ts">
import { monthRangePickerUi } from './index';
import { MonthRangePicker as AkazaMonthRangePicker } from 'akaza-ui';
import type { MonthRangePickerProps } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const props = defineProps<
  MonthRangePickerProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => {
  const { size, variant, class: customClass, ui, ...rest } = props;
  return rest;
});

const ui = computed(() => monthRangePickerUi(props.size, props.variant, props.class, props.ui));
</script>

<template>
  <AkazaMonthRangePicker v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaMonthRangePicker>
</template>
