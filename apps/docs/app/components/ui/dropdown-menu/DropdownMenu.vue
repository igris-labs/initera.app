<script setup lang="ts">
import { Menu } from '../menu';
import type { MenuItem } from 'akaza-ui';
import { computed } from 'vue';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import { dropdownMenuDefaults, dropdownMenuVariants } from '.';
import type { DropdownMenuVariants } from '.';

interface Props {
  items: MenuItem[];
  variant?: DropdownMenuVariants['variant'];
  size?: DropdownMenuVariants['size'];
  class?: string;
}

const { items, variant, size, class: customClass } = defineProps<Props>();

const resolvedSize = computed(
  () =>
    size ??
    uiConfig.components?.dropdownMenu?.size ??
    (['sm', 'md', 'lg'] as const).find((value) => value === uiConfig.defaults?.size) ??
    dropdownMenuDefaults.size,
);
const resolvedVariant = computed(
  () =>
    variant ??
    uiConfig.components?.dropdownMenu?.variant ??
    (['default', 'quiet'] as const).find((value) => value === uiConfig.defaults?.variant) ??
    dropdownMenuDefaults.variant,
);

const ui = computed(() => {
  return {
    root: 'flex w-full',
    content: cn(
      dropdownMenuVariants({ variant: resolvedVariant.value, size: resolvedSize.value }),
      customClass,
    ),
    submenuContent: 'top-auto bottom-0 ml-1',
  };
});
</script>

<template>
  <Menu :items="items" :size="resolvedSize" :ui="ui">
    <template #trigger="{ toggle, triggerProps }"
      ><slot :toggle="toggle" :trigger-props="triggerProps"
    /></template>
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot v-if="slotName !== 'default'" :name="slotName" v-bind="slotProps || {}" />
    </template>
  </Menu>
</template>
