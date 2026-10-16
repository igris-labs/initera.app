import { cva } from 'class-variance-authority';
import type { VariantProps } from 'class-variance-authority';
import uiConfig from '../ui.config';

export { default as DropdownMenu } from './DropdownMenu.vue';

export const dropdownMenuDefaults = {
  size:
    (['sm', 'md', 'lg'] as const).find(
      (value) => value === (uiConfig.components?.dropdownMenu?.size ?? uiConfig.defaults?.size),
    ) ?? 'sm',
  variant:
    (['default', 'quiet'] as const).find(
      (value) =>
        value === (uiConfig.components?.dropdownMenu?.variant ?? uiConfig.defaults?.variant),
    ) ?? 'default',
};

export const dropdownMenuVariants = cva(
  'rounded-lg border border-outline bg-elevated text-foreground shadow-lg',
  {
    variants: {
      variant: { default: '', quiet: 'shadow-md' },
      size: { sm: 'min-w-36', md: 'min-w-44', lg: 'min-w-56' },
    },
    defaultVariants: {
      variant: dropdownMenuDefaults.variant,
      size: dropdownMenuDefaults.size,
    },
  },
);

export type DropdownMenuVariants = VariantProps<typeof dropdownMenuVariants>;
