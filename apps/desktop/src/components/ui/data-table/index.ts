import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as DataTable } from './DataTable.vue';
export const dataTableDefaults = {
  size: uiConfig.components?.dataTable?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.dataTable?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const dataTableVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: dataTableDefaults.variant,
    size: dataTableDefaults.size,
  },
});
export type DataTableVariants = VariantProps<typeof dataTableVariants>;

export const uiParts: Record<string, string> = {
  root: 'overflow-hidden rounded-lg border border-outline bg-elevated text-sm',
  viewport: 'overflow-auto',
  table: 'w-full border-collapse',
  headerCell:
    'border-b border-outline bg-muted px-3 py-2 text-left text-xs font-medium text-foreground-muted',
  cell: 'border-b border-outline px-3 py-2',
  row: 'hover:bg-muted/50',
  pagination: 'flex items-center justify-end gap-1 p-2',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    cell: 'text-xs px-2 py-1.5',
    headerCell: 'text-xs min-h-7 px-2',
  },
  md: {
    cell: 'text-sm px-3 py-2',
    headerCell: 'text-xs min-h-8 px-2.5 text-sm',
  },
  lg: {
    cell: 'text-base px-3.5 py-2.5',
    headerCell: 'text-sm min-h-9 px-3 text-base',
  },
};

export function dataTableUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? dataTableDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, dataTableVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
