<script setup lang="ts">
import { tagsInputUi } from './index';
import { TagsInput as AkazaTagsInput } from 'akaza-ui';
import type { TagsInputProps } from 'akaza-ui';
import { computed } from 'vue';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

defineOptions({ inheritAttrs: false });

const {
  size,
  variant,
  class: customClass,
  ui: customUi,
  trim = true,
  ...forwardedProps
} = defineProps<
  TagsInputProps & {
    size?: PrimitiveSize;
    variant?: PrimitiveVariant;
    class?: string;
    ui?: Record<string, string>;
  }
>();

const akazaProps = computed(() => ({ ...forwardedProps, trim }));

const ui = computed(() => tagsInputUi(size, variant, customClass, customUi));
</script>

<template>
  <AkazaTagsInput v-bind="{ ...akazaProps, ...$attrs }" :ui="ui">
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaTagsInput>
</template>
