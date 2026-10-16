<script setup lang="ts">
import { useWorkspaceStore } from '../../stores/workspace';
import { Button } from '../ui/button';
import { Icon } from '../ui/icon';

interface Props {
  title: string;
  label?: string;
  titleId: string;
  close: () => void;
  cycle?: string;
}

const { title, label, titleId, close, cycle } = defineProps<Props>();
const workspace = useWorkspaceStore();
</script>

<template>
  <div class="flex min-w-0 items-center gap-2 text-xs text-foreground-muted">
    <h2 :id="titleId" class="sr-only">{{ title }}</h2>
    <span class="max-w-48 truncate rounded-md bg-muted px-2 py-1 font-medium text-foreground">
      {{ workspace.workspaceName }}
    </span>
    <Icon name="i-tabler-chevron-right" class="size-3.5 shrink-0" />
    <template v-if="cycle">
      <span class="max-w-32 truncate">{{ cycle }}</span>
      <Icon name="i-tabler-chevron-right" class="size-3.5 shrink-0" />
    </template>
    <span class="truncate">{{ label ?? title }}</span>
  </div>
  <Button icon="i-tabler-x" variant="ghost" size="icon" aria-label="Close dialog" @click="close" />
</template>
