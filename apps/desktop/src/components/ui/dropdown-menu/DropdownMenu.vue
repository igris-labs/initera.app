<script setup lang="ts">
import { menuUi } from '../menu';
import { Menu as AkazaMenu } from 'akaza-ui';
import type { MenuItem } from 'akaza-ui';
import { computed } from 'vue';
import { cn } from '../../../lib/utils';
import { dropdownMenuDefaults, dropdownMenuVariants } from '.';
import type { DropdownMenuVariants } from '.';

interface Props {
  items: MenuItem[];
  variant?: DropdownMenuVariants['variant'];
  size?: DropdownMenuVariants['size'];
  class?: string;
}

const { items, variant, size, class: customClass } = defineProps<Props>();

const ui = computed(() => {
  const menu = menuUi(size ?? dropdownMenuDefaults.size);
  return {
    ...menu,
    root: 'flex w-full',
    content: cn(menu.content, dropdownMenuVariants({ variant, size }), customClass),
    submenuContent: cn(menu.submenuContent, 'top-auto bottom-0 ml-3 min-w-56 whitespace-nowrap'),
  };
});
</script>

<template>
  <AkazaMenu :items="items" :ui="ui">
    <template #trigger="{ toggle, triggerProps }"
      ><slot :toggle="toggle" :trigger-props="triggerProps"
    /></template>
    <template v-for="(_, slotName) in $slots" #[slotName]="slotProps">
      <slot v-if="slotName !== 'default'" :name="slotName" v-bind="slotProps || {}" />
    </template>
  </AkazaMenu>
</template>
