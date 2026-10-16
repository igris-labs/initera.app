<script setup lang="ts">
import { toggleGroupUi } from './index';
import { ToggleGroup as AkazaToggleGroup } from 'akaza-ui';
import type { ToggleGroupProps } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const {
  size,
  variant,
  class: customClass,
  ui: customUi,
  loop = true,
  rovingFocus = true,
  allowEmpty = true,
  ...forwardedProps
} = defineProps<
  ToggleGroupProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, loop, rovingFocus, allowEmpty }));

const ui = computed(() => toggleGroupUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaToggleGroup v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaToggleGroup>
</template>
