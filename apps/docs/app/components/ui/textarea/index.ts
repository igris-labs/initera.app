import { cva } from 'class-variance-authority';
import type { VariantProps } from 'class-variance-authority';
import uiConfig from '../ui.config';

export { default as Textarea } from './Textarea.vue';

export const textareaDefaults = {
  size:
    (['sm', 'md', 'lg'] as const).find(
      (value) => value === (uiConfig.components?.textarea?.size ?? uiConfig.defaults?.size),
    ) ?? 'sm',
  variant:
    (['outline', 'soft', 'bare'] as const).find(
      (value) => value === (uiConfig.components?.textarea?.variant ?? uiConfig.defaults?.variant),
    ) ?? 'outline',
};

export const textareaVariants = cva(
  'w-full resize-y rounded-lg border border-outline bg-default text-foreground shadow-none outline-none placeholder:text-foreground-muted focus-visible:border-inverted focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-inverted/35 focus-visible:shadow-none focus-visible:ring-0 disabled:cursor-not-allowed disabled:opacity-55',
  {
    variants: {
      size: {
        sm: 'min-h-16 px-2.5 py-1.5 text-xs',
        md: 'min-h-24 px-3 py-2 text-sm',
        lg: 'min-h-32 px-3.5 py-2 text-base',
      },
      variant: {
        outline: '',
        soft: 'border-transparent bg-muted',
        bare: 'border-transparent bg-transparent focus-visible:outline-0',
      },
    },
    defaultVariants: {
      size: textareaDefaults.size,
      variant: textareaDefaults.variant,
    },
  },
);

export type TextareaVariants = VariantProps<typeof textareaVariants>;
