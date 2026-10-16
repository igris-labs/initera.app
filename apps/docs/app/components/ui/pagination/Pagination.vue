<script setup lang="ts">
import { paginationUi } from './index';
import { Pagination as AkazaPagination } from 'akaza-ui';
import type { PaginationProps } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const {
  size,
  variant,
  class: customClass,
  ui: customUi,
  showPreviousNext = true,
  ...forwardedProps
} = defineProps<
  PaginationProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, showPreviousNext }));

const ui = computed(() => paginationUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaPagination v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaPagination>
</template>
