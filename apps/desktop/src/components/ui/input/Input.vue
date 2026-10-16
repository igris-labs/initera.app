<script setup lang="ts">
import { Input as AkazaInput } from 'akaza-ui';
import { computed, ref } from 'vue';
import { cn } from '../../../lib/utils';
import { inputDefaults, inputVariants } from '.';
import type { InputVariants } from '.';
import { Icon } from '../icon';

interface Props {
  modelValue?: string;
  icon?: string;
  size?: InputVariants['size'];
  variant?: InputVariants['variant'];
  class?: string;
  ui?: { root?: string };
}

defineOptions({ inheritAttrs: false });

const { modelValue = '', icon, size, variant, class: customClass, ui } = defineProps<Props>();

const emit = defineEmits<{ 'update:modelValue': [value: string] }>();

const wrapper = ref<HTMLSpanElement | null>(null);

const inputRef = computed(() => wrapper.value?.querySelector('input') ?? undefined);

const resolvedSize = computed(() => size ?? inputDefaults.size);

defineExpose({ inputRef });
</script>

<template>
  <span ref="wrapper" :class="cn('relative block', customClass)">
    <Icon
      v-if="icon"
      :name="icon"
      :size="resolvedSize === 'lg' ? 'md' : 'sm'"
      :class="[
        'pointer-events-none absolute top-1/2 -translate-y-1/2 text-foreground-dimmed',
        resolvedSize === 'lg' ? 'left-3.5' : resolvedSize === 'md' ? 'left-3' : 'left-2.5',
      ]"
    />
    <AkazaInput
      v-bind="$attrs"
      :model-value="modelValue"
      :ui="{
        root: cn(
          inputVariants({ size, variant }),
          icon && (resolvedSize === 'lg' ? 'pl-10' : resolvedSize === 'md' ? 'pl-9' : 'pl-8'),
          ui?.root,
        ),
      }"
      @update:model-value="emit('update:modelValue', $event)"
    />
  </span>
</template>
