<script setup lang="ts">
import { Icon } from '../components/ui/icon';

definePageMeta({ layout: 'marketing' });
useSeoMeta({
  title: 'Open source | Initera',
  description:
    'Initera is open source under the MIT License. Read the code and contribute on GitHub.',
});
</script>

<template>
  <div class="mx-auto w-full max-w-[82rem] px-4 py-20 sm:px-6 lg:py-28">
    <p class="text-xs font-semibold tracking-widest text-foreground-muted uppercase">Open source</p>
    <h1
      class="mt-5 max-w-[52rem] text-balance text-[clamp(2.7rem,5vw,4.5rem)] leading-tight font-semibold tracking-tight text-foreground-highlighted"
    >
      Built in the open.
    </h1>
    <p class="mt-6 max-w-[44rem] text-base leading-7 text-foreground-muted">
      Initera's source code is public under the MIT License. You can inspect how the desktop app
      works, use it at home or at work, and help improve it.
    </p>

    <div class="mt-14 grid gap-8 pt-12 lg:grid-cols-3 lg:gap-16">
      <section>
        <h2 class="text-xl font-semibold text-foreground-highlighted">Read the source</h2>
        <p class="mt-3 text-sm leading-7 text-foreground-muted">
          The app, website, documentation, and build workflows live in one repository.
        </p>
        <a
          href="https://github.com/igris-labs/initera.app"
          target="_blank"
          rel="noopener noreferrer"
          class="mt-5 inline-flex items-center gap-2 text-sm font-medium text-foreground-highlighted hover:underline underline-offset-4"
          >GitHub repository <Icon name="i-tabler-arrow-up-right" size="sm"
        /></a>
      </section>
      <section>
        <h2 class="text-xl font-semibold text-foreground-highlighted">Report a problem</h2>
        <p class="mt-3 text-sm leading-7 text-foreground-muted">
          If something breaks or feels unclear, open an issue with the steps that led to it and the
          app version.
        </p>
        <a
          href="https://github.com/igris-labs/initera.app/issues"
          target="_blank"
          rel="noopener noreferrer"
          class="mt-5 inline-flex items-center gap-2 text-sm font-medium text-foreground-highlighted hover:underline underline-offset-4"
          >GitHub issues <Icon name="i-tabler-arrow-up-right" size="sm"
        /></a>
      </section>
      <section>
        <h2 class="text-xl font-semibold text-foreground-highlighted">Contribute</h2>
        <p class="mt-3 text-sm leading-7 text-foreground-muted">
          The contributing guide explains the workspace layout, development commands, and pull
          request process.
        </p>
        <a
          href="https://github.com/igris-labs/initera.app/blob/main/CONTRIBUTING.md"
          target="_blank"
          rel="noopener noreferrer"
          class="mt-5 inline-flex items-center gap-2 text-sm font-medium text-foreground-highlighted hover:underline underline-offset-4"
          >Contributing guide <Icon name="i-tabler-arrow-up-right" size="sm"
        /></a>
      </section>
    </div>
    <p class="mt-16 max-w-[44rem] pt-8 text-sm leading-7 text-foreground-muted">
      The current release is a local desktop app. A self-hosted backend, accounts, teams, and sync
      are on the roadmap; they are not available in the desktop release yet.
    </p>
  </div>
</template>
