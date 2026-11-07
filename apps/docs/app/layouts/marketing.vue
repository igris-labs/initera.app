<script setup lang="ts">
import type { NavigationMenuItem } from 'akaza-ui';
import { computed, ref, resolveComponent } from 'vue';
import { Button, buttonVariants } from '../components/ui/button';
import { Icon } from '../components/ui/icon';
import { NavigationMenu } from '../components/ui/navigation-menu';

const route = useRoute();
const theme = useCookie<'dark' | 'light'>('docs-theme', { default: () => 'dark' });
const mobileMenuOpen = ref(false);
const headerCta = buttonVariants({ variant: 'solid', size: 'sm' }).root();
const nuxtLink = resolveComponent('NuxtLink');
const shell = 'mx-auto w-full max-w-[82rem] px-4 sm:px-6';
const brand =
  'inline-flex shrink-0 items-center gap-3 text-[1.18rem] font-bold tracking-[-0.055em] text-foreground-highlighted no-underline';
const footerLink =
  'text-sm text-foreground-muted no-underline transition-colors hover:text-foreground-highlighted';

const footerGroups = [
  {
    title: 'Product',
    links: [
      { label: 'Features', to: '/product' },
      { label: 'Local first', to: '/local-first' },
      { label: 'Open source', to: '/open-source' },
      { label: 'Download', to: '/download' },
    ],
  },
  {
    title: 'Guides',
    links: [
      { label: 'Documentation', to: '/getting-started/quick-start' },
      { label: 'Keyboard shortcuts', to: '/reference/keyboard-shortcuts' },
      { label: 'Backups and restore', to: '/reference/backups' },
    ],
  },
  {
    title: 'Project',
    links: [
      { label: 'Changelog', to: '/changelog' },
      { label: 'About', to: '/about' },
      { label: 'GitHub', to: 'https://github.com/igris-labs/initera.app', external: true },
      {
        label: 'Contribute',
        to: 'https://github.com/igris-labs/initera.app/blob/main/CONTRIBUTING.md',
        external: true,
      },
      {
        label: 'Report an issue',
        to: 'https://github.com/igris-labs/initera.app/issues',
        external: true,
      },
    ],
  },
];

const navigationItems = computed<NavigationMenuItem[]>(() => [
  {
    label: 'Product',
    value: 'product',
    active: ['/product', '/local-first', '/open-source'].includes(route.path),
    children: [
      { label: 'Features', to: '/product', as: nuxtLink },
      { label: 'Local first', to: '/local-first', as: nuxtLink },
      { label: 'Open source', to: '/open-source', as: nuxtLink },
    ],
  },
  { label: 'Docs', to: '/getting-started/quick-start', as: nuxtLink },
  { label: 'Changelog', to: '/changelog', as: nuxtLink, active: route.path === '/changelog' },
]);

const mobileNavigationItems = computed<NavigationMenuItem[]>(() => [
  ...navigationItems.value,
  { label: 'Download', to: '/download', as: nuxtLink, active: route.path === '/download' },
]);

useHead({ htmlAttrs: { class: theme } });

watch(
  () => route.path,
  () => {
    mobileMenuOpen.value = false;
  },
);

function toggleTheme() {
  theme.value = theme.value === 'light' ? 'dark' : 'light';
}
</script>

