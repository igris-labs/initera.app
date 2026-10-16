<script setup lang="ts">
import { Icon } from '../components/ui/icon';

definePageMeta({ layout: 'marketing' });

useSeoMeta({
  title: 'About Initera',
  description:
    'Initera is a local desktop project management app by Rahul Vashishtha at Igris Labs.',
});
</script>

<template>
  <div class="marketing-interior marketing-shell">
    <div class="marketing-interior-heading">
      <p class="marketing-kicker">ABOUT INITERA</p>
      <h1>Built for the work between an idea and done.</h1>
      <p>
        Initera gives projects, issues, cycles, and time records one home on your computer. It is
        written by Rahul Vashishtha under Igris Labs, his personal laboratory.
      </p>
    </div>

    <div class="marketing-about-grid">
      <div class="marketing-about-mark" aria-hidden="true">
        <img src="/initera-symbol.svg" alt="" class="dark:brightness-0 dark:invert" />
      </div>
      <div class="marketing-about-copy">
        <h2>Make the work easier to follow.</h2>
        <p>
          Start with an issue. Connect it to a project when it needs a larger plan. Add a cycle when
          the team needs a time frame. The app keeps the record close to the work, including
          updates, comments, and time entries.
        </p>
        <p>
          Initera uses Vue, Tauri, and SQLite. Your work stays on your computer unless you export a
          backup. The project is available under the MIT License.
        </p>
        <NuxtLink to="/reference/about" class="marketing-text-link">
          Product and license details <Icon name="i-tabler-arrow-right" size="sm" />
        </NuxtLink>
      </div>
    </div>
  </div>
</template>
