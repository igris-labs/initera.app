<script setup lang="ts">
import { Sidebar as AkazaSidebar } from 'akaza-ui';
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
}

const { ariaLabel = 'Sidebar', mobile, variant, size, class: customClass } = defineProps<Props>();

const open = defineModel<boolean>('open', { required: true });
const collapsed = defineModel<boolean>('collapsed', { required: true });

const width = computed(() => sidebarWidths[size ?? sidebarDefaults.size]);

const ui = computed(() => ({
  root: cn('min-h-0', customClass),
  panel: sidebarVariants({ variant, size }),
  header: 'flex h-(--ui-header-height) items-center gap-2 px-2',
  body: 'min-h-0',
  footer: 'p-2',
  mobileOverlay: 'bg-scrim',
  mobileContent: 'bg-shell',
  mobileWrapper: 'h-full',
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
