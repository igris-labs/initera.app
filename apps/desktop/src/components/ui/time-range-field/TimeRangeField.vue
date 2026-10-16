<script setup lang="ts">
import { timeRangeFieldUi } from './index';
import { TimeRangeField as AkazaTimeRangeField } from 'akaza-ui';
import type { TimeRangeFieldProps } from 'akaza-ui';
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
  TimeRangeFieldProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, selectOnFocus, autoAdvance }));

const ui = computed(() => timeRangeFieldUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaTimeRangeField v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaTimeRangeField>
</template>
