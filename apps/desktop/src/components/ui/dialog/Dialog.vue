<script setup lang="ts">
import { Dialog as AkazaDialog } from 'akaza-ui';
import type { DialogUi } from 'akaza-ui';
import { computed } from 'vue';
import { cn } from '../../../lib/utils';
import { dialogVariants } from '.';
import type { DialogVariants } from '.';
import { Button } from '../button';

interface Props {
  title: string;
  description?: string;
  closeOnBackdropClick?: boolean;
  variant?: DialogVariants['variant'];
  size?: DialogVariants['size'];
  class?: string;
  ui?: DialogUi;
}

const {
  title,
  description,
  closeOnBackdropClick,
  variant,
  size,
  class: customClass,
  ui: uiOverrides,
} = defineProps<Props>();

const open = defineModel<boolean>('open', { required: true });

const ui = computed(() => ({
  overlay: cn('fixed inset-0 bg-scrim', uiOverrides?.overlay),
  content: cn(dialogVariants({ variant, size }), customClass, uiOverrides?.content),
  header: cn(
    'flex min-h-12 shrink-0 items-start justify-between gap-4 border-b border-outline px-4 py-3',
    uiOverrides?.header,
  ),
  description: cn('sr-only', uiOverrides?.description),
  body: cn('min-h-0 overflow-y-auto p-4', uiOverrides?.body),
  footer: cn(
    'flex shrink-0 flex-wrap items-center justify-end gap-2 border-t border-outline px-4 py-3',
    uiOverrides?.footer,
  ),
}));
</script>

<template>
  <AkazaDialog
    v-model="open"
    :ui="ui"
    :title="title"
    :description="description"
    :close-on-backdrop-click="closeOnBackdropClick"
  >
    <template #header="{ close, titleId }">
      <slot name="header" v-bind="{ close, titleId }">
        <div>
          <h2 :id="titleId" class="text-sm font-semibold text-foreground-highlighted">
            {{ title }}
          </h2>
          <p v-if="description" class="mt-1 text-sm text-foreground-muted">{{ description }}</p>
        </div>
        <Button
          icon="i-tabler-x"
          variant="ghost"
          size="icon"
          aria-label="Close dialog"
          @click="close"
        />
      </slot>
    </template>
    <template #body><slot name="body" /></template>
    <template #footer><slot name="footer" /></template>
  </AkazaDialog>
</template>
