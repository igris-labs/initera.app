<script setup lang="ts">
import { hoverPreviewCardUi } from './index';
import { HoverPreviewCard as AkazaHoverPreviewCard } from 'akaza-ui';
import type { HoverPreviewCardProps } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const {
  size,
  variant,
  class: customClass,
  ui: customUi,
  openOnClick = true,
  ...forwardedProps
} = defineProps<
  HoverPreviewCardProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, openOnClick }));

const ui = computed(() => hoverPreviewCardUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaHoverPreviewCard v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaHoverPreviewCard>
</template>
