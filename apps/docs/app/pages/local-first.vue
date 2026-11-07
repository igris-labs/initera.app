<script setup lang="ts">
import { Icon } from '../components/ui/icon';

definePageMeta({ layout: 'marketing' });
useSeoMeta({
  title: 'Local-first | Initera',
  description: 'Learn where Initera stores workspace data and how to back up or move your work.',
});
</script>

<template>
  <div class="mx-auto w-full max-w-[82rem] px-4 py-20 sm:px-6 lg:py-28">
    <p class="text-xs font-semibold tracking-widest text-foreground-muted uppercase">Local first</p>
    <h1
      class="mt-5 max-w-[52rem] text-balance text-[clamp(2.7rem,5vw,4.5rem)] leading-tight font-semibold tracking-tight text-foreground-highlighted"
    >
      A workspace you can keep.
    </h1>
    <p class="mt-6 max-w-[43rem] text-base leading-7 text-foreground-muted">
      You can start Initera without an account. The desktop app keeps your workspace data in a local
      database and stores uploaded images as files in its media folder.
    </p>

    <figure class="mt-14 overflow-hidden rounded-xl border border-outline bg-muted">
      <NuxtImg
        src="/images/app/settings.png"
        alt="Workspace settings showing the local data folder"
        width="1440"
        height="900"
        sizes="100vw lg:1280px"
        class="block h-auto w-full"
      />
    </figure>

    <div class="mt-16 grid gap-12 pt-12 lg:grid-cols-3 lg:gap-16">
      <section>
        <h2 class="text-xl font-semibold text-foreground-highlighted">Find your files</h2>
        <p class="mt-3 text-sm leading-7 text-foreground-muted">
          Workspace settings show the data folder used by this installation. The database contains
          your records; images live in a neighboring media folder.
        </p>
      </section>
      <section>
        <h2 class="text-xl font-semibold text-foreground-highlighted">Make a backup</h2>
        <p class="mt-3 text-sm leading-7 text-foreground-muted">
          Use the backup controls in Settings to save a copy. The backup includes the database and
          its images. Keep a copy somewhere separate from the computer you use every day.
        </p>
      </section>
      <section>
        <h2 class="text-xl font-semibold text-foreground-highlighted">Restore or move</h2>
        <p class="mt-3 text-sm leading-7 text-foreground-muted">
          Import the backup on another installation and review it before restoring. The backup guide
          covers the steps and what will be replaced.
        </p>
      </section>
    </div>

    <div class="mt-16 pt-8">
      <h2 class="text-xl font-semibold text-foreground-highlighted">What about sync?</h2>
      <p class="mt-3 max-w-[43rem] text-sm leading-7 text-foreground-muted">
        The current desktop release does not sync workspaces between computers. Accounts, teams, and
        a self-hosted backend are planned. Backups are the supported way to transfer a workspace
        today.
      </p>
      <NuxtLink
        to="/reference/backups"
        class="mt-5 inline-flex items-center gap-2 text-sm font-medium text-foreground-highlighted no-underline hover:underline underline-offset-4"
        >Read the backup guide <Icon name="i-tabler-arrow-right" size="sm"
      /></NuxtLink>
    </div>
  </div>
</template>
