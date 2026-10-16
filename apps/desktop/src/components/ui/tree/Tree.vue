<script setup lang="ts" generic="T extends object">
import { treeUi } from './index';
import { Tree as AkazaTree } from 'akaza-ui';
import type { TreeProps } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const {
  size,
  variant,
  class: customClass,
  ui: customUi,
  expandOnClick = true,
  typeahead = true,
  transition = true,
  ...forwardedProps
} = defineProps<
  Omit<TreeProps<T>, 'size' | 'variant' | 'class' | 'ui'> & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, expandOnClick, typeahead, transition }));

const ui = computed(() => treeUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaTree v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaTree>
</template>
