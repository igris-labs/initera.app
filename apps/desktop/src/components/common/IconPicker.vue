<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import {
  defaultProjectIcon,
  iconChoices,
  iconColor,
  iconColors,
} from '../../lib/icon-presentation';
import type { IconColor } from '../../lib/icon-presentation';
import { Icon } from '../ui/icon';
import { Input } from '../ui/input';
import { Popover } from '../ui/popover';

interface Props {
  ariaLabel?: string;
  size?: 'sm' | 'md';
}

const { ariaLabel = 'Choose icon and color', size = 'md' } = defineProps<Props>();

const icon = defineModel<string>('icon', { required: true });
const color = defineModel<IconColor>('color', { required: true });
const open = ref(false);
const search = ref('');
const searchInput = ref<{ inputRef?: HTMLInputElement } | null>(null);

const groups = ['Objects', 'Planning', 'Work'] as const;

const matches = computed(() =>
  iconChoices.filter((choice) =>
    `${choice.label} ${choice.group}`.toLowerCase().includes(search.value.trim().toLowerCase()),
  ),
);

watch(open, async (value) => {
  if (!value) return;
  search.value = '';
  await nextTick();
  searchInput.value?.inputRef?.focus();
});

function selectIcon(value: string) {
  icon.value = value;
}

function selectColor(value: IconColor) {
  color.value = value;
}
</script>

<template>
  <Popover
    v-model:open="open"
    side="bottom"
    align="start"
    teleport="body"
    size="md"
    class="workbench-surface"
  >
    <template #trigger="{ toggle, triggerProps }">
      <button
        v-bind="triggerProps"
        type="button"
        :aria-label="ariaLabel"
        :title="ariaLabel"
        :class="[
          'flex shrink-0 items-center justify-center rounded-md border border-transparent transition-colors hover:border-outline hover:bg-accented/60 focus-visible:outline-2 focus-visible:outline-info',
          size === 'sm' ? 'size-8' : 'size-9',
        ]"
        :style="{
          backgroundColor: `color-mix(in oklch, ${iconColor(color).swatch} 13%, transparent)`,
        }"
        @click="toggle"
      >
        <Icon :name="icon || defaultProjectIcon" :class="['size-5', iconColor(color).text]" />
      </button>
    </template>
    <template #content>
      <div class="min-w-0 w-full">
        <Input
          ref="searchInput"
          v-model="search"
          aria-label="Search icons"
          placeholder="Search icons…"
          size="sm"
          class="mb-2 min-w-0 w-full"
        />
        <div class="max-h-49 space-y-2 overflow-y-auto pr-1">
          <div
            v-for="group in groups"
            v-show="matches.some((choice) => choice.group === group)"
            :key="group"
          >
            <p class="mb-1 px-1 text-xs text-foreground-dimmed">{{ group }}</p>
            <div class="grid min-w-0 grid-cols-6 gap-1">
              <button
                v-for="choice in matches.filter((item) => item.group === group)"
                :key="choice.name"
                type="button"
                :aria-label="choice.label"
                :aria-pressed="icon === choice.name"
                :title="choice.label"
                :class="[
                  'flex size-8 items-center justify-center rounded-md border border-transparent transition-colors hover:bg-accented focus-visible:outline-2 focus-visible:outline-info',
                  icon === choice.name && 'border-outline bg-accented',
                ]"
                @click="selectIcon(choice.name)"
              >
                <Icon :name="choice.name" :class="['size-4', iconColor(color).text]" />
              </button>
            </div>
          </div>
          <p v-if="!matches.length" class="px-1 py-3 text-xs text-foreground-muted">
            No icons found
          </p>
        </div>
        <div class="mt-2 border-t border-outline pt-2">
          <p class="mb-1.5 px-1 text-xs text-foreground-muted">Color</p>
          <div class="flex flex-wrap gap-1">
            <button
              v-for="(choice, value) in iconColors"
              :key="value"
              type="button"
              :aria-label="`${choice.label} color`"
              :aria-pressed="color === value"
              :title="choice.label"
              :class="[
                'flex size-7 items-center justify-center rounded-md border border-transparent transition-colors hover:bg-accented focus-visible:outline-2 focus-visible:outline-info',
                color === value && 'border-outline bg-accented',
              ]"
              @click="selectColor(value)"
            >
              <span class="size-3.5 rounded-full" :style="{ backgroundColor: choice.swatch }" />
            </button>
          </div>
        </div>
      </div>
    </template>
  </Popover>
</template>
