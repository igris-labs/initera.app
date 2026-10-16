<script setup lang="ts">
import { dateFieldUi } from './index';
import { calendarUi } from '../calendar';
import { DateField as AkazaDateField } from 'akaza-ui';
import type { DateFieldProps, DateFieldUi } from 'akaza-ui';
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
  DateFieldProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: DateFieldUi;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, selectOnFocus, autoAdvance }));

const ui = computed(() => {
  const { calendar, ...overrides } = customUi ?? {};
  return {
    ...dateFieldUi(size, variant, customClass, overrides),
    calendar: {
      ...calendarUi(size),
      ...calendar,
    },
  };
});
</script>

<template>
  <AkazaDateField v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaDateField>
</template>
