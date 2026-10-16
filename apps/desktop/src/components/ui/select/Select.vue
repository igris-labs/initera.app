<script setup lang="ts">
import { Select as AkazaSelect } from 'akaza-ui';
import { computed, ref } from 'vue';
import { cn } from '../../../lib/utils';
import { selectDefaults, selectVariants } from '.';
import type { SelectVariants } from '.';
import { Icon } from '../icon';

interface SelectItem {
  [key: string]: unknown;
  label: string;
  value: string;
  icon?: string;
  iconClass?: string;
  shortcut?: string;
}

interface Props {
  modelValue?: string;
  items: readonly (string | SelectItem)[];
  ariaLabel?: string;
  placeholder?: string;
  placeholderIcon?: string;
  displayLabel?: string;
  variant?: SelectVariants['variant'];
  size?: SelectVariants['size'];
  side?: 'top' | 'right' | 'bottom' | 'left';
  searchable?: boolean;
  searchPlaceholder?: string;
  class?: string;
}

const {
  modelValue = '',
  items,
  ariaLabel,
  placeholder,
  placeholderIcon,
  displayLabel,
  variant,
  size,
  side,
  searchable = false,
  searchPlaceholder,
  class: customClass,
} = defineProps<Props>();

const emit = defineEmits<{
  'update:modelValue': [value: string];
  'update:search': [value: string];
}>();

const isOpen = ref(false);

const resolvedSize = computed(() => size ?? selectDefaults.size);

const optionSize = {
  xs: 'min-h-6 px-2 text-xs',
  sm: 'min-h-7 px-2.5 text-xs',
  md: 'min-h-8 px-2.5 text-sm',
} as const;
const searchSize = {
  xs: 'h-6 px-2 text-xs',
  sm: 'h-7 px-2.5 text-xs',
  md: 'h-8 px-2.5 text-sm',
} as const;

const options = computed(() =>
  items.map((item) => (typeof item === 'string' ? { label: item, value: item } : item)),
);

const ui = computed(() => ({
  root: cn('min-w-0', customClass),
  trigger: selectVariants({ variant, size: resolvedSize.value }),
  content:
    'z-50 min-w-40 rounded-lg border border-outline bg-elevated p-1.5 text-foreground shadow-lg',
  viewport: 'max-h-64 overflow-auto',
  empty: cn(
    'px-2.5 py-2 text-foreground-muted',
    resolvedSize.value === 'md' ? 'text-sm leading-5' : 'text-xs leading-4',
  ),
  option: cn(
    'flex cursor-pointer items-center gap-2 rounded-md text-foreground-highlighted outline-none transition-colors duration-100 data-[akaza-state=checked]:bg-muted hover:bg-accented focus-visible:bg-accented data-akaza-highlighted:bg-accented',
    optionSize[resolvedSize.value],
  ),
  value: 'min-w-0 truncate font-medium text-foreground-highlighted',
  icon:
    variant === 'pill' || variant === 'ghost' ? 'hidden' : 'ml-auto shrink-0 text-foreground-muted',
  optionText: 'min-w-0 flex-1',
  optionLabel: cn(
    'block truncate font-medium text-foreground-highlighted',
    resolvedSize.value === 'md' ? 'leading-5' : 'leading-4',
  ),
  indicator: 'flex size-4 shrink-0 items-center justify-center text-xs text-info',
  searchInput: cn(
    'w-full border-0 border-b border-outline bg-transparent text-foreground outline-none placeholder:text-foreground-dimmed',
    searchSize[resolvedSize.value],
  ),
}));

function onShortcut(event: KeyboardEvent) {
  if (!isOpen.value || event.metaKey || event.ctrlKey || event.altKey) return;
  const option = options.value.find((item) => item.shortcut === event.key);
  if (!option) return;
  event.preventDefault();
  event.stopPropagation();
  emit('update:modelValue', option.value);
  isOpen.value = false;
}
</script>

<template>
  <AkazaSelect
    v-model:open="isOpen"
    :model-value="modelValue"
    :options="options"
    :aria-label="ariaLabel"
    :placeholder="placeholder"
    :side="side"
    :autocomplete="searchable"
    :search-placeholder="searchPlaceholder"
    :ui="ui"
    @keydown.capture="onShortcut"
    @update:model-value="emit('update:modelValue', String($event))"
    @update:search="emit('update:search', String($event))"
  >
    <template #value="{ option, label }">
      <span
        :class="['flex min-w-0 items-center truncate', variant === 'pill' ? 'gap-1.5' : 'gap-2']"
      >
        <Icon
          v-if="option?.icon || placeholderIcon"
          :name="String((displayLabel && placeholderIcon) || option?.icon || placeholderIcon)"
          :class="[
            'size-4 shrink-0',
            displayLabel || !option ? 'text-foreground-muted' : option.iconClass,
          ]"
        />
        <span :class="['truncate', displayLabel && 'text-foreground-muted']">{{
          displayLabel || label
        }}</span>
      </span>
    </template>
    <template #option="{ option, label, isSelected }">
      <Icon
        v-if="option.icon"
        :name="String(option.icon)"
        :class="['size-4 shrink-0', option.iconClass]"
      />
      <span
        :class="[
          'akaza-select-option-label min-w-0 flex-1 truncate font-medium text-foreground-highlighted',
          resolvedSize === 'md' ? 'leading-5' : 'leading-4',
        ]"
        >{{ label }}</span
      >
      <Icon v-if="isSelected" name="i-tabler-check" class="size-4 shrink-0 text-info" />
      <span v-if="option.shortcut" class="text-xs tabular-nums text-foreground-dimmed">{{
        option.shortcut
      }}</span>
    </template>
  </AkazaSelect>
</template>
