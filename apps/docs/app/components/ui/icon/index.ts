import { cva } from 'class-variance-authority';
import type { VariantProps } from 'class-variance-authority';

export { default as Icon } from './Icon.vue';

export const iconDefaults = {
  size: 'sm',
  variant: 'default',
} as const;

export const iconVariants = cva('shrink-0', {
  variants: {
    variant: {
      default: 'text-current',
      muted: 'text-foreground-muted',
      dimmed: 'text-foreground-dimmed',
      info: 'text-info',
    },
    size: { xs: 'size-3', sm: 'size-4', md: 'size-5', lg: 'size-6' },
  },
  defaultVariants: {
    variant: iconDefaults.variant,
    size: iconDefaults.size,
  },
});

export type IconVariants = VariantProps<typeof iconVariants>;
