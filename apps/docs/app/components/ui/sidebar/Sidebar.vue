<script setup lang="ts">
import { Sidebar as AkazaSidebar } from 'akaza-ui';
import type { SidebarUi } from 'akaza-ui';
import { computed } from 'vue';
import { cn } from '../../../lib/utils';
import { sidebarDefaults, sidebarVariants, sidebarWidths } from '.';
import type { SidebarVariants } from '.';

interface Props {
  ariaLabel?: string;
  mobile?: boolean;
  variant?: SidebarVariants['variant'];
  size?: SidebarVariants['size'];
  class?: string;
  ui?: SidebarUi;
}

const {
  ariaLabel = 'Sidebar',
  mobile,
  variant,
  size,
  class: customClass,
  ui: uiOverrides,
} = defineProps<Props>();

const open = defineModel<boolean>('open', { required: true });
const collapsed = defineModel<boolean>('collapsed', { required: true });

const width = computed(() => sidebarWidths[size ?? sidebarDefaults.size]);

const ui = computed(() => ({
  root: cn('min-h-0', customClass, uiOverrides?.root),
  panel: cn(sidebarVariants({ variant, size }), uiOverrides?.panel),
  header: cn('flex h-(--ui-header-height) items-center gap-2 px-2', uiOverrides?.header),
  body: cn('min-h-0', uiOverrides?.body),
  footer: cn('p-2', uiOverrides?.footer),
  mobileOverlay: cn('bg-scrim', uiOverrides?.mobileOverlay),
  mobileContent: cn('bg-shell', uiOverrides?.mobileContent),
  mobileWrapper: cn('h-full', uiOverrides?.mobileWrapper),
}));
</script>

<template>
  <AkazaSidebar
    v-model:open="open"
    v-model:collapsed="collapsed"
    collapsible="offcanvas"
    :aria-label="ariaLabel"
    :mobile="mobile"
    :default-size="width"
    :mobile-size="width"
    :ui="ui"
  >
    <template #header="slotProps"><slot name="header" v-bind="slotProps" /></template>
    <template #default="slotProps"><slot v-bind="slotProps" /></template>
    <template #footer="slotProps"><slot name="footer" v-bind="slotProps" /></template>
  </AkazaSidebar>
</template>
