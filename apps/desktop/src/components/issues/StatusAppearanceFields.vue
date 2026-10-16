<script setup lang="ts">
import { computed, ref } from 'vue';
import { statusIconChoices, statusPalette } from '../../lib/issue-presentation';
import type { StatusColor } from '../../lib/issue-presentation';
import { Icon } from '../ui/icon';
import { Input } from '../ui/input';

const icon = defineModel<string>('icon', { required: true });
const color = defineModel<StatusColor>('color', { required: true });
const search = ref('');

const categories = ['Workflow', 'Planning', 'Work'] as const;

const matchingIcons = computed(() =>
  statusIconChoices.filter((choice) =>
    `${choice.label} ${choice.group}`.toLowerCase().includes(search.value.trim().toLowerCase()),
  ),
);

const selectedLabel = computed(
  () => statusIconChoices.find((choice) => choice.name === icon.value)?.label ?? 'Custom icon',
);
</script>

<template>
  <fieldset class="min-w-0 space-y-1.5">
    <legend class="text-xs text-foreground-muted">Icon · {{ selectedLabel }}</legend>
    <Input v-model="search" aria-label="Search group icons" placeholder="Search icons" size="sm" />
    <div class="max-h-44 space-y-2 overflow-y-auto pr-1">
      <template v-for="category in categories" :key="category">
        <div v-if="matchingIcons.some((choice) => choice.group === category)">
          <p class="mb-1 text-xs text-foreground-dimmed">{{ category }}</p>
          <div class="grid grid-cols-6 gap-1">
            <button
              v-for="choice in matchingIcons.filter((item) => item.group === category)"
              :key="choice.name"
              type="button"
              :aria-label="choice.label"
              :aria-pressed="icon === choice.name"
              :title="choice.label"
              :class="[
                'flex size-7 items-center justify-center rounded-md border transition-colors hover:bg-accented focus-visible:outline-2 focus-visible:outline-info',
                icon === choice.name ? 'border-info bg-accented' : 'border-transparent',
              ]"
              @click="icon = choice.name"
            >
              <Icon :name="choice.name" :class="['size-4', statusPalette[color]]" />
            </button>
          </div>
        </div>
      </template>
      <p v-if="!matchingIcons.length" class="py-2 text-xs text-foreground-muted">
        No matching icons
      </p>
    </div>
  </fieldset>
  <fieldset class="space-y-1.5">
    <legend class="text-xs text-foreground-muted">Color</legend>
    <div class="flex gap-1">
      <button
        v-for="(className, name) in statusPalette"
        :key="name"
        type="button"
        :aria-label="`${name} color`"
        :aria-pressed="color === name"
        :class="[
          'flex size-8 items-center justify-center rounded-md border transition-colors hover:bg-accented focus-visible:outline-2 focus-visible:outline-info',
          color === name ? 'border-info bg-accented' : 'border-outline/60',
        ]"
        @click="color = name"
      >
        <Icon name="i-tabler-circle-filled" :class="['size-4', className]" />
      </button>
    </div>
  </fieldset>
</template>
