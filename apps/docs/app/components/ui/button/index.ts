import { cva } from 'class-variance-authority';
import type { VariantProps } from 'class-variance-authority';
import uiConfig from '../ui.config';

export { default as Button } from './Button.vue';

export const buttonDefaults = {
  size:
    (['xs', 'sm', 'md', 'lg', 'icon'] as const).find(
      (value) => value === (uiConfig.components?.button?.size ?? uiConfig.defaults?.size),
    ) ?? 'sm',
  variant:
    (['solid', 'outline', 'soft', 'ghost', 'link'] as const).find(
      (value) => value === (uiConfig.components?.button?.variant ?? uiConfig.defaults?.variant),
    ) ?? 'solid',
};

export const buttonVariants = cva(
  'inline-flex cursor-pointer touch-manipulation items-center justify-center rounded-lg border font-medium whitespace-nowrap transition-[background-color,color,border-color,transform] duration-150 ease-out focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-inverted disabled:cursor-not-allowed disabled:opacity-55',
  {
    variants: {
      variant: {
        solid: 'ipex-execution',
        outline: 'border-outline bg-default text-foreground hover:bg-muted active:bg-accented',
        soft: 'border-transparent bg-muted text-foreground hover:bg-accented',
        ghost: 'border-transparent bg-transparent text-foreground hover:bg-accented/70',
        link: 'border-transparent bg-transparent text-foreground underline-offset-4 hover:underline',
      },
      size: {
        xs: 'min-h-6 gap-1 px-2 text-xs',
        sm: 'min-h-7 gap-1.5 px-2.5 text-xs',
        md: 'min-h-8 gap-1.5 px-2.5 text-sm',
        lg: 'min-h-9 gap-2 px-3.5 text-base',
        icon: 'size-7 p-0 text-xs',
      },
      square: {
        true: '',
        false: '',
      },
      color: {
        neutral: '',
        info: '',
        success: '',
        warning: '',
        error: '',
      },
    },
    compoundVariants: [
      { size: 'xs', square: true, class: 'w-6 px-0' },
      { size: 'sm', square: true, class: 'w-7 px-0' },
      { size: 'md', square: true, class: 'w-8 px-0' },
      { size: 'lg', square: true, class: 'w-9 px-0' },
      { variant: ['ghost', 'link'], color: 'info', class: 'text-info' },
      { variant: ['ghost', 'link'], color: 'success', class: 'text-success' },
      { variant: ['ghost', 'link'], color: 'warning', class: 'text-warning' },
      { variant: ['ghost', 'link'], color: 'error', class: 'text-error' },
    ],
    defaultVariants: {
      variant: buttonDefaults.variant,
      size: buttonDefaults.size,
      color: 'neutral',
    },
  },
);

export type ButtonVariants = VariantProps<typeof buttonVariants>;
