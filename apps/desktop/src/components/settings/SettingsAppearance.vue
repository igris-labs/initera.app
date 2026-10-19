<script setup lang="ts">
import { useThemeChange } from '../../composables/useThemeChange';
import type { ThemePreference } from '../../types/preferences';
import { RadioGroup } from '../ui/radio-group';

const { theme, changeTheme } = useThemeChange();
const appearances: { value: ThemePreference; label: string; description: string }[] = [
  { value: 'system', label: 'System', description: 'Follow your device appearance' },
  { value: 'light', label: 'Light', description: 'Use the light workspace palette' },
  { value: 'dark', label: 'Dark', description: 'Use the dark workspace palette' },
];

function updateAppearance(value: string, details: { event?: Event }) {
  changeTheme(value as ThemePreference, details.event?.target as Element | undefined);
}
</script>

<template>
  <section aria-labelledby="appearance-heading">
    <h3 id="appearance-heading" class="text-sm font-medium text-foreground-highlighted">
      Appearance
    </h3>
    <RadioGroup
      :model-value="theme"
      :options="appearances"
      aria-label="Appearance"
      size="sm"
      :ui="{
        root: 'mt-3 gap-1',
        item: 'min-h-12 w-full rounded-md px-3 py-2 text-left transition-colors duration-150 ease-out hover:bg-muted focus-visible:outline-2 focus-visible:outline-info',
      }"
      @value-change="updateAppearance"
    >
      <template #item="{ option, isChecked }">
        <span
          :class="[
            'flex size-4 shrink-0 items-center justify-center rounded-full border',
            isChecked ? 'border-info' : 'border-foreground-dimmed',
          ]"
          aria-hidden="true"
        >
          <span v-if="isChecked" class="size-2 rounded-full bg-info" />
        </span>
        <span class="min-w-0 flex-1">
          <span class="block text-sm font-medium text-foreground">{{ option.label }}</span>
          <span class="block text-xs text-foreground-muted">{{ option.description }}</span>
        </span>
      </template>
    </RadioGroup>
  </section>
</template>
