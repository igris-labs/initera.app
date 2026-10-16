<script setup lang="ts">
import type { NavigationMenuItem } from 'akaza-ui';
import { computed, ref, resolveComponent } from 'vue';
import { Button, buttonVariants } from '../components/ui/button';
import { NavigationMenu } from '../components/ui/navigation-menu';

const route = useRoute();
const theme = useCookie<'dark' | 'light'>('docs-theme', { default: () => 'light' });
const mobileMenuOpen = ref(false);
const headerCta = buttonVariants({ variant: 'solid', size: 'sm' });
const nuxtLink = resolveComponent('NuxtLink');

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
  { label: 'Get started', to: '/download', as: nuxtLink, active: route.path === '/download' },
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
  <div class="marketing-site">
    <header class="marketing-header">
      <div class="marketing-shell marketing-header-inner">
        <NuxtLink to="/" class="marketing-brand" aria-label="Initera home">
          <img src="/initera-symbol.svg" alt="" class="dark:brightness-0 dark:invert" />
          <span>Initera</span>
        </NuxtLink>

        <NavigationMenu
          :key="route.fullPath"
          :items="navigationItems"
          aria-label="Main navigation"
          :ui="{
            root: 'marketing-desktop-nav',
            list: 'flex items-center gap-1',
            link: 'flex min-h-8 items-center rounded-md px-3 text-xs font-medium text-foreground-muted transition-colors hover:bg-accented hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-info data-akaza-active:bg-accented data-akaza-active:text-foreground-highlighted',
          }"
        />

        <div class="marketing-header-actions">
          <Button
            :icon="theme === 'light' ? 'i-tabler-moon' : 'i-tabler-sun'"
            variant="ghost"
            size="icon"
            :aria-label="theme === 'light' ? 'Use dark theme' : 'Use light theme'"
            @click="toggleTheme"
          />
          <NuxtLink
            to="/download"
            :class="[headerCta, 'marketing-header-cta']"
            :aria-current="route.path === '/download' ? 'page' : undefined"
            >Get started</NuxtLink
          >
          <Button
            :icon="mobileMenuOpen ? 'i-tabler-x' : 'i-tabler-menu-2'"
            variant="ghost"
            size="icon"
            class="marketing-menu-toggle"
            :aria-label="mobileMenuOpen ? 'Close menu' : 'Open menu'"
            :aria-expanded="mobileMenuOpen"
            aria-controls="marketing-mobile-nav"
            @click="mobileMenuOpen = !mobileMenuOpen"
          />
        </div>
      </div>

      <NavigationMenu
        v-if="mobileMenuOpen"
        :key="route.fullPath"
        id="marketing-mobile-nav"
        :items="mobileNavigationItems"
        orientation="vertical"
        content-mode="inline"
        aria-label="Mobile navigation"
        :ui="{
          root: 'marketing-mobile-nav',
          list: 'flex w-full flex-col gap-1',
          item: 'w-full',
          link: 'flex min-h-10 w-full items-center rounded-md px-3 text-sm font-medium text-foreground-muted transition-colors hover:bg-accented hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-info data-akaza-active:bg-accented data-akaza-active:text-foreground-highlighted',
        }"
        @select="mobileMenuOpen = false"
      />
    </header>

    <main id="main-content"><slot /></main>

    <footer class="marketing-footer">
      <div class="marketing-shell marketing-footer-main">
        <div class="marketing-footer-brand">
          <NuxtLink to="/" class="marketing-brand">
            <img src="/initera-symbol.svg" alt="" class="dark:brightness-0 dark:invert" />
            <span>Initera</span>
          </NuxtLink>
          <p>A local desktop workspace for the work you want to move forward.</p>
        </div>
        <div class="marketing-footer-links">
          <div>
            <h2>Explore</h2>
            <NuxtLink to="/#features">Features</NuxtLink>
            <NuxtLink to="/download">Run Initera</NuxtLink>
            <NuxtLink to="/about">About</NuxtLink>
          </div>
          <div>
            <h2>Learn</h2>
            <NuxtLink to="/getting-started/quick-start">Quick start</NuxtLink>
            <NuxtLink to="/reference/keyboard-shortcuts">Shortcuts</NuxtLink>
            <NuxtLink to="/reference/backups">Backups</NuxtLink>
          </div>
        </div>
      </div>
      <div class="marketing-shell marketing-footer-bottom">
        <span>© {{ new Date().getFullYear() }} Rahul Vashishtha · MIT License</span>
        <span
          >Made at
          <a href="https://igrislabs.com" target="_blank" rel="noopener noreferrer"
            >Igris Labs</a
          ></span
        >
      </div>
    </footer>
  </div>
</template>
