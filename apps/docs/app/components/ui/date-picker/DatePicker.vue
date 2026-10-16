<script setup lang="ts">
import type { CalendarDate } from '@internationalized/date';
import { computed } from 'vue';
import { Icon } from '../icon';
import { DateField } from '../date-field';

interface Props {
  label: string;
  calendarLabel: string;
  modelValue?: CalendarDate;
  variant?: 'pill' | 'ghost';
}

const props = defineProps<Props>();

const emit = defineEmits<{ 'update:modelValue': [value: CalendarDate | undefined] }>();

const display = computed(() =>
  props.modelValue
    ? new Intl.DateTimeFormat(undefined, {
        month: 'short',
        day: 'numeric',
        year: 'numeric',
      }).format(new Date(props.modelValue.year, props.modelValue.month - 1, props.modelValue.day))
    : props.label,
);
</script>

<template>
  <DateField
    :model-value="modelValue"
    :calendar-label="calendarLabel"
    :aria-label="calendarLabel"
    size="sm"
    show-calendar
    :ui="{
      root: 'min-h-0 border-0 bg-transparent p-0',
      segments: 'sr-only',
      calendarTrigger:
        variant === 'ghost'
          ? 'min-h-7 w-full justify-start gap-1.5 rounded-lg border border-transparent bg-transparent px-2.5 text-xs text-foreground hover:bg-muted/75'
          : 'min-h-7 gap-1.5 rounded-lg border border-outline bg-default px-2.5 text-xs text-foreground-muted hover:bg-accented hover:text-foreground dark:bg-muted',
      calendarContent: 'z-80',
    }"
    @update:model-value="emit('update:modelValue', $event as CalendarDate | undefined)"
  >
    <template #calendar-trigger>
      <Icon name="i-tabler-calendar" class="size-3.5" />
      <span>{{ display }}</span>
    </template>
  </DateField>
</template>
