<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { labelSwatch } from '../../lib/label-presentation';
import { useLabelsStore } from '../../stores/labels';
import { Button } from '../ui/button';
import { Checkbox } from '../ui/checkbox';
import { Input } from '../ui/input';
import { Popover } from '../ui/popover';
import { Icon } from '../ui/icon';

interface Props {
  compact?: boolean;
  pill?: boolean;
}

const { compact = false, pill = false } = defineProps<Props>();

const labels = defineModel<string[]>({ required: true });
const registry = useLabelsStore();
const open = ref(false);
const search = ref('');
const searchInput = ref<{ inputRef?: HTMLInputElement } | null>(null);

const options = computed(() =>
  [...new Set([...registry.assignable.map((label) => label.name), ...labels.value])].filter(
    (label) => label.toLowerCase().includes(search.value.toLowerCase()),
  ),
);

async function createSearchedLabel() {
  const name = search.value.trim();
  if (!name) return;
  await registry.create(name, '#999aa3');
  toggleLabel(name);
}

function toggleLabel(label: string) {
  labels.value = labels.value.includes(label)
    ? labels.value.filter((item) => item !== label)
    : [...labels.value, label];
}

watch(open, async (value) => {
  if (!value) return;
  search.value = '';
  await nextTick();
  searchInput.value?.inputRef?.focus();
});
</script>

<template>
  <Popover v-model:open="open" size="sm" align="start" teleport="body" class="workbench-surface">
    <template #trigger="{ toggle, triggerProps }">
      <Button
        v-if="compact"
        v-bind="triggerProps"
        :icon="labels.length ? 'i-tabler-plus' : 'i-tabler-tag'"
        :label="labels.length ? undefined : 'Add label'"
        aria-label="Add label"
        variant="ghost"
        size="sm"
        :square="!!labels.length"
        @click="toggle"
      />
      <Button
        v-else
        v-bind="triggerProps"
        :icon="labels.length ? undefined : 'i-tabler-tag'"
        :aria-label="labels.length ? `Labels: ${labels.join(', ')}` : 'Add labels'"
        :variant="pill ? 'outline' : 'soft'"
        size="sm"
        :class="pill ? 'dark:bg-muted dark:hover:bg-accented' : undefined"
        @click="toggle"
      >
        <span v-if="labels.length > 1" class="flex -space-x-1" aria-hidden="true">
          <span
            v-for="label in labels.slice(0, 3)"
            :key="label"
            class="size-3 rounded-full border-2 border-default"
            :style="{ backgroundColor: labelSwatch(label) }"
          />
        </span>
        <span
          v-else-if="labels.length === 1"
          class="size-2 rounded-full"
          :style="{ backgroundColor: labelSwatch(labels[0]!) }"
          aria-hidden="true"
        />
        {{ labels.length > 1 ? `${labels.length} labels` : labels[0] || 'Labels' }}
      </Button>
    </template>
    <template #content>
      <div class="p-1">
        <Input
          ref="searchInput"
          v-model="search"
          aria-label="Search labels"
          placeholder="Add labels..."
          class="mb-1"
        />
        <div class="max-h-36 overflow-auto">
          <Checkbox
            v-for="label in options"
            :key="label"
            :model-value="labels.includes(label)"
            :label="label"
            size="sm"
            :ui="{
              wrapper: `flex min-h-7 w-full cursor-pointer items-center gap-2 rounded-md px-2 text-left hover:bg-accented focus-within:bg-accented ${labels.includes(label) ? 'bg-accented/50' : ''}`,
              root: 'order-2 ml-auto size-5 rounded-none border-0 bg-transparent data-[akaza-state=checked]:border-transparent data-[akaza-state=checked]:bg-transparent',
              indicator: 'text-foreground-highlighted',
              text: 'order-1 min-w-0 flex-1',
              label: 'flex items-center gap-2 text-xs font-medium text-foreground-highlighted',
            }"
            @update:model-value="toggleLabel(label)"
          >
            <template #label>
              <span class="size-2 rounded-full" :style="{ backgroundColor: labelSwatch(label) }" />
              <span class="min-w-0 flex-1 truncate">{{ label }}</span>
            </template>
            <template #indicator>
              <Icon name="i-tabler-check" class="size-4.5 stroke-[2.5]" />
            </template>
          </Checkbox>
          <p v-if="!options.length" class="px-2 py-3 text-xs text-foreground-muted">
            No labels found
          </p>
          <Button
            v-if="
              search.trim() &&
              !registry.items.some(
                (item) =>
                  !item.deletedAt && item.name.toLowerCase() === search.trim().toLowerCase(),
              )
            "
            :label="`Create ${search.trim()}`"
            icon="i-tabler-plus"
            variant="ghost"
            size="xs"
            class="w-full justify-start"
            @click="createSearchedLabel"
          />
        </div>
      </div>
    </template>
  </Popover>
</template>
