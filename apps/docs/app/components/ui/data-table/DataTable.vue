<script setup lang="ts" generic="T extends object">
import { dataTableUi } from './index';
import { DataTable as AkazaDataTable } from 'akaza-ui';
import type { DataTableProps } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const {
  size,
  variant,
  class: customClass,
  ui: customUi,
  showPagination = true,
  ...forwardedProps
} = defineProps<
  Omit<DataTableProps<T>, 'size' | 'variant' | 'class' | 'ui'> & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, showPagination }));

const ui = computed(() => dataTableUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaDataTable v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaDataTable>
</template>