<template>
  <div class="min-h-dvh bg-default text-foreground">
    <header class="sticky top-0 z-30 bg-default/90 backdrop-blur-lg">
      <div :class="[shell, 'flex min-h-18 items-center justify-between gap-8']">
        <NuxtLink to="/" :class="brand" aria-label="Initera home">
          <img
            src="/initera-symbol.svg"
            alt=""
            class="h-[1.9rem] w-[1.35rem] dark:brightness-0 dark:invert"
          />
          <span>Initera</span>
        </NuxtLink>

        <NavigationMenu
          :key="route.fullPath"
          :items="navigationItems"
          aria-label="Main navigation"
          :ui="{
            root: 'max-[900px]:hidden',
            list: 'flex items-center gap-1 data-[ipex-highlight-active=true]:before:opacity-0',
            trigger:
              'flex min-h-9 items-center gap-1.5 rounded-md px-3 text-[13px] font-medium text-foreground-muted transition-colors hover:bg-accented hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-info data-akaza-active:bg-accented data-akaza-active:text-foreground-highlighted',
            link: 'flex min-h-9 items-center rounded-md px-3 text-[13px] font-medium text-foreground-muted transition-colors hover:bg-accented hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-info data-akaza-active:bg-accented data-akaza-active:text-foreground-highlighted',
            content: 'w-48 min-w-0 rounded-lg border border-outline bg-default p-1 shadow-xl',
            viewport: 'w-full overflow-hidden',
            panel: 'w-full max-w-none p-0',
            contentList: 'grid gap-0.5',
            contentItem: 'w-full',
            contentLink:
              'flex min-h-9 w-full flex-col items-start justify-center rounded-md px-3 py-2 text-left text-[13px] whitespace-nowrap text-foreground-muted before:hidden hover:bg-accented hover:text-foreground-highlighted data-akaza-highlighted:bg-accented data-akaza-highlighted:text-foreground-highlighted data-[akaza-state=checked]:bg-accented data-[akaza-state=checked]:text-foreground-highlighted',
          }"
        />

        <div class="flex items-center gap-3">
          <a
            href="https://github.com/igris-labs/initera.app"
            target="_blank"
            rel="noopener noreferrer"
            :class="buttonVariants({ variant: 'ghost', size: 'md', shape: 'icon' }).root()"
            aria-label="Initera on GitHub"
          >
            <Icon name="i-tabler-brand-github" size="sm" />
          </a>
          <Button
            variant="ghost"
            size="md"
            shape="icon"
            :aria-label="theme === 'light' ? 'Use dark theme' : 'Use light theme'"
            @click="toggleTheme"
            ><Icon :name="theme === 'light' ? 'i-tabler-moon' : 'i-tabler-sun'" size="sm"
          /></Button>
          <NuxtLink
            to="/download"
            :class="[headerCta, 'max-[900px]:hidden']"
            :aria-current="route.path === '/download' ? 'page' : undefined"
            >Download</NuxtLink
          >
          <Button
            variant="ghost"
            size="md"
            shape="icon"
            class="min-[901px]:hidden"
            :aria-label="mobileMenuOpen ? 'Close menu' : 'Open menu'"
            :aria-expanded="mobileMenuOpen"
            aria-controls="marketing-mobile-nav"
            @click="mobileMenuOpen = !mobileMenuOpen"
            ><Icon :name="mobileMenuOpen ? 'i-tabler-x' : 'i-tabler-menu-2'" size="sm"
          /></Button>
        </div>
      </div>

      <NavigationMenu
        v-if="mobileMenuOpen"
        id="marketing-mobile-nav"
        :key="route.fullPath"
        :items="mobileNavigationItems"
        orientation="vertical"
        content-mode="inline"
        aria-label="Mobile navigation"
        :ui="{
          root: 'border-t border-outline px-6 pb-4 pt-2 min-[901px]:hidden',
          list: 'flex w-full flex-col gap-1 data-[ipex-highlight-active=true]:before:opacity-0',
          item: 'w-full',
          trigger:
            'flex min-h-10 w-full items-center rounded-md px-3 text-sm font-medium text-foreground-muted transition-colors hover:bg-accented hover:text-foreground-highlighted',
          link: 'flex min-h-10 w-full items-center rounded-md px-3 text-sm font-medium text-foreground-muted transition-colors hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-info data-akaza-active:bg-accented data-akaza-active:text-foreground-highlighted',
          inlineList: 'ml-3 border-l border-outline pl-2',
          inlineLink:
            'flex min-h-9 items-center rounded-md px-3 text-sm text-foreground-muted hover:bg-accented hover:text-foreground-highlighted',
        }"
        @select="mobileMenuOpen = false"
      />
    </header>

    <main id="main-content"><slot /></main>

    <footer class="bg-muted">
      <div
        :class="[shell, 'grid gap-12 py-16 sm:py-20 lg:grid-cols-[minmax(0,1fr)_auto] lg:gap-20']"
      >
        <div>
          <NuxtLink to="/" :class="brand">
            <img
              src="/initera-symbol.svg"
              alt=""
              class="h-[1.9rem] w-[1.35rem] dark:brightness-0 dark:invert"
            />
            <span>Initera</span>
          </NuxtLink>
          <p class="mt-4 max-w-72 text-sm leading-6 text-foreground-muted">
            Plan projects, track issues, and review progress in one desktop app.
          </p>
        </div>
        <div class="grid grid-cols-2 gap-x-8 gap-y-10 sm:grid-cols-3 sm:gap-x-12">
          <div
            v-for="group in footerGroups"
            :key="group.title"
            class="flex min-w-32 flex-col gap-3"
          >
            <h2 class="mb-1 text-xs font-semibold text-foreground-highlighted">
              {{ group.title }}
            </h2>
            <NuxtLink
              v-for="link in group.links"
              :key="link.to"
              :to="link.to"
              :external="link.external"
              :target="link.external ? '_blank' : undefined"
              :rel="link.external ? 'noopener noreferrer' : undefined"
              :class="footerLink"
            >
              {{ link.label }}
            </NuxtLink>
          </div>
        </div>
      </div>
      <div
        :class="[
          shell,
          'flex flex-col justify-between gap-4 py-5 text-[0.72rem] text-foreground-dimmed sm:flex-row',
        ]"
      >
        <span>
          © {{ new Date().getFullYear() }} Initera · Created by
          <a
            href="https://rahulv.dev"
            target="_blank"
            rel="noopener noreferrer"
            class="hover:text-foreground-highlighted"
            >Rahul Vashishtha</a
          >
          at
          <a
            href="https://igrislabs.com"
            target="_blank"
            rel="noopener noreferrer"
            class="hover:text-foreground-highlighted"
            >Igris Labs</a
          >
        </span>
        <div class="flex items-center gap-2">
          <a
            href="https://github.com/igris-labs/initera.app/blob/main/LICENSE"
            target="_blank"
            rel="noopener noreferrer"
            :class="footerLink"
            >MIT License</a
          >
          <a
            href="https://x.com/rahulv_dev"
            target="_blank"
            rel="noopener noreferrer"
            :class="[footerLink, 'inline-flex rounded-md p-2 hover:bg-accented']"
            aria-label="Rahul Vashishtha on X"
          >
            <Icon name="i-tabler-brand-x" size="sm" />
          </a>
        </div>
      </div>
    </footer>
  </div>
</template>
