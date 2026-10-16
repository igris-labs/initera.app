import { cva } from 'class-variance-authority';
import type { VariantProps } from 'class-variance-authority';
import uiConfig from '../ui.config';

export { default as Icon } from './Icon.vue';

export const iconDefaults = {
  size:
    (['xs', 'sm', 'md', 'lg'] as const).find(
      (value) => value === (uiConfig.components?.icon?.size ?? uiConfig.defaults?.size),
    ) ?? 'sm',
  variant:
    (['default', 'muted', 'dimmed', 'info'] as const).find(
      (value) => value === (uiConfig.components?.icon?.variant ?? uiConfig.defaults?.variant),
    ) ?? 'default',
};

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
