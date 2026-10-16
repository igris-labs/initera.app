import type { VariantProps } from 'class-variance-authority';
import { cva } from 'class-variance-authority';
import { cn } from '../../../lib/utils';
import uiConfig from '../ui.config';
import type { PrimitiveSize, PrimitiveVariant } from '../ui.types';

export { default as MonthRangePicker } from './MonthRangePicker.vue';
export const monthRangePickerDefaults = {
  size: uiConfig.components?.monthRangePicker?.size ?? uiConfig.defaults?.size ?? 'sm',
  variant:
    uiConfig.components?.monthRangePicker?.variant ?? uiConfig.defaults?.variant ?? 'default',
};

export const monthRangePickerVariants = cva('', {
  variants: {
    variant: { default: '', soft: 'bg-muted', outline: 'border border-outline' },
    size: { sm: 'text-xs', md: 'text-sm', lg: 'text-base' },
  },
  defaultVariants: {
    variant: monthRangePickerDefaults.variant,
    size: monthRangePickerDefaults.size,
  },
});
export type MonthRangePickerVariants = VariantProps<typeof monthRangePickerVariants>;

export const uiParts: Record<string, string> = {
  root: 'rounded-lg border border-outline bg-elevated p-3',
  header: 'flex items-center justify-between',
  previous:
    'inline-flex size-7 items-center justify-center rounded-md text-foreground-muted transition-colors duration-150 hover:bg-accented hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-inverted disabled:opacity-40',
  next: 'inline-flex size-7 items-center justify-center rounded-md text-foreground-muted transition-colors duration-150 hover:bg-accented hover:text-foreground-highlighted focus-visible:outline-2 focus-visible:outline-inverted disabled:opacity-40',
  heading: 'text-sm font-medium',
  grid: 'w-full',
  cellTrigger:
    'rounded-md hover:bg-accented data-akaza-selected:bg-inverted data-akaza-selected:text-foreground-inverted',
};

export const uiSizes: Record<PrimitiveSize, Record<string, string>> = {
  sm: {
    cellTrigger: 'size-7 text-xs',
    previous: 'size-7 text-xs',
    next: 'size-7 text-xs',
    heading: 'text-xs',
  },
  md: {
    cellTrigger: 'size-8 text-sm',
    previous: 'size-8 text-sm',
    next: 'size-8 text-sm',
    heading: 'text-sm',
  },
  lg: {
    cellTrigger: 'size-9 text-base',
    previous: 'size-9 text-base',
    next: 'size-9 text-base',
    heading: 'text-base',
  },
};

export function monthRangePickerUi(
  size?: PrimitiveSize,
  variant?: PrimitiveVariant,
  customClass?: string,
  overrides?: Record<string, string>,
): Record<string, string> {
  const resolvedSize = size ?? monthRangePickerDefaults.size;
  const styles = { ...uiParts };

  for (const [part, classes] of Object.entries(uiSizes[resolvedSize])) {
    if (part in styles) styles[part] = cn(styles[part], classes);
  }

  styles.root = cn(
    styles.root,
    monthRangePickerVariants({ size: resolvedSize, variant }),
    customClass,
  );

  for (const [part, classes] of Object.entries(overrides ?? {})) {
    styles[part] = cn(styles[part], classes);
  }

  return styles;
}
