<script setup lang="ts">
import { ref } from 'vue';
import { Button, buttonVariants } from '../components/ui/button';
import { Icon } from '../components/ui/icon';

const route = useRoute();
const theme = useCookie<'dark' | 'light'>('docs-theme', { default: () => 'light' });
const mobileMenuOpen = ref(false);
const headerCta = buttonVariants({ variant: 'solid', size: 'sm' });

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

        <nav class="marketing-desktop-nav" aria-label="Main navigation">
          <NuxtLink to="/#features">Features</NuxtLink>
          <NuxtLink to="/#workflow">How it works</NuxtLink>
          <NuxtLink to="/getting-started/quick-start">Docs</NuxtLink>
          <NuxtLink to="/about">About</NuxtLink>
        </nav>

        <div class="marketing-header-actions">
          <Button
            :icon="theme === 'light' ? 'i-tabler-moon' : 'i-tabler-sun'"
            variant="ghost"
            size="icon"
            :aria-label="theme === 'light' ? 'Use dark theme' : 'Use light theme'"
            @click="toggleTheme"
          />
          <NuxtLink to="/download" :class="[headerCta, 'marketing-header-cta']"
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

      <nav
        v-if="mobileMenuOpen"
        id="marketing-mobile-nav"
        class="marketing-mobile-nav"
        aria-label="Mobile navigation"
      >
        <NuxtLink to="/#features" @click="mobileMenuOpen = false">Features</NuxtLink>
        <NuxtLink to="/#workflow" @click="mobileMenuOpen = false">How it works</NuxtLink>
        <NuxtLink to="/getting-started/quick-start">Docs</NuxtLink>
        <NuxtLink to="/about">About</NuxtLink>
        <NuxtLink to="/download"
          >Get started <Icon name="i-tabler-arrow-up-right" size="sm"
        /></NuxtLink>
      </nav>
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
