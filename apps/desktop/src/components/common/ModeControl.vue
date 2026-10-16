<script setup lang="ts" generic="T extends string">
import { ToggleGroup } from '../ui/toggle-group';

interface Props {
  items: { label: string; value: T }[];
  label: string;
}

defineProps<Props>();

const value = defineModel<T>({ required: true });
</script>

<template>
  <ToggleGroup
    :model-value="value"
    :options="items"
    type="single"
    :allow-empty="false"
    :aria-label="label"
    size="sm"
    :ui="{
      root: 'gap-1 bg-transparent p-0',
      item: 'text-foreground-muted hover:bg-muted data-[akaza-state=on]:bg-accented data-[akaza-state=on]:text-foreground-highlighted data-[akaza-state=on]:shadow-none data-[akaza-state=on]:ring-0',
    }"
    @update:model-value="value = $event as T"
  />
</template>
