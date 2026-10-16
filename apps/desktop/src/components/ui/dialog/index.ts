import { cva } from 'class-variance-authority';
import type { VariantProps } from 'class-variance-authority';
import uiConfig from '../ui.config';

export { default as Dialog } from './Dialog.vue';

export const dialogDefaults = {
  size:
    (['sm', 'md', 'lg'] as const).find(
      (value) => value === (uiConfig.components?.dialog?.size ?? uiConfig.defaults?.size),
    ) ?? 'sm',
  variant:
    (['default', 'quiet'] as const).find(
      (value) => value === (uiConfig.components?.dialog?.variant ?? uiConfig.defaults?.variant),
    ) ?? 'default',
};

export const dialogVariants = cva(
  'fixed top-1/2 left-1/2 flex max-h-[calc(100dvh-2rem)] min-h-0 w-[calc(100vw-2rem)] min-w-0 -translate-x-1/2 -translate-y-1/2 flex-col overflow-hidden rounded-xl border border-outline bg-elevated text-foreground shadow-xl',
  {
    variants: {
      variant: { default: '', quiet: 'shadow-lg' },
      size: { sm: 'max-w-sm', md: 'max-w-lg', lg: 'max-w-2xl' },
    },
    defaultVariants: {
      variant: dialogDefaults.variant,
      size: dialogDefaults.size,
    },
  },
);

export type DialogVariants = VariantProps<typeof dialogVariants>;
