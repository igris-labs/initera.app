<script setup lang="ts">
import { AppLayout } from 'akaza-ui';
import { onMounted, onUnmounted, ref, watch } from 'vue';
import DocsSearch from '../components/docs/DocsSearch.vue';
import DocsSidebar from '../components/docs/DocsSidebar.vue';
import { Button } from '../components/ui/button';
import { Drawer } from '../components/ui/drawer';
import { Icon } from '../components/ui/icon';
import { Sidebar } from '../components/ui/sidebar';

const route = useRoute();
const { data: docsNavigation } = await useDocsNavigation();
const theme = useCookie<'dark' | 'light'>('docs-theme', { default: () => 'dark' });
const searchOpen = ref(false);
const mobileNavOpen = ref(false);
const desktopNavOpen = ref(true);
const desktopNavCollapsed = ref(false);
const homePath = computed(() => docsNavigation.value?.[0]?.items[0]?.path ?? '/');
const currentPageTitle = computed(
  () =>
    docsNavigation.value?.flatMap((group) => group.items).find((item) => item.path === route.path)
      ?.title ?? 'User guide',
);
const currentPageCategory = computed(
  () =>
    docsNavigation.value?.find((group) => group.items.some((item) => item.path === route.path))
      ?.title ?? 'Documentation',
);

useHead({ htmlAttrs: { class: theme } });

function onGlobalKeydown(event: KeyboardEvent) {
  if (event.key.toLowerCase() !== 'k' || !(event.metaKey || event.ctrlKey)) return;
  event.preventDefault();
  searchOpen.value = true;
}

onMounted(() => window.addEventListener('keydown', onGlobalKeydown));
onUnmounted(() => window.removeEventListener('keydown', onGlobalKeydown));

watch(
  () => route.path,
  () => {
    mobileNavOpen.value = false;
  },
);

function toggleTheme() {
  theme.value = theme.value === 'dark' ? 'light' : 'dark';
}
</script>

<template>
  <AppLayout
    scroll="page"
    sticky-header
    sticky-sides
    :ui="{
      root: 'min-h-dvh bg-default text-foreground',
      header: 'z-20 flex h-12 items-center border-b border-outline bg-default',
      left: 'hidden lg:block',
      main: 'min-w-0',
    }"
  >
    <template #header>
      <div class="flex h-full w-full items-center gap-2 px-3 lg:w-60 lg:border-r lg:border-outline">
        <Button
          icon="i-tabler-layout-sidebar-left-expand"
          variant="ghost"
          size="icon"
          class="lg:hidden"
          aria-label="Open navigation"
          @click="mobileNavOpen = true"
        />
        <NuxtLink
          to="/"
          class="flex items-center gap-2 text-sm font-semibold text-foreground-highlighted"
        >
          <img src="/initera-symbol.svg" alt="" class="h-4 w-3 dark:brightness-0 dark:invert" />
          <span>Initera</span>
        </NuxtLink>
        <NuxtLink
          :to="homePath"
          class="border-l border-outline pl-2 text-sm font-medium text-foreground-muted"
        >
          Docs
        </NuxtLink>
        <Button
          icon="i-tabler-search"
          variant="ghost"
          size="icon"
          class="ml-auto"
          aria-label="Search documentation"
          @click="searchOpen = true"
        />
      </div>

      <div class="hidden min-w-0 items-center gap-2 px-6 text-xs text-foreground-muted lg:flex">
        <span>{{ currentPageCategory }}</span>
        <Icon name="i-tabler-chevron-right" size="xs" />
        <span class="text-foreground-highlighted">{{ currentPageTitle }}</span>
      </div>

      <Button
        :icon="theme === 'dark' ? 'i-tabler-sun' : 'i-tabler-moon'"
        variant="ghost"
        size="icon"
        class="ml-auto mr-3"
        :aria-label="theme === 'dark' ? 'Use light theme' : 'Use dark theme'"
        @click="toggleTheme"
      />
    </template>

    <template #left>
      <Sidebar
        v-model:open="desktopNavOpen"
        v-model:collapsed="desktopNavCollapsed"
        :mobile="false"
        size="md"
        aria-label="Documentation sidebar"
        class="h-[calc(100dvh-3rem)]"
        :ui="{ panel: 'border-r border-outline bg-default', header: 'hidden' }"
      >
        <DocsSidebar />
      </Sidebar>
    </template>

    <slot />

    <Drawer
      v-model="mobileNavOpen"
      title="Documentation"
      side="bottom"
      class="h-[min(85dvh,44rem)] w-full overflow-hidden rounded-t-xl border-l-0 border-t border-outline bg-default"
      :ui="{
        header: 'flex items-center justify-between px-4 py-3',
        title: 'text-sm font-semibold text-foreground-highlighted',
        body: 'min-h-0 flex-1 overflow-y-auto p-0',
      }"
    >
      <template #handle>
        <div class="flex justify-center pb-1 pt-2">
          <span class="h-1 w-10 rounded-full bg-foreground-muted/40" />
        </div>
      </template>
      <template #header="{ close, titleId }">
        <h2 :id="titleId" class="text-sm font-semibold text-foreground-highlighted">
          Documentation
        </h2>
        <Button
          icon="i-tabler-x"
          variant="ghost"
          size="icon"
          aria-label="Close navigation"
          @click="close"
        />
      </template>
      <template #body>
        <DocsSidebar @navigate="mobileNavOpen = false" />
      </template>
    </Drawer>

    <DocsSearch v-model:open="searchOpen" />
  </AppLayout>
</template>
