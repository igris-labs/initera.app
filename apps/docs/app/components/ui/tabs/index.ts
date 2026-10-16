import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export const tabsDefaults = {
  size: uiConfig.components?.tabs?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.tabs?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const tabsVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: tabsDefaults.variant,
    size: tabsDefaults.size,
  },
});
export type TabsVariants = VariantProps<typeof tabsVariants>;
export { default as Tabs } from './Tabs.vue';

export const uiParts: Record<string, string> = {
  root: 'text-sm',
  list: 'inline-flex items-center gap-0.5 rounded-lg bg-muted p-0.5',
  tab: 'min-h-6 rounded-md px-2 text-xs font-medium text-foreground-muted data-[akaza-state=active]:bg-default data-[akaza-state=active]:text-foreground data-[akaza-state=active]:shadow-sm data-[akaza-state=active]:ring-1 data-[akaza-state=active]:ring-outline',
  panel: 'pt-3',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    tab: 'text-xs min-h-7 px-2',
  },
  md: {
    tab: 'text-sm min-h-8 px-2.5',
  },
  lg: {
    tab: 'text-base min-h-9 px-3',
  },
};

export function tabsUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? tabsDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, tabsVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
