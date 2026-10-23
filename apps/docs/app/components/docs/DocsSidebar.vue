<script setup lang="ts">
import type { NavigationMenuItem } from 'akaza-ui';
import { computed, resolveComponent } from 'vue';
import { Icon } from '../ui/icon';
import { NavigationMenu } from '../ui/navigation-menu';

const emit = defineEmits<{
  navigate: [];
}>();

const route = useRoute();
const { data: docsNavigation } = await useDocsNavigation();
const nuxtLink = resolveComponent('NuxtLink');

const items = computed<NavigationMenuItem[]>(() =>
  (docsNavigation.value ?? []).flatMap((group) => [
    { value: `group-${group.title}`, label: group.title, type: 'label' },
    ...group.items.map((item) => ({
      value: item.path,
      label: item.title,
      to: item.path,
      as: nuxtLink,
      iconName: item.icon,
    })),
  ]),
);
</script>

<template>
  <NavigationMenu
    :items="items"
    :model-value="route.path"
    orientation="vertical"
    content-mode="inline"
    aria-label="Documentation"
    :ui="{
      root: 'h-full overflow-y-auto px-3 py-5',
      list: 'flex w-full flex-col items-stretch gap-0.5 [&>li:has(>.akaza-navigation-menu-group-label):not(:first-child)]:mt-5',
      item: 'w-full',
      link: 'flex w-full min-h-8 items-center gap-2 rounded-md px-2 text-xs text-foreground-muted outline-none transition-colors hover:bg-accented hover:text-foreground-highlighted focus-visible:ring-2 focus-visible:ring-info data-akaza-active:bg-accented data-akaza-active:font-medium data-akaza-active:text-foreground-highlighted',
      groupLabel: 'px-2 pb-1.5 pt-1 text-xs font-semibold text-foreground-muted',
      itemLeading: 'shrink-0',
    }"
    @select="emit('navigate')"
  >
    <template #item-leading="{ item }">
      <Icon :name="String(item.iconName)" size="sm" />
    </template>
  </NavigationMenu>
</template>
