import { cva } from 'class-variance-authority';
import type { VariantProps } from 'class-variance-authority';
import uiConfig from '../ui.config';

export { default as Input } from './Input.vue';

export const inputDefaults = {
  size:
    (['sm', 'md', 'lg'] as const).find(
      (value) => value === (uiConfig.components?.input?.size ?? uiConfig.defaults?.size),
    ) ?? 'sm',
  variant:
    (['outline', 'soft', 'bare'] as const).find(
      (value) => value === (uiConfig.components?.input?.variant ?? uiConfig.defaults?.variant),
    ) ?? 'outline',
};

export const inputVariants = cva(
  'w-full rounded-lg border border-outline bg-default text-foreground shadow-none outline-none placeholder:text-foreground-muted focus-visible:border-inverted focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-inverted/35 focus-visible:shadow-none focus-visible:ring-0 disabled:cursor-not-allowed disabled:opacity-55',
  {
    variants: {
      size: {
        sm: 'min-h-7 px-2.5 text-xs',
        md: 'min-h-8 px-3 text-sm',
        lg: 'min-h-9 px-3.5 text-base',
      },
      variant: {
        outline: '',
        soft: 'border-transparent bg-muted',
        bare: 'border-transparent bg-transparent focus-visible:outline-0',
      },
    },
    defaultVariants: {
      size: inputDefaults.size,
      variant: inputDefaults.variant,
    },
  },
);

export type InputVariants = VariantProps<typeof inputVariants>;
