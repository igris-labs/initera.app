<script setup lang="ts">
import { Button as AkazaButton } from 'akaza-ui';
import { computed, useSlots } from 'vue';
import type { HTMLAttributes } from 'vue';
import { cn } from '../../../lib/utils';
import { buttonDefaults, buttonVariants } from '.';
import type { ButtonVariants } from '.';
import { Icon } from '../icon';

interface Props {
  label?: string;
  icon?: string;
  variant?: ButtonVariants['variant'];
  size?: ButtonVariants['size'];
  square?: boolean;
  color?: ButtonVariants['color'];
  type?: 'button' | 'submit' | 'reset';
  disabled?: boolean;
  class?: HTMLAttributes['class'];
}

const {
  label,
  icon,
  variant,
  size,
  square,
  color = 'neutral',
  type = 'button',
  disabled,
  class: customClass,
} = defineProps<Props>();

const slots = useSlots();

const resolvedSize = computed(() => size ?? buttonDefaults.size);

const resolvedSquare = computed(() => square ?? Boolean(icon && !label && !slots.default));

const classes = computed(() =>
  cn(
    buttonVariants({ variant, size: resolvedSize.value, square: resolvedSquare.value, color }),
    customClass,
  ),
);
</script>

<template>
  <AkazaButton :type="type" :disabled="disabled" :class="classes">
    <Icon
      v-if="icon"
      :name="icon"
      :size="resolvedSize === 'md' || resolvedSize === 'lg' ? 'md' : 'sm'"
    />
    <slot>{{ label }}</slot>
  </AkazaButton>
</template>
