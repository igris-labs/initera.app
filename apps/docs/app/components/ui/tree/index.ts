import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export const treeDefaults = {
  size: uiConfig.components?.tree?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.tree?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const treeVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: treeDefaults.variant,
    size: treeDefaults.size,
  },
});
export type TreeVariants = VariantProps<typeof treeVariants>;
export { default as Tree } from './Tree.vue';

export const uiParts: Record<string, string> = {
  root: 'text-sm',
  row: 'flex min-h-7 items-center gap-1 rounded-md px-2 hover:bg-muted data-akaza-selected:bg-accented',
  label: 'truncate',
  loading: 'p-2 text-xs text-foreground-muted',
  empty: 'p-2 text-xs text-foreground-muted',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    row: 'text-xs min-h-7 px-2',
  },
  md: {
    row: 'text-sm min-h-8 px-2.5',
  },
  lg: {
    row: 'text-base min-h-9 px-3',
  },
};

export function treeUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? treeDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, treeVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
