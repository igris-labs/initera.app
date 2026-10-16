<script setup lang="ts">
import { Icon } from '../ui/icon';
</script>

<template>
  <div class="docs-feature-grid my-6 grid gap-3 sm:grid-cols-2">
    <NuxtLink
      to="/issues/issues"
      class="rounded-md border border-outline bg-elevated p-4 outline-none transition-colors hover:bg-accented focus-visible:ring-2 focus-visible:ring-info"
    >
      <span
        class="mb-3 flex size-7 items-center justify-center rounded-md bg-muted text-foreground-muted"
      >
        <Icon name="i-tabler-list-details" size="sm" />
      </span>
      <strong class="text-sm font-medium text-foreground-highlighted">Issues</strong>
      <p class="mt-1 text-xs leading-5 text-foreground-muted">
        Capture tasks, set priorities, and follow progress.
      </p>
    </NuxtLink>

    <NuxtLink
      to="/projects/projects"
      class="rounded-md border border-outline bg-elevated p-4 outline-none transition-colors hover:bg-accented focus-visible:ring-2 focus-visible:ring-info"
    >
      <span
        class="mb-3 flex size-7 items-center justify-center rounded-md bg-muted text-foreground-muted"
      >
        <Icon name="i-tabler-cube" size="sm" />
      </span>
      <strong class="text-sm font-medium text-foreground-highlighted">Projects</strong>
      <p class="mt-1 text-xs leading-5 text-foreground-muted">
        Organize related work with milestones and updates.
      </p>
    </NuxtLink>
  </div>
</template>
