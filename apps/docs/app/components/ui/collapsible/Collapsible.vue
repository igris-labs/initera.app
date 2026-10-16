<script setup lang="ts">
import { collapsibleUi } from './index';
import { Collapsible as AkazaCollapsible } from 'akaza-ui';
import type { CollapsibleProps } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const props = defineProps<
  CollapsibleProps & {
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

const ui = computed(() => collapsibleUi(props.size, props.variant, props.class, props.ui));
</script>

<template>
  <AkazaCollapsible v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaCollapsible>
</template>
