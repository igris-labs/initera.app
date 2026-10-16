<script setup lang="ts">
import { stepperUi } from './index';
import { Stepper as AkazaStepper } from 'akaza-ui';
import type { StepperProps } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const {
  size,
  variant,
  class: customClass,
  ui: customUi,
  linear = true,
  ...forwardedProps
} = defineProps<
  StepperProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, linear }));

const ui = computed(() => stepperUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaStepper v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaStepper>
</template>
