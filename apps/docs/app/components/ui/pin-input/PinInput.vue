<script setup lang="ts">
import { pinInputUi } from './index';
import { PinInput as AkazaPinInput } from 'akaza-ui';
import type { PinInputProps } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const {
  size,
  variant,
  class: customClass,
  ui: customUi,
  selectOnFocus = true,
  ...forwardedProps
} = defineProps<
  PinInputProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, selectOnFocus }));

const ui = computed(() => pinInputUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaPinInput v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaPinInput>
</template>
