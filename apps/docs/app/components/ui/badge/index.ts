import { cva } from 'class-variance-authority';
import type { VariantProps } from 'class-variance-authority';

export { default as Badge } from './Badge.vue';

export const badgeDefaults = { size: 'sm', variant: 'subtle' } as const;

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

declare module '../ui.types' {
  interface UiComponentConfig {
    badge?: Pick<BadgeVariants, 'size' | 'variant'>;
  }
}
