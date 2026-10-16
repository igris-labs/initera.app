<script setup lang="ts">
import { computed } from 'vue';
import { cn } from '../../../lib/utils';
import { labelSwatch } from '../../../lib/label-presentation';
import { badgeDefaults, badgeVariants } from '.';
import type { BadgeVariants } from '.';

interface Props {
  label?: string;
  variant?: BadgeVariants['variant'];
  size?: BadgeVariants['size'];
  class?: string;
  dot?: boolean;
}

const { label, variant, size, class: customClass, dot = false } = defineProps<Props>();

const classes = computed(() => cn(badgeVariants({ variant, size }), customClass));

const resolvedSize = computed(() => size ?? badgeDefaults.size);
</script>

<template>
  <span :class="classes"
    ><span
      v-if="dot && label"
      :class="['mr-1.5 shrink-0 rounded-full', resolvedSize === 'md' ? 'size-2' : 'size-1.5']"
      :style="{ backgroundColor: labelSwatch(label) }"
      aria-hidden="true"
    /><slot>{{ label }}</slot></span
  >
</template>
