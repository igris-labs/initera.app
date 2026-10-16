import { cva } from 'class-variance-authority';
import type { VariantProps } from 'class-variance-authority';
import uiConfig from '../ui.config';

export { default as Select } from './Select.vue';

export const selectDefaults = {
  size:
    (['xs', 'sm', 'md'] as const).find(
      (value) => value === (uiConfig.components?.select?.size ?? uiConfig.defaults?.size),
    ) ?? 'sm',
  variant:
    (['outline', 'soft', 'ghost', 'pill'] as const).find(
      (value) => value === (uiConfig.components?.select?.variant ?? uiConfig.defaults?.variant),
    ) ?? 'outline',
};

export const selectVariants = cva(
  'flex cursor-pointer items-center justify-between gap-2 border text-left text-foreground outline-none transition-[background-color,border-color] duration-150 ease-out focus-visible:border-inverted focus-visible:ring-2 focus-visible:ring-inverted/15 disabled:cursor-not-allowed disabled:opacity-55',
  {
    variants: {
      variant: {
        outline:
          'w-full rounded-lg border-outline bg-default hover:bg-muted data-[akaza-state=open]:bg-muted',
        soft: 'w-full rounded-lg border-transparent bg-muted hover:bg-accented data-[akaza-state=open]:bg-accented',
        ghost:
          'w-fit max-w-full rounded-lg border-transparent bg-transparent hover:bg-muted/75 data-[akaza-state=open]:bg-muted/75',
        pill: 'w-auto rounded-lg border-outline bg-default hover:bg-muted dark:bg-muted dark:hover:bg-accented data-[akaza-state=open]:bg-accented',
      },
      size: {
        xs: 'min-h-6 px-2 text-xs',
        sm: 'min-h-7 px-2.5 text-xs',
        md: 'min-h-8 px-3 text-sm',
      },
    },
    defaultVariants: {
      variant: selectDefaults.variant,
      size: selectDefaults.size,
    },
  },
);

export type SelectVariants = VariantProps<typeof selectVariants>;
