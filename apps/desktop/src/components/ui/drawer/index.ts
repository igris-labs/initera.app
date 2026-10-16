import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Drawer } from './Drawer.vue';
export const drawerDefaults = {
  size: uiConfig.components?.drawer?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.drawer?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const drawerVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: drawerDefaults.variant,
    size: drawerDefaults.size,
  },
});
export type DrawerVariants = VariantProps<typeof drawerVariants>;

export const uiParts: Record<string, string> = {
  overlay: 'fixed inset-0 bg-scrim',
  content: 'border-l border-outline bg-elevated text-foreground shadow-lg',
  header: 'border-b border-outline p-4',
  body: 'overflow-auto p-4',
  footer: 'border-t border-outline p-4',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {},
  md: {},
  lg: {},
};

export function drawerUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? drawerDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.content = cn(styles.content, drawerVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
