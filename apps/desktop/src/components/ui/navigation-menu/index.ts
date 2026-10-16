import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as NavigationMenu } from './NavigationMenu.vue';
export const navigationMenuDefaults = {
  size: uiConfig.components?.navigationMenu?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.navigationMenu?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const navigationMenuVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: navigationMenuDefaults.variant,
    size: navigationMenuDefaults.size,
  },
});
export type NavigationMenuVariants = VariantProps<typeof navigationMenuVariants>;

export const uiParts: Record<string, string> = {
  root: 'text-sm',
  list: 'flex items-center gap-1',
  trigger: 'flex min-h-7 items-center gap-2 rounded-md px-2 hover:bg-muted',
  link: 'flex min-h-7 items-center gap-2 rounded-md px-2 hover:bg-muted',
  content: 'rounded-lg border border-outline bg-elevated p-2 shadow-lg',
  inlineLink: 'flex min-h-7 items-center gap-2 rounded-md px-2 hover:bg-muted',
  groupLabel: 'px-2 py-1 text-xs text-foreground-muted',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {},
  md: {},
  lg: {},
};

export function navigationMenuUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? navigationMenuDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.content = cn(
    styles.content,
    navigationMenuVariants({ size: resolvedSize, variant }),
    customClass,
  );

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
