import { cva } from 'class-variance-authority';
import type { VariantProps } from 'class-variance-authority';
import uiConfig from '../ui.config';

export { default as Badge } from './Badge.vue';

export const badgeDefaults = {
  size:
    (['xs', 'sm', 'md'] as const).find(
      (value) => value === (uiConfig.components?.badge?.size ?? uiConfig.defaults?.size),
    ) ?? 'sm',
  variant:
    (['subtle', 'outline', 'solid'] as const).find(
      (value) => value === (uiConfig.components?.badge?.variant ?? uiConfig.defaults?.variant),
    ) ?? 'subtle',
};

export const badgeVariants = cva(
  'inline-flex items-center rounded-md border font-medium whitespace-nowrap',
  {
    variants: {
      variant: {
        subtle: 'border-outline bg-muted text-foreground',
        outline: 'border-outline bg-transparent text-foreground',
        solid: 'border-transparent bg-inverted text-foreground-inverted',
      },
      size: { xs: 'px-1 py-0 text-xs', sm: 'px-1.5 py-0.5 text-xs', md: 'px-2 py-1 text-sm' },
    },
    defaultVariants: {
      variant: badgeDefaults.variant,
      size: badgeDefaults.size,
    },
  },
);

export type BadgeVariants = VariantProps<typeof badgeVariants>;
