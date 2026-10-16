<script setup lang="ts">
import { dateRangeFieldUi } from './index';
import { DateRangeField as AkazaDateRangeField } from 'akaza-ui';
import type { DateRangeFieldProps } from 'akaza-ui';
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
  DateRangeFieldProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, selectOnFocus, autoAdvance }));

const ui = computed(() => dateRangeFieldUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaDateRangeField v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaDateRangeField>
</template>
