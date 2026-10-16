<script setup lang="ts">
import { RouterLink, useRoute } from 'vue-router';

interface Props {
  project: string;
}

defineProps<Props>();

const route = useRoute();

const tabs = [
  { name: 'project-overview', label: 'Overview' },
  { name: 'project-issues', label: 'Issues' },
  { name: 'project-structure', label: 'Structure' },
] as const;
</script>

<template>
  <nav aria-label="Project pages" class="flex items-center gap-1 border-b border-outline px-4 py-2">
    <RouterLink
      v-for="tab in tabs"
      :key="tab.name"
      :to="{ name: tab.name, params: { project } }"
      :aria-current="route.name === tab.name ? 'page' : undefined"
      :class="[
        'rounded-md px-3 py-1.5 text-xs transition-colors',
        route.name === tab.name
          ? 'bg-(--ui-nav-active) font-medium text-foreground-highlighted'
          : 'text-foreground-muted hover:bg-accented hover:text-foreground-highlighted',
      ]"
      >{{ tab.label }}</RouterLink
    >
  </nav>
</template>
