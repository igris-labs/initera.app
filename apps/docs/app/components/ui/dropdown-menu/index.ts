import { cva } from 'class-variance-authority';
import type { VariantProps } from 'class-variance-authority';

export { default as DropdownMenu } from './DropdownMenu.vue';

export const dropdownMenuDefaults = { size: 'sm', variant: 'default' } as const;

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

declare module '../ui.types' {
  interface UiComponentConfig {
    dropdownMenu?: Pick<DropdownMenuVariants, 'size' | 'variant'>;
  }
}
