<script setup lang="ts">
import { checkboxUi } from './index';
import { Checkbox as AkazaCheckbox } from 'akaza-ui';
import type { CheckboxProps, CheckboxValue } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const {
  size,
  variant,
  class: customClass,
  ui: customUi,
  trueValue = true,
  falseValue = false,
  ...forwardedProps
} = defineProps<
  CheckboxProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const model = defineModel<CheckboxValue>({ default: false });

const akazaProps = computed(() => ({ ...forwardedProps, trueValue, falseValue }));

const ui = computed(() => checkboxUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaCheckbox v-bind="{ ...akazaProps, ...$attrs }" v-model="model" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaCheckbox>
</template>
