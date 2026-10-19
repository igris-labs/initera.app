<script setup lang="ts">
import type { NavigationMenuItem } from 'akaza-ui';
import { computed, ref, resolveComponent } from 'vue';
import { Button, buttonVariants } from '../components/ui/button';
import { NavigationMenu } from '../components/ui/navigation-menu';

const route = useRoute();
const theme = useCookie<'dark' | 'light'>('docs-theme', { default: () => 'dark' });
const mobileMenuOpen = ref(false);
const headerCta = buttonVariants({ variant: 'solid', size: 'sm' });
const nuxtLink = resolveComponent('NuxtLink');
const shell = 'mx-auto w-full max-w-[82rem] px-4 sm:px-6';
const brand =
  'inline-flex shrink-0 items-center gap-3 text-[1.18rem] font-bold tracking-[-0.055em] text-foreground-highlighted no-underline';
const footerLink =
  'text-xs text-foreground-muted no-underline transition-colors hover:text-foreground-highlighted';

const navigationItems = computed<NavigationMenuItem[]>(() => [
  {
    label: 'Features',
    to: '/#features',
    as: nuxtLink,
    active: route.path === '/' && route.hash === '#features',
  },
  {
    label: 'How it works',
    to: '/#workflow',
    as: nuxtLink,
    active: route.path === '/' && route.hash === '#workflow',
  },
  { label: 'Docs', to: '/getting-started/quick-start', as: nuxtLink },
  { label: 'About', to: '/about', as: nuxtLink, active: route.path === '/about' },
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
    <header class="sticky top-0 z-30 border-b border-outline bg-default/90 backdrop-blur-lg">
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
            list: 'flex items-center gap-1',
            link: 'flex min-h-8 items-center rounded-md px-3 text-xs font-medium text-foreground-muted transition-colors hover:bg-accented hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-info data-akaza-active:bg-accented data-akaza-active:text-foreground-highlighted',
          }"
        />

        <div class="flex items-center gap-3">
          <Button
            :icon="theme === 'light' ? 'i-tabler-moon' : 'i-tabler-sun'"
            variant="ghost"
            size="icon"
            :aria-label="theme === 'light' ? 'Use dark theme' : 'Use light theme'"
            @click="toggleTheme"
          />
          <NuxtLink
            to="/download"
            :class="[headerCta, 'max-[900px]:hidden']"
            :aria-current="route.path === '/download' ? 'page' : undefined"
            >Download</NuxtLink
          >
          <Button
            :icon="mobileMenuOpen ? 'i-tabler-x' : 'i-tabler-menu-2'"
            variant="ghost"
            size="icon"
            class="min-[901px]:hidden"
            :aria-label="mobileMenuOpen ? 'Close menu' : 'Open menu'"
            :aria-expanded="mobileMenuOpen"
            aria-controls="marketing-mobile-nav"
            @click="mobileMenuOpen = !mobileMenuOpen"
          />
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
          list: 'flex w-full flex-col gap-1',
          item: 'w-full',
          link: 'flex min-h-10 w-full items-center rounded-md px-3 text-sm font-medium text-foreground-muted transition-colors hover:bg-accented hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-info data-akaza-active:bg-accented data-akaza-active:text-foreground-highlighted',
        }"
        @select="mobileMenuOpen = false"
      />
    </header>

    <main id="main-content"><slot /></main>

    <footer class="border-t border-outline bg-muted">
      <div :class="[shell, 'flex flex-col justify-between gap-12 py-16 sm:py-20 md:flex-row']">
        <div>
          <NuxtLink to="/" :class="brand">
            <img
              src="/initera-symbol.svg"
              alt=""
              class="h-[1.9rem] w-[1.35rem] dark:brightness-0 dark:invert"
            />
            <span>Initera</span>
          </NuxtLink>
          <p class="mt-4 max-w-64 text-xs leading-7 text-foreground-muted">
            Projects, issues, cycles, and time tracking on your desktop.
          </p>
        </div>
        <div class="flex gap-12 sm:gap-24">
          <div class="flex min-w-28 flex-col gap-3">
            <h2 class="mb-2 text-xs font-semibold text-foreground-highlighted">Explore</h2>
            <NuxtLink to="/#features" :class="footerLink">Features</NuxtLink>
            <NuxtLink to="/download" :class="footerLink">Download</NuxtLink>
            <NuxtLink to="/about" :class="footerLink">About</NuxtLink>
          </div>
          <div class="flex min-w-28 flex-col gap-3">
            <h2 class="mb-2 text-xs font-semibold text-foreground-highlighted">Learn</h2>
            <NuxtLink to="/getting-started/quick-start" :class="footerLink">Quick start</NuxtLink>
            <NuxtLink to="/reference/keyboard-shortcuts" :class="footerLink">Shortcuts</NuxtLink>
            <NuxtLink to="/reference/backups" :class="footerLink">Backups</NuxtLink>
          </div>
        </div>
      </div>
      <div
        :class="[
          shell,
          'flex flex-col justify-between gap-4 border-t border-outline py-5 text-[0.72rem] text-foreground-dimmed sm:flex-row',
        ]"
      >
        <span>© {{ new Date().getFullYear() }} Rahul Vashishtha · MIT License</span>
        <a
          href="https://igrislabs.com"
          target="_blank"
          rel="noopener noreferrer"
          class="underline underline-offset-4"
          >Igris Labs</a
        >
      </div>
    </footer>
  </div>
</template>
