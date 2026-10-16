import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as Pagination } from './Pagination.vue';
export const paginationDefaults = {
  size: uiConfig.components?.pagination?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant: uiConfig.components?.pagination?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const paginationVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: paginationDefaults.variant,
    size: paginationDefaults.size,
  },
});
export type PaginationVariants = VariantProps<typeof paginationVariants>;

export const uiParts: Record<string, string> = {
  root: 'flex items-center gap-1',
  list: 'flex items-center gap-1',
  item: 'inline-flex size-7 items-center justify-center rounded-md text-xs hover:bg-muted',
  page: 'data-akaza-active:bg-accented',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    item: 'size-7 text-xs',
  },
  md: {
    item: 'size-8 text-sm',
  },
  lg: {
    item: 'size-9 text-base',
  },
};

export function paginationUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? paginationDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(styles.root, paginationVariants({ size: resolvedSize, variant }), customClass);

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
