import { cva } from 'class-variance-authority';
import type { VariantProps } from 'class-variance-authority';
import uiConfig from '../ui.config';

export { default as Popover } from './Popover.vue';

export const popoverDefaults = {
  size:
    (['sm', 'md', 'lg'] as const).find(
      (value) => value === (uiConfig.components?.popover?.size ?? uiConfig.defaults?.size),
    ) ?? 'sm',
  variant:
    (['default', 'quiet'] as const).find(
      (value) => value === (uiConfig.components?.popover?.variant ?? uiConfig.defaults?.variant),
    ) ?? 'default',
};

export const popoverVariants = cva(
  'rounded-lg border border-outline bg-elevated text-foreground shadow-lg',
  {
    variants: {
      variant: { default: '', quiet: 'shadow-md' },
      size: { sm: 'w-56 p-2', md: 'w-72 p-3', lg: 'w-96 p-4' },
    },
    defaultVariants: {
      variant: popoverDefaults.variant,
      size: popoverDefaults.size,
    },
  },
);

export type PopoverVariants = VariantProps<typeof popoverVariants>;
