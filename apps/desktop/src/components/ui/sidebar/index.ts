import { cva } from 'class-variance-authority';
import type { VariantProps } from 'class-variance-authority';
import uiConfig from '../ui.config';

export { default as Sidebar } from './Sidebar.vue';

export const sidebarWidths = { sm: 224, md: 240, lg: 288 } as const;

export const sidebarDefaults = {
  size:
    (['sm', 'md', 'lg'] as const).find(
      (value) => value === (uiConfig.components?.sidebar?.size ?? uiConfig.defaults?.size),
    ) ?? 'sm',
  variant:
    (['default', 'muted'] as const).find(
      (value) => value === (uiConfig.components?.sidebar?.variant ?? uiConfig.defaults?.variant),
    ) ?? 'default',
};

export const sidebarVariants = cva('h-full text-foreground', {
  variants: {
    variant: {
      default: 'bg-shell',
      muted: 'bg-muted',
    },
    size: {
      sm: 'text-xs',
      md: 'text-sm',
      lg: 'text-base',
    },
  },
  defaultVariants: {
    variant: sidebarDefaults.variant,
    size: sidebarDefaults.size,
  },
});

export type SidebarVariants = VariantProps<typeof sidebarVariants>;
