import { cva } from 'class-variance-authority';
import type { VariantProps } from 'class-variance-authority';
import uiConfig from '../ui.config';

export { default as Switch } from './Switch.vue';

export const switchDefaults = {
  size:
    (['sm', 'md'] as const).find(
      (value) => value === (uiConfig.components?.switch?.size ?? uiConfig.defaults?.size),
    ) ?? 'sm',
  variant:
    (['default', 'subtle'] as const).find(
      (value) => value === (uiConfig.components?.switch?.variant ?? uiConfig.defaults?.variant),
    ) ?? 'default',
};

export const switchVariants = cva(
  'inline-flex shrink-0 cursor-pointer items-center rounded-full border border-outline bg-accented p-0.5 transition-colors duration-150 ease-out data-[akaza-state=checked]:border-inverted data-[akaza-state=checked]:bg-inverted focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-inverted',
  {
    variants: {
      size: { sm: 'h-4 w-8', md: 'h-5 w-9' },
      variant: { default: '', subtle: 'border-transparent bg-elevated' },
    },
    defaultVariants: {
      size: switchDefaults.size,
      variant: switchDefaults.variant,
    },
  },
);

export type SwitchVariants = VariantProps<typeof switchVariants>;
