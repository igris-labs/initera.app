<script setup lang="ts">
import type { AppShortcut } from '../../lib/app-shortcuts';
import { shortcutHint } from '../../lib/app-shortcuts';
import { Tooltip } from '../ui/tooltip';

interface Props {
  label: string;
  shortcut?: AppShortcut;
  direction?: 'top' | 'bottom' | 'left' | 'right';
}

defineProps<Props>();
</script>

<template>
  <Tooltip :direction="direction ?? 'bottom'" :delay-duration="350">
    <template #trigger><slot /></template>
    <template #content>
      <span class="inline-flex items-center gap-3 text-xs">
        <span>{{ label }}</span>
        <kbd
          v-if="shortcut"
          class="rounded-md border border-outline bg-muted px-1.5 py-0.5 text-xs font-medium whitespace-nowrap text-foreground-highlighted"
          >{{ shortcutHint(shortcut) }}</kbd
        >
      </span>
    </template>
  </Tooltip>
</template>
