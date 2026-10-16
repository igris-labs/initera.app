<script setup lang="ts">
import { numberFieldUi } from './index';
import { NumberField as AkazaNumberField } from 'akaza-ui';
import type { NumberFieldProps } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const {
  size,
  variant,
  class: customClass,
  ui: customUi,
  stepSnapping = true,
  focusOnChange = true,
  ...forwardedProps
} = defineProps<
  NumberFieldProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, stepSnapping, focusOnChange }));

const ui = computed(() => numberFieldUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaNumberField v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaNumberField>
</template>
